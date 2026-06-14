use crate::app::{App, AppResult, AudioCommand, PlaybackState, SelectedList};
use ratatui::crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
/// Handles the key events and updates the state of [`App`].
pub async fn handle_key_events(key_event: KeyEvent, app: &mut App) -> AppResult<()> {
    let visible_episodes = app.visible_episodes();

    match app.input_mode {
        crate::app::InputMode::Normal => match key_event.code {
            // Exit application on `ESC` or `q`
            KeyCode::Esc | KeyCode::Char('q') => {
                if app.show_help {
                    app.show_help = false;
                } else {
                    app.quit();
                }
            }
            // Exit application on `Ctrl-C`
            KeyCode::Char('c') | KeyCode::Char('C')
                if key_event.modifiers == KeyModifiers::CONTROL =>
            {
                app.quit();
            }
            KeyCode::Tab => {
                cycle_focus(app);
            }
            KeyCode::Char('/') => {
                app.selected_list = SelectedList::Search;
                app.input_mode = crate::app::InputMode::Editing;
            }
            KeyCode::Char('h') | KeyCode::Char('?') => {
                app.show_help = !app.show_help;
            }
            KeyCode::Up => match app.selected_list {
                SelectedList::Menu if app.selected_menu_item > 0 => {
                    app.selected_menu_item -= 1;
                    app.menu_list_state.select(Some(app.selected_menu_item));
                }
                SelectedList::Episodes if app.selected_episode > 0 => {
                    app.selected_episode -= 1;
                    app.episode_list_state.select(Some(app.selected_episode));
                }
                SelectedList::About => {
                    app.about_scroll = app.about_scroll.saturating_sub(1);
                }
                SelectedList::Credits => {
                    app.credits_scroll = app.credits_scroll.saturating_sub(1);
                }
                _ => {}
            },
            KeyCode::Down => match app.selected_list {
                SelectedList::Menu if app.selected_menu_item < 1 => {
                    // Assuming 2 menu items for now
                    app.selected_menu_item += 1;
                    app.menu_list_state.select(Some(app.selected_menu_item));
                }
                SelectedList::Episodes
                    if app.selected_episode < visible_episodes.len().saturating_sub(1) =>
                {
                    app.selected_episode += 1;
                    app.episode_list_state.select(Some(app.selected_episode));
                }
                SelectedList::About => {
                    app.about_scroll = app.about_scroll.saturating_add(1);
                }
                SelectedList::Credits => {
                    app.credits_scroll = app.credits_scroll.saturating_add(1);
                }
                _ => {}
            },
            KeyCode::Enter if app.selected_list == SelectedList::Menu => {
                app.show_favorites_only = app.selected_menu_item == 1;
                app.selected_episode = 0;
                app.clamp_selected_episode();
                app.status_message = if app.show_favorites_only {
                    "Showing favorites".to_string()
                } else {
                    "Showing all episodes".to_string()
                };
                app.save_user_config()?;
            }
            KeyCode::Enter
                if app.selected_list == SelectedList::Episodes && !visible_episodes.is_empty() =>
            {
                app.selected_episode = app
                    .selected_episode
                    .min(visible_episodes.len().saturating_sub(1));
                app.episode_list_state.select(Some(app.selected_episode));

                app.playback_state = PlaybackState::Playing;
                let episode = visible_episodes[app.selected_episode].clone();
                app.current_track = Some(episode.clone());
                let url = episode.audio_url.clone();
                app.audio_manager.tx.send(AudioCommand::Play(url)).await?;
                app.save_user_config()?;
            }
            KeyCode::Char(' ') => match app.playback_state {
                PlaybackState::Playing => {
                    app.playback_state = PlaybackState::Paused;
                    app.audio_manager.tx.send(AudioCommand::Pause).await?;
                }
                PlaybackState::Paused => {
                    app.playback_state = PlaybackState::Playing;
                    app.audio_manager.tx.send(AudioCommand::Pause).await?;
                }
                _ => {}
            },
            KeyCode::Char('s') => {
                app.playback_state = PlaybackState::Stopped;
                app.audio_manager.tx.send(AudioCommand::Stop).await?;
            }
            KeyCode::Char('+') | KeyCode::Char('=') => {
                app.volume = (app.volume + 0.1).min(1.0);
                app.audio_manager
                    .tx
                    .send(AudioCommand::Volume(app.volume))
                    .await?;
                app.save_user_config()?;
            }
            KeyCode::Char('-') | KeyCode::Char('_') => {
                app.volume = (app.volume - 0.1).max(0.0);
                app.audio_manager
                    .tx
                    .send(AudioCommand::Volume(app.volume))
                    .await?;
                app.save_user_config()?;
            }
            KeyCode::Char('f') => {
                app.toggle_selected_favorite();
                app.save_user_config()?;
            }
            KeyCode::Char('F') => {
                app.toggle_favorites_filter();
                app.save_user_config()?;
            }
            // Counter handlers
            KeyCode::Right => {}
            KeyCode::Left => {}
            // Other handlers you could add here.
            _ => {}
        },
        crate::app::InputMode::Editing => match key_event.code {
            KeyCode::Tab => {
                cycle_focus(app);
            }
            KeyCode::Enter => {
                app.input_mode = crate::app::InputMode::Normal;
            }
            KeyCode::Char(c) => {
                app.search_query.push(c);
                app.selected_episode = 0;
                app.episode_list_state.select(Some(0));
                app.clamp_selected_episode();
            }
            KeyCode::Backspace => {
                app.search_query.pop();
                app.selected_episode = 0;
                app.episode_list_state.select(Some(0));
                app.clamp_selected_episode();
            }
            KeyCode::Esc => {
                app.input_mode = crate::app::InputMode::Normal;
                if app.selected_list == SelectedList::Search {
                    app.selected_list = SelectedList::Episodes;
                }
            }
            _ => {}
        },
    }
    Ok(())
}

fn cycle_focus(app: &mut App) {
    app.selected_list = match app.selected_list {
        SelectedList::Menu => SelectedList::Episodes,
        SelectedList::Episodes => SelectedList::Search,
        SelectedList::Search => SelectedList::About,
        SelectedList::About => SelectedList::Credits,
        SelectedList::Credits => SelectedList::Menu,
    };

    app.input_mode = if app.selected_list == SelectedList::Search {
        crate::app::InputMode::Editing
    } else {
        crate::app::InputMode::Normal
    };
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::app::{AudioManager, Episode, InputMode};
    use ratatui::widgets::ListState;
    use std::sync::{Arc, RwLock};
    use stream_download::http::reqwest::Client;
    use tokio::sync::mpsc;

    fn episode(title: &str, audio_url: &str) -> Episode {
        Episode {
            title: title.to_string(),
            audio_url: audio_url.to_string(),
            author: String::new(),
            duration: String::new(),
            key_words: String::new(),
            pub_date: String::new(),
            link: String::new(),
        }
    }

    fn key(code: KeyCode) -> KeyEvent {
        KeyEvent::new(code, KeyModifiers::NONE)
    }

    fn test_app() -> App {
        let (_status_tx, status_rx) = mpsc::unbounded_channel();
        let mut episode_list_state = ListState::default();
        episode_list_state.select(Some(0));
        let mut menu_list_state = ListState::default();
        menu_list_state.select(Some(0));

        App {
            episodes: Arc::new(RwLock::new(vec![
                episode("Episode 1: Ambient", "https://example.test/1.mp3"),
                episode("Episode 2: Beats", "https://example.test/2.mp3"),
            ])),
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
                tx: mpsc::channel(4).0,
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
            audio_status_rx: status_rx,
        }
    }

    #[tokio::test]
    async fn f_toggles_selected_episode_favorite() {
        let mut app = test_app();

        handle_key_events(key(KeyCode::Char('f')), &mut app)
            .await
            .unwrap();
        assert_eq!(app.favorites.len(), 1);
        assert!(app.is_favorite(&app.visible_episodes()[0]));

        handle_key_events(key(KeyCode::Char('f')), &mut app)
            .await
            .unwrap();
        assert!(app.favorites.is_empty());
    }

    #[tokio::test]
    async fn capital_f_toggles_favorites_filter() {
        let mut app = test_app();
        handle_key_events(key(KeyCode::Char('f')), &mut app)
            .await
            .unwrap();

        handle_key_events(key(KeyCode::Char('F')), &mut app)
            .await
            .unwrap();

        assert!(app.show_favorites_only);
        assert_eq!(app.visible_episodes().len(), 1);
    }

    #[tokio::test]
    async fn menu_enter_selects_favorites_filter() {
        let mut app = test_app();
        app.selected_list = SelectedList::Menu;
        app.selected_menu_item = 1;

        handle_key_events(key(KeyCode::Enter), &mut app)
            .await
            .unwrap();

        assert!(app.show_favorites_only);
        assert_eq!(app.episode_list_state.selected(), None);
    }

    #[tokio::test]
    async fn tab_cycles_past_search_while_editing() {
        let mut app = test_app();

        handle_key_events(key(KeyCode::Tab), &mut app)
            .await
            .unwrap();
        assert_eq!(app.selected_list, SelectedList::Search);
        assert_eq!(app.input_mode, InputMode::Editing);

        handle_key_events(key(KeyCode::Tab), &mut app)
            .await
            .unwrap();
        assert_eq!(app.selected_list, SelectedList::About);
        assert_eq!(app.input_mode, InputMode::Normal);

        handle_key_events(key(KeyCode::Tab), &mut app)
            .await
            .unwrap();
        assert_eq!(app.selected_list, SelectedList::Credits);
    }
}
