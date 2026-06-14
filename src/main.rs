use petalblade::app::{App, AudioCommand, music_for_programming};
use petalblade::event::{Event, EventHandler};
use petalblade::handler::handle_key_events;
use petalblade::tui::Tui;
use ratatui::Terminal;
use ratatui::backend::CrosstermBackend;
use std::error::Error;
use std::io;
use std::sync::{Arc, RwLock};
#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let mut app = App::new();

    match music_for_programming().await {
        Ok(mfp_episodes) => {
            app.episodes = Arc::new(RwLock::new(mfp_episodes));
        }
        Err(e) => {
            app.status_message = format!("Network Error: Check connection. ({})", e);
        }
    }

    if let Err(e) = app.load_user_config() {
        app.status_message = format!("Config Error: using defaults. ({})", e);
    }
    let _ = app
        .audio_manager
        .tx
        .send(AudioCommand::Volume(app.volume))
        .await;

    let backend = CrosstermBackend::new(io::stdout());
    let terminal = Terminal::new(backend)?;
    let events = EventHandler::new(250);
    let mut tui = Tui::new(terminal, events);
    tui.init()?;
    while app.running {
        tui.draw(&mut app)?;
        match tui.events.next()? {
            Event::Key(key_event) => {
                handle_key_events(key_event, &mut app).await?;
                app.drain_audio_status();
            }
            Event::Mouse(_) => {}
            Event::Resize(_, _) => {}
            Event::Tick => app.drain_audio_status(),
        }
    }

    let save_result = app.save_user_config();
    let shutdown_result = app.shutdown_audio().await;
    tui.exit()?;
    save_result?;
    shutdown_result?;
    Ok(())
}
