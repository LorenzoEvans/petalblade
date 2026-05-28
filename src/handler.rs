use crate::app::{App, AppResult, PlaybackState, SelectedList, AudioCommand};
use ratatui::crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use std::sync::Arc;
/// Handles the key events and updates the state of [`App`].
pub async fn handle_key_events(key_event: KeyEvent, app: &mut App) -> AppResult<()> {
    let episodes_clone = Arc::clone(&app.episodes);

    match key_event.code {
        // Exit application on `ESC` or `q`
        KeyCode::Esc | KeyCode::Char('q') => {
            app.quit();
        }
        // Exit application on `Ctrl-C`
        KeyCode::Char('c') | KeyCode::Char('C') => {
            if key_event.modifiers == KeyModifiers::CONTROL {
                app.quit();
            }
        }
        KeyCode::Tab => {
            app.selected_list = match app.selected_list {
                SelectedList::Menu => SelectedList::Episodes,
                SelectedList::Episodes => SelectedList::About,
                SelectedList::About => SelectedList::Credits,
                SelectedList::Credits => SelectedList::Menu,
            };
        }
        KeyCode::Up => match app.selected_list {
            SelectedList::Menu => {
                if app.selected_menu_item > 0 {
                    app.selected_menu_item -= 1;
                    app.menu_list_state.select(Some(app.selected_menu_item));
                }
            }
            SelectedList::Episodes => {
                if app.selected_episode > 0 {
                    app.selected_episode -= 1;
                    app.episode_list_state.select(Some(app.selected_episode));
                }
            }
            SelectedList::About => {
                app.about_scroll = app.about_scroll.saturating_sub(1);
            }
            SelectedList::Credits => {
                app.credits_scroll = app.credits_scroll.saturating_sub(1);
            }
        },
        KeyCode::Down => match app.selected_list {
            SelectedList::Menu => {
                if app.selected_menu_item < 1 {
                    // Assuming 2 menu items for now
                    app.selected_menu_item += 1;
                    app.menu_list_state.select(Some(app.selected_menu_item));
                }
            }
            SelectedList::Episodes => {
                if app.selected_episode < episodes_clone.read().unwrap().len() - 1 {
                    app.selected_episode += 1;
                    app.episode_list_state.select(Some(app.selected_episode));
                }
            }
            SelectedList::About => {
                app.about_scroll = app.about_scroll.saturating_add(1);
            }
            SelectedList::Credits => {
                app.credits_scroll = app.credits_scroll.saturating_add(1);
            }
        },
        KeyCode::Enter => match app.selected_list {
            SelectedList::Episodes => {
                app.playback_state = PlaybackState::Playing;
                let episode = episodes_clone.read().unwrap()[app.selected_episode].clone();
                app.current_track = Some(episode.clone());
                let url = episode.audio_url.clone();
                app.audio_manager.tx.send(AudioCommand::Play(url)).await?;
            }
            _ => {}
        },
        KeyCode::Char(' ') => {
            match app.playback_state {
                PlaybackState::Playing => {
                    app.playback_state = PlaybackState::Paused;
                    app.audio_manager.tx.send(AudioCommand::Pause).await?;
                }
                PlaybackState::Paused => {
                    app.playback_state = PlaybackState::Playing;
                    app.audio_manager.tx.send(AudioCommand::Pause).await?;
                }
                _ => {}
            }
        }
        KeyCode::Char('s') => {
            app.playback_state = PlaybackState::Stopped;
            app.audio_manager.tx.send(AudioCommand::Stop).await?;
        }
        KeyCode::Char('+') | KeyCode::Char('=') => {
            app.volume = (app.volume + 0.1).min(1.0);
            app.audio_manager.tx.send(AudioCommand::Volume(app.volume)).await?;
        }
        KeyCode::Char('-') | KeyCode::Char('_') => {
            app.volume = (app.volume - 0.1).max(0.0);
            app.audio_manager.tx.send(AudioCommand::Volume(app.volume)).await?;
        }
        // Counter handlers
        KeyCode::Right => {}
        KeyCode::Left => {}
        // Other handlers you could add here.
        _ => {}
    }
    Ok(())
}
