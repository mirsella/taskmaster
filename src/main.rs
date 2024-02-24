/* ************************************************************************** */
/*                                                                            */
/*                                                        :::      ::::::::   */
/*   main.rs                                            :+:      :+:    :+:   */
/*                                                    +:+ +:+         +:+     */
/*   By: nguiard <nguiard@student.42.fr>            +#+  +:+       +#+        */
/*                                                +#+#+#+#+#+   +#+           */
/*   Created: 2024/02/21 10:09:10 by nguiard           #+#    #+#             */
/*   Updated: 2024/02/22 19:23:54 by nguiard          ###   ########.fr       */
/*                                                                            */
/* ************************************************************************** */

mod config;
mod logger;
mod program;
mod tui;

use config::Config;
use crossterm::event::{self, Event, KeyCode};
use std::{env::args, error::Error, process::exit, time::Duration};
use tracing::{error, info};
use tui::{Command, Tui};

fn main() -> Result<(), Box<dyn Error>> {
    let mut tui = Tui::new()?;
    let tracing_filter_handle =
        logger::init_logger("taskmaster.log").map_err(|e| format!("starting tracing: {e}"))?;
    let config_path = args().nth(1).unwrap_or("config/default.toml".to_string());
    let mut config = match Config::load(&config_path) {
        Ok(v) => v,
        Err(e) => {
            error!("parsing the configuration file {config_path:?}: {e}",);
            exit(1);
        }
    };
    config.tracing_filter_handle = Some(tracing_filter_handle);
    config.reload_tracing_level()?;

    // FIXME: should be in program.update
    for program in &mut config.program {
        // TODO: only if program is set to autostart
        program.launch();
    }

    loop {
        info!("Drawing TUI");
        error!("Drawing TUI");
        tui.draw(&config.program)?;
        if event::poll(Duration::from_millis(10))? {
            if let Event::Key(key) = event::read()? {
                match key.code {
                    KeyCode::Enter => match tui.handle_enter() {
                        Some(Command::Quit) => break,
                        Some(Command::LogLevel(level)) => {
                            info!(?level, "Changing log level");
                            config.loglevel = level;
                            config.reload_tracing_level()?;
                        }
                        Some(cmd) => info!(?cmd, "Command entered"),
                        _ => (),
                    },
                    KeyCode::Up => tui.history_up(),
                    KeyCode::Down => tui.history_down(),
                    _ => tui.handle_other_event(&Event::Key(key)),
                }
            }
        }
    }
    info!("Exiting, gracefully stopping programs");
    for mut program in config.program {
        // TODO: should gracefully stop the program with the timeout or kill it
        program.kill();
    }
    Ok(())
}
