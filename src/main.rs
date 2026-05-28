use petalblade::app::{App, music_for_programming};
use petalblade::event::{Event, EventHandler};
use petalblade::handler::handle_key_events;
use petalblade::tui::Tui;
use ratatui::Terminal;
use ratatui::backend::CrosstermBackend;
use std::error::Error;
use std::io;
use std::sync::{
    Arc, RwLock,
};
#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let mut app = App::new();

    let mfp_episodes = music_for_programming().await.unwrap();
    app.episodes = Arc::new(RwLock::new(mfp_episodes));
    let backend = CrosstermBackend::new(io::stderr());
    let terminal = Terminal::new(backend)?;
    let events = EventHandler::new(250);
    let mut tui = Tui::new(terminal, events);
    tui.init()?;
    while app.running {
        tui.draw(&mut app)?;
        match tui.events.next()? {
            Event::Key(key_event) => handle_key_events(key_event, &mut app).await?,
            Event::Mouse(_) => {}
            Event::Resize(_, _) => {}
        }
    }

    tui.exit()?;
    Ok(())
}
