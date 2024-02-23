mod tui;

use crossterm::event::{self, Event, KeyCode};
use std::{
    error::Error,
    time::{Duration, Instant},
};
use tracing::{error, info, trace};
use tracing_subscriber::{layer::SubscriberExt, registry, util::SubscriberInitExt, EnvFilter};
use tui::Tui;
use tui_input::backend::crossterm::EventHandler;
use tui_logger::tracing_subscriber_layer;

fn main() -> Result<(), Box<dyn Error>> {
    tui_logger::set_default_level(log::LevelFilter::Trace);
    registry()
        .with(tracing_subscriber_layer())
        .with(EnvFilter::new("ui=trace"))
        .init();
    let mut tui = Tui::new()?;
    let start = Instant::now();

    let data = "brbrbrbrbrbrbrbrfoobar";
    error!(data);
    info!(data);
    trace!(data);

    while start.elapsed().as_secs() < 10 {
        tui.draw()?;
        if event::poll(Duration::from_millis(50))? {
            if let Event::Key(key) = event::read()? {
                match key.code {
                    KeyCode::Enter => {
                        info!("Enter key pressed !");
                    }
                    _ => {
                        tui.input.handle_event(&Event::Key(key));
                    }
                }
            }
        }
    }

    Ok(())
}
