use ratatui::widgets::ListState;
use reqwest::get;
use rodio::{Decoder, OutputStream, Sink};
use rss::Channel;
use std::{
    error::Error,
    io::BufReader,
    path::PathBuf,
    sync::atomic::{AtomicU64, Ordering},
    sync::{Arc, RwLock},
};
use stream_download::http::HttpStream;
use stream_download::http::reqwest::Client;
use stream_download::source::SourceStream;
use stream_download::storage::temp::TempStorageProvider;
use stream_download::{Settings, StreamDownload};
use tokio::sync::{mpsc, oneshot};
use tokio::task::JoinHandle;

use crate::config::{
    EpisodeRef, FavoriteEpisode, UserConfig, default_config_path, load_config, save_config,
};

type AudioResult<T> = Result<T, Box<dyn std::error::Error + Send + Sync>>;
pub type AppResult<T> = Result<T, Box<dyn Error>>;

const AUDIO_PREFETCH_BYTES: u64 = 4 * 1024 * 1024;

#[derive(Debug, Clone)]
pub struct Episode {
    pub title: String,
    pub audio_url: String,
    pub author: String,
    pub duration: String,
    pub key_words: String,
    pub pub_date: String,
    pub link: String,
}

pub struct AudioManager {
    pub tx: mpsc::Sender<AudioCommand>,
    pub handle: Option<JoinHandle<AudioResult<()>>>,
}
pub struct App {
    pub episodes: Arc<RwLock<Vec<Episode>>>,
    pub current_track: Option<Episode>,
    pub playback_state: PlaybackState,
    pub episode_list_state: ListState,
    pub menu_list_state: ListState,
    pub selected_episode: usize,
    pub selected_menu_item: usize,
    pub about_scroll: u16,
    pub credits_scroll: u16,
    pub running: bool,
    pub client: Client,
    pub selected_list: SelectedList,
    pub audio_manager: AudioManager,
    pub volume: f32,
    pub search_query: String,
    pub input_mode: InputMode,
    pub show_help: bool,
    pub status_message: String,
    pub favorites: Vec<FavoriteEpisode>,
    pub show_favorites_only: bool,
    pub config_path: Option<PathBuf>,
    pub audio_status_rx: mpsc::UnboundedReceiver<AudioStatus>,
}

#[derive(Clone, Debug, PartialEq)]
pub enum SelectedList {
    Menu,
    Episodes,
    About,
    Credits,
    Search,
}

#[derive(Debug, PartialEq)]
pub enum InputMode {
    Normal,
    Editing,
}

impl App {
    pub fn new() -> Self {
        let mut episode_list_state = ListState::default();
        episode_list_state.select(Some(0));
        let mut menu_list_state = ListState::default();
        menu_list_state.select(Some(0));
        let client = Client::new();

        let (tx, mut rx) = mpsc::channel(100);
        let (status_tx, status_rx) = mpsc::unbounded_channel();
        let rt_handle = tokio::runtime::Handle::current();
        let (done_tx, done_rx) = oneshot::channel();
        let play_generation = Arc::new(AtomicU64::new(0));

        std::thread::spawn(move || {
            let (_stream, stream_handle) = match OutputStream::try_default() {
                Ok(s) => s,
                Err(e) => {
                    let _ = status_tx.send(AudioStatus::Error(format!("Audio output error: {e}")));
                    let _ = done_tx.send(());
                    return;
                }
            };
            let sink = match Sink::try_new(&stream_handle) {
                Ok(s) => Arc::new(s),
                Err(e) => {
                    let _ = status_tx.send(AudioStatus::Error(format!("Audio sink error: {e}")));
                    let _ = done_tx.send(());
                    return;
                }
            };
            let mut sink = sink;
            let mut current_volume = 1.0;

            while let Some(command) = rt_handle.block_on(rx.recv()) {
                match command {
                    AudioCommand::Play(url) => {
                        let request_id = play_generation.fetch_add(1, Ordering::SeqCst) + 1;
                        sink.stop();
                        sink = match Sink::try_new(&stream_handle) {
                            Ok(new_sink) => Arc::new(new_sink),
                            Err(e) => {
                                let _ = status_tx
                                    .send(AudioStatus::Error(format!("Audio sink error: {e}")));
                                continue;
                            }
                        };
                        sink.set_volume(current_volume);
                        let sink_clone = Arc::clone(&sink);
                        let status_tx = status_tx.clone();
                        let play_generation = Arc::clone(&play_generation);
                        rt_handle.spawn(async move {
                            let url = match url.parse() {
                                Ok(url) => url,
                                Err(e) => {
                                    let _ = status_tx.send(AudioStatus::Error(format!(
                                        "Invalid audio URL: {e}"
                                    )));
                                    return;
                                }
                            };

                            match HttpStream::<Client>::create(url).await {
                                Ok(stream) => {
                                    match StreamDownload::from_stream(
                                        stream,
                                        TempStorageProvider::default(),
                                        Settings::default().prefetch_bytes(AUDIO_PREFETCH_BYTES),
                                    )
                                    .await
                                    {
                                        Ok(reader) => {
                                            if request_id != play_generation.load(Ordering::SeqCst)
                                            {
                                                return;
                                            }

                                            let status_tx = status_tx.clone();
                                            let play_generation = Arc::clone(&play_generation);
                                            let join_result =
                                                tokio::task::spawn_blocking(move || {
                                                    if request_id
                                                        != play_generation.load(Ordering::SeqCst)
                                                    {
                                                        return Ok::<bool, String>(false);
                                                    }

                                                    let source =
                                                        Decoder::new(BufReader::new(reader))
                                                            .map_err(|e| {
                                                                format!("Audio decoder error: {e}")
                                                            })?;

                                                    if request_id
                                                        != play_generation.load(Ordering::SeqCst)
                                                    {
                                                        return Ok(false);
                                                    }

                                                    sink_clone.append(source);
                                                    sink_clone.play();
                                                    Ok(true)
                                                })
                                                .await;

                                            match join_result {
                                                Ok(Ok(true)) => {
                                                    let _ = status_tx.send(AudioStatus::Playing);
                                                }
                                                Ok(Ok(false)) => {}
                                                Ok(Err(message)) => {
                                                    let _ =
                                                        status_tx.send(AudioStatus::Error(message));
                                                }
                                                Err(e) => {
                                                    let _ = status_tx.send(AudioStatus::Error(
                                                        format!("Audio worker error: {e}"),
                                                    ));
                                                }
                                            }
                                        }
                                        Err(e) => {
                                            let _ = status_tx.send(AudioStatus::Error(format!(
                                                "Stream download error: {e}"
                                            )));
                                        }
                                    }
                                }
                                Err(e) => {
                                    let _ = status_tx.send(AudioStatus::Error(format!(
                                        "Audio stream error: {e}"
                                    )));
                                }
                            }
                        });
                    }
                    AudioCommand::Pause => {
                        if sink.is_paused() {
                            sink.play();
                        } else {
                            sink.pause();
                        }
                    }
                    AudioCommand::Stop => {
                        play_generation.fetch_add(1, Ordering::SeqCst);
                        sink.stop();
                    }
                    AudioCommand::Volume(vol) => {
                        current_volume = vol;
                        sink.set_volume(vol);
                    }
                    AudioCommand::Shutdown => {
                        play_generation.fetch_add(1, Ordering::SeqCst);
                        sink.stop();
                        break;
                    }
                    _ => {}
                }
            }
            let _ = done_tx.send(());
        });

        let handle = tokio::spawn(async move {
            let _ = done_rx.await;
            Ok(())
        });

        Self {
            episodes: Arc::new(RwLock::new(Vec::new())),
            current_track: None,
            selected_episode: 0,
            selected_menu_item: 0,
            about_scroll: 0,
            credits_scroll: 0,
            playback_state: PlaybackState::Stopped,
            episode_list_state,
            menu_list_state,
            running: true,
            client,
            selected_list: SelectedList::Episodes,
            audio_manager: AudioManager {
                tx,
                handle: Some(handle),
            },
            volume: 1.0,
            search_query: String::new(),
            input_mode: InputMode::Normal,
            show_help: false,
            status_message: String::new(),
            favorites: Vec::new(),
            show_favorites_only: false,
            config_path: default_config_path(),
            audio_status_rx: status_rx,
        }
    }

    pub fn quit(&mut self) {
        self.running = false;
    }

    pub fn apply_config(&mut self, config: UserConfig) {
        let config = config.normalized();
        self.volume = config.volume;
        self.favorites = config.favorites;
        self.show_favorites_only = config.favorites_only;

        if let Some(last_selected) = config.last_selected {
            self.select_episode_ref(&last_selected);
        }
    }

    pub fn load_user_config(&mut self) -> AppResult<()> {
        if let Some(path) = &self.config_path {
            let config = load_config(path)?;
            self.apply_config(config);
        }
        Ok(())
    }

    pub fn save_user_config(&self) -> AppResult<()> {
        if let Some(path) = &self.config_path {
            save_config(path, &self.user_config())?;
        }
        Ok(())
    }

    pub fn user_config(&self) -> UserConfig {
        UserConfig {
            volume: self.volume,
            favorites: self.favorites.clone(),
            last_selected: self.current_selected_episode().map(|episode| {
                if let Some(current_track) = &self.current_track {
                    EpisodeRef::from_episode(current_track)
                } else {
                    EpisodeRef::from_episode(&episode)
                }
            }),
            favorites_only: self.show_favorites_only,
        }
    }

    pub fn current_selected_episode(&self) -> Option<Episode> {
        self.visible_episodes().get(self.selected_episode).cloned()
    }

    pub fn visible_episodes(&self) -> Vec<Episode> {
        let episodes = self.episodes.read().unwrap();
        episodes
            .iter()
            .filter(|episode| self.episode_is_visible(episode))
            .cloned()
            .collect()
    }

    pub fn episode_is_visible(&self, episode: &Episode) -> bool {
        episode_matches_query(episode, &self.search_query)
            && (!self.show_favorites_only || self.is_favorite(episode))
    }

    pub fn is_favorite(&self, episode: &Episode) -> bool {
        self.favorites
            .iter()
            .any(|favorite| favorite.matches_episode(episode))
    }

    pub fn toggle_selected_favorite(&mut self) {
        let Some(episode) = self.current_selected_episode() else {
            self.status_message = "No episode selected".to_string();
            return;
        };

        let favorite = FavoriteEpisode::from_episode(&episode);
        if let Some(index) = self
            .favorites
            .iter()
            .position(|existing| existing.matches_episode(&episode))
        {
            self.favorites.remove(index);
            self.status_message = format!("Removed favorite: {}", episode.title);
        } else {
            self.favorites.push(favorite);
            self.status_message = format!("Added favorite: {}", episode.title);
        }

        self.clamp_selected_episode();
    }

    pub fn toggle_favorites_filter(&mut self) {
        self.show_favorites_only = !self.show_favorites_only;
        self.selected_episode = 0;
        self.episode_list_state.select(Some(0));
        self.status_message = if self.show_favorites_only {
            "Showing favorites".to_string()
        } else {
            "Showing all episodes".to_string()
        };
    }

    pub fn clamp_selected_episode(&mut self) {
        let visible_len = self.visible_episodes().len();
        if visible_len == 0 {
            self.selected_episode = 0;
            self.episode_list_state.select(None);
        } else {
            self.selected_episode = self.selected_episode.min(visible_len - 1);
            self.episode_list_state.select(Some(self.selected_episode));
        }
    }

    pub fn select_episode_ref(&mut self, episode_ref: &EpisodeRef) {
        let episodes = self.visible_episodes();
        if let Some(index) = episodes
            .iter()
            .position(|episode| episode_ref.matches_episode(episode))
        {
            self.selected_episode = index;
            self.episode_list_state.select(Some(index));
        }
    }

    pub fn drain_audio_status(&mut self) {
        while let Ok(status) = self.audio_status_rx.try_recv() {
            match status {
                AudioStatus::Playing => {
                    self.playback_state = PlaybackState::Playing;
                    self.status_message.clear();
                }
                AudioStatus::Error(message) => {
                    self.status_message = message;
                    self.playback_state = PlaybackState::Stopped;
                }
            }
        }
    }

    pub async fn shutdown_audio(&mut self) -> AppResult<()> {
        let _ = self.audio_manager.tx.send(AudioCommand::Shutdown).await;
        if let Some(handle) = self.audio_manager.handle.take() {
            match handle.await {
                Ok(Ok(())) => {}
                Ok(Err(error)) => return Err(format!("Audio task error: {error}").into()),
                Err(error) => return Err(Box::new(error)),
            }
        }
        Ok(())
    }
}

impl Default for App {
    fn default() -> Self {
        Self::new()
    }
}

pub fn episode_matches_query(episode: &Episode, query: &str) -> bool {
    if query.is_empty() {
        return true;
    }

    let query = query.to_lowercase();
    episode.title.to_lowercase().contains(&query)
        || episode.author.to_lowercase().contains(&query)
        || episode.key_words.to_lowercase().contains(&query)
}

#[derive(Debug, Clone)]
pub enum PlaybackState {
    Playing,
    Paused,
    Stopped,
    Muted,
}

#[derive(Debug)]
pub enum AudioStatus {
    Playing,
    Error(String),
}

#[derive(Debug)]
pub enum AudioCommand {
    NextEpisode,
    PrevEpisode,
    Play(String),
    Pause,
    Stop,
    Volume(f32),
    Shutdown,
}

const MFP_FEED: &str = "https://musicforprogramming.net/rss.xml";

pub async fn music_for_programming() -> Result<Vec<Episode>, Box<dyn Error>> {
    let response = match get(MFP_FEED).await {
        Ok(res) => res,
        Err(e) => {
            return Err(Box::new(e));
        }
    };
    let content = response.error_for_status()?.text().await?;
    parse_episodes(&content)
}

pub fn parse_episodes(content: &str) -> Result<Vec<Episode>, Box<dyn Error>> {
    let channel = Channel::read_from(content.as_bytes())?;
    let mut episodes = Vec::new();

    for item in channel.items() {
        let title = item.title().unwrap_or("Unknown Title").to_owned();
        let audio_url = item.comments().unwrap_or("").to_owned();
        let itunes_ext = item.itunes_ext().cloned();

        let (author, duration, keywords) = if let Some(ext) = itunes_ext {
            (
                ext.author.unwrap_or_else(|| "Unknown".to_string()),
                ext.duration.unwrap_or_else(|| "0:00".to_string()),
                ext.keywords.unwrap_or_else(|| "".to_string()),
            )
        } else {
            ("Unknown".to_string(), "0:00".to_string(), "".to_string())
        };

        let pub_date = item.pub_date().unwrap_or("Unknown Date").to_owned();
        let link = item.link().unwrap_or("").to_owned();
        let episode = Episode {
            title,
            audio_url,
            author,
            duration,
            key_words: keywords,
            pub_date,
            link,
        };
        episodes.push(episode);
    }

    Ok(episodes)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::sync::mpsc;

    fn episode(title: &str, audio_url: &str, link: &str) -> Episode {
        Episode {
            title: title.to_string(),
            audio_url: audio_url.to_string(),
            author: "Author".to_string(),
            duration: "1:23".to_string(),
            key_words: "ambient drones".to_string(),
            pub_date: "Today".to_string(),
            link: link.to_string(),
        }
    }

    fn test_app_with_episodes(episodes: Vec<Episode>) -> App {
        let (_tx, rx) = mpsc::unbounded_channel();
        let mut episode_list_state = ListState::default();
        episode_list_state.select(Some(0));
        let mut menu_list_state = ListState::default();
        menu_list_state.select(Some(0));

        App {
            episodes: Arc::new(RwLock::new(episodes)),
            current_track: None,
            playback_state: PlaybackState::Stopped,
            episode_list_state,
            menu_list_state,
            selected_episode: 0,
            selected_menu_item: 0,
            about_scroll: 0,
            credits_scroll: 0,
            running: true,
            client: Client::new(),
            selected_list: SelectedList::Episodes,
            audio_manager: AudioManager {
                tx: mpsc::channel(1).0,
                handle: None,
            },
            volume: 1.0,
            search_query: String::new(),
            input_mode: InputMode::Normal,
            show_help: false,
            status_message: String::new(),
            favorites: Vec::new(),
            show_favorites_only: false,
            config_path: None,
            audio_status_rx: rx,
        }
    }

    #[test]
    fn parses_rss_items_with_defaults() {
        let rss = r#"<?xml version="1.0" encoding="UTF-8"?>
<rss version="2.0">
  <channel>
    <title>musicForProgramming</title>
    <item>
      <title>Episode 1: Test</title>
      <comments>https://example.test/audio.mp3</comments>
      <link>https://example.test/episode</link>
      <pubDate>Mon, 01 Jan 2024 00:00:00 +0000</pubDate>
    </item>
    <item>
      <comments>https://example.test/untitled.mp3</comments>
    </item>
  </channel>
</rss>"#;

        let episodes = parse_episodes(rss).unwrap();

        assert_eq!(episodes.len(), 2);
        assert_eq!(episodes[0].title, "Episode 1: Test");
        assert_eq!(episodes[0].author, "Unknown");
        assert_eq!(episodes[1].title, "Unknown Title");
    }

    #[test]
    fn visible_episodes_honors_search_and_favorites_filter() {
        let mut app = test_app_with_episodes(vec![
            episode("Episode 1: Ambient", "https://example.test/1.mp3", ""),
            episode("Episode 2: Rhythmic", "https://example.test/2.mp3", ""),
        ]);

        app.toggle_selected_favorite();
        app.search_query = "ambient".to_string();
        app.show_favorites_only = true;

        let visible = app.visible_episodes();

        assert_eq!(visible.len(), 1);
        assert_eq!(visible[0].title, "Episode 1: Ambient");
    }

    #[test]
    fn episode_query_matching_uses_title_author_and_keywords() {
        let episode = episode("Episode 12: Systems", "https://example.test/12.mp3", "");

        assert!(episode_matches_query(&episode, "systems"));
        assert!(episode_matches_query(&episode, "author"));
        assert!(episode_matches_query(&episode, "DRONES"));
        assert!(!episode_matches_query(&episode, "drums"));
    }
}
