mod tui;

use crossterm::event::{self, Event, KeyCode};
use std::{
    error::Error,
    time::{Duration, Instant},
};
use tracing::{error, info, trace};
use tracing_subscriber::{layer::SubscriberExt, registry, util::SubscriberInitExt, EnvFilter};
use tui::Tui;
use tui_logger::tracing_subscriber_layer;

use crate::tui::Command;

fn main() -> Result<(), Box<dyn Error>> {
    tui_logger::set_default_level(log::LevelFilter::Trace);
    registry()
        .with(tracing_subscriber_layer())
        .with(EnvFilter::new("ui=trace"))
        .init();
    let mut tui = Tui::new()?;

    let data = "brbrbrbrbrbrbrbrfoobar";
    error!(data);
    info!(data);
    trace!(data);

    loop {
        tui.draw()?;
        if event::poll(Duration::from_millis(10))? {
            if let Event::Key(key) = event::read()? {
                match key.code {
                    KeyCode::Enter => match tui.handle_enter() {
                        Some(Command::Quit) => break,
                        Some(cmd) => info!(?cmd, "Command entered"),
                        _ => (),
                    },
                    KeyCode::Up => {
                        tui.history_up();
                    }
                    KeyCode::Down => {
                        tui.history_down();
                    }
                    _ => {
                        tui.handle_other_event(&Event::Key(key));
                    }
                }
            }
        }
    }

    Ok(())
}
