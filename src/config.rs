use crate::app::Episode;
use serde::{Deserialize, Serialize};
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

const CONFIG_FILE: &str = "config.toml";

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(default)]
pub struct UserConfig {
    pub volume: f32,
    pub favorites: Vec<FavoriteEpisode>,
    pub last_selected: Option<EpisodeRef>,
    pub favorites_only: bool,
}

impl Default for UserConfig {
    fn default() -> Self {
        Self {
            volume: 1.0,
            favorites: Vec::new(),
            last_selected: None,
            favorites_only: false,
        }
    }
}

impl UserConfig {
    pub fn normalized(mut self) -> Self {
        self.volume = self.volume.clamp(0.0, 1.0);
        self
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct EpisodeRef {
    pub title: String,
    pub audio_url: String,
    pub link: String,
}

impl EpisodeRef {
    pub fn from_episode(episode: &Episode) -> Self {
        Self {
            title: episode.title.clone(),
            audio_url: episode.audio_url.clone(),
            link: episode.link.clone(),
        }
    }

    pub fn stable_id(&self) -> &str {
        if self.audio_url.is_empty() {
            &self.link
        } else {
            &self.audio_url
        }
    }

    pub fn matches_episode(&self, episode: &Episode) -> bool {
        let episode_ref = Self::from_episode(episode);
        let self_id = self.stable_id();
        let episode_id = episode_ref.stable_id();

        if self_id.is_empty() || episode_id.is_empty() {
            self.title == episode.title
        } else {
            self_id == episode_id
        }
    }
}

pub type FavoriteEpisode = EpisodeRef;

pub fn default_config_path() -> Option<PathBuf> {
    dirs::config_dir().map(|dir| dir.join("petalblade").join(CONFIG_FILE))
}

pub fn load_config(path: &Path) -> Result<UserConfig, Box<dyn std::error::Error>> {
    match fs::read_to_string(path) {
        Ok(contents) => Ok(toml::from_str::<UserConfig>(&contents)?.normalized()),
        Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(UserConfig::default()),
        Err(error) => Err(Box::new(error)),
    }
}

pub fn save_config(path: &Path, config: &UserConfig) -> Result<(), Box<dyn std::error::Error>> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }

    fs::write(path, toml::to_string_pretty(&config.clone().normalized())?)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn temp_config_path() -> PathBuf {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        std::env::temp_dir()
            .join("petalblade-tests")
            .join(format!("config-{nanos}.toml"))
    }

    #[test]
    fn missing_config_loads_defaults() {
        let path = temp_config_path();
        let config = load_config(&path).unwrap();

        assert_eq!(config, UserConfig::default());
    }

    #[test]
    fn config_round_trips_and_clamps_volume() {
        let path = temp_config_path();
        let config = UserConfig {
            volume: 1.8,
            favorites: vec![FavoriteEpisode {
                title: "Episode 1".to_string(),
                audio_url: "https://example.test/audio.mp3".to_string(),
                link: "https://example.test/episode".to_string(),
            }],
            last_selected: Some(EpisodeRef {
                title: "Episode 2".to_string(),
                audio_url: String::new(),
                link: "https://example.test/episode-2".to_string(),
            }),
            favorites_only: true,
        };

        save_config(&path, &config).unwrap();
        let loaded = load_config(&path).unwrap();

        assert_eq!(loaded.volume, 1.0);
        assert_eq!(loaded.favorites.len(), 1);
        assert!(loaded.favorites_only);
        assert_eq!(
            loaded.last_selected.unwrap().link,
            "https://example.test/episode-2"
        );
    }
}
