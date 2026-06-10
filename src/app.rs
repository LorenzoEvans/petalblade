use ratatui::widgets::ListState;
use reqwest::get;
use rodio::{Decoder, OutputStream, Sink};
use rss::Channel;
use std::{
    error::Error,
    io::BufReader,
    sync::{Arc, RwLock},
};
use stream_download::http::HttpStream;
use stream_download::http::reqwest::Client;
use stream_download::source::SourceStream;
use stream_download::storage::temp::TempStorageProvider;
use stream_download::{Settings, StreamDownload};
use tokio::sync::{mpsc, oneshot};
use tokio::task::JoinHandle;

type AudioResult<T> = Result<T, Box<dyn std::error::Error + Send + Sync>>;
pub type AppResult<T> = Result<T, Box<dyn Error>>;

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

    pub handle: JoinHandle<AudioResult<()>>,
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
        let rt_handle = tokio::runtime::Handle::current();
        let (done_tx, done_rx) = oneshot::channel();
        
        std::thread::spawn(move || {
            let (_stream, stream_handle) = match OutputStream::try_default() {
                Ok(s) => s,
                Err(_) => return,
            };
            let sink = match Sink::try_new(&stream_handle) {
                Ok(s) => Arc::new(s),
                Err(_) => return,
            };
            
            while let Some(command) = rt_handle.block_on(rx.recv()) {
                match command {
                    AudioCommand::Play(url) => {
                        sink.stop();
                        let sink_clone = Arc::clone(&sink);
                        rt_handle.spawn(async move {
                            match HttpStream::<Client>::create(url.parse().unwrap()).await {
                                Ok(stream) => {
                                    match StreamDownload::from_stream(
                                        stream,
                                        TempStorageProvider::default(),
                                        Settings::default(),
                                    ).await {
                                        Ok(reader) => {
                                            if let Ok(source) = Decoder::new(BufReader::new(reader)) {
                                                sink_clone.append(source);
                                                sink_clone.play();
                                            }
                                        }
                                        Err(e) => eprintln!("StreamDownload error: {}", e),
                                    }
                                }
                                Err(e) => eprintln!("HttpStream error: {}", e),
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
                        sink.stop();
                    }
                    AudioCommand::Volume(vol) => {
                        sink.set_volume(vol);
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
            audio_manager: AudioManager { tx, handle },
            volume: 1.0,
            search_query: String::new(),
            input_mode: InputMode::Normal,
            show_help: false,
            status_message: String::new(),
        }
    }
    pub fn quit(&mut self) {
        self.running = false;
    }
}

#[derive(Debug, Clone)]
pub enum PlaybackState {
    Playing,
    Paused,
    Stopped,
    Muted,
}

#[derive(Debug)]
pub enum AudioCommand {
    NextEpisode,
    PrevEpisode,
    Play(String),
    Pause,
    Stop,
    Volume(f32),
}

const MFP_FEED: &str = "https://musicforprogramming.net/rss.xml";

pub async fn music_for_programming() -> Result<Vec<Episode>, Box<dyn Error>> {
    let response = match get(MFP_FEED).await {
        Ok(res) => res,
        Err(e) => {
            return Err(Box::new(e));
        }
    };
    let mut episodes = Vec::new();

    if response.status().is_success() {
        let content = response.text().await?;
        let channel = Channel::read_from(content.as_bytes())?;

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
    }

    Ok(episodes)
}
