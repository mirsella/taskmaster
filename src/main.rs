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
use program::Process;
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

    for program in &mut config.program {
        program.start();
    }

    let mut pending_quit = false;
    loop {
        if pending_quit
            && config.program.iter().all(|p| {
                p.childs
                    .iter()
                    .all(|c| matches!(c.process, Process::NotRunning(_)))
            })
        {
            info!("All programs have stopped. Quitting");
            break;
        }
        tui.draw(&config.program)?;
        if event::poll(Duration::from_millis(10))? {
            if let Event::Key(key) = event::read()? {
                match key.code {
                    KeyCode::Enter => match tui.handle_enter() {
                        Some(Command::Quit) => {
                            info!("Gracefully shutting down programs");
                            pending_quit = true;
                            for program in &mut config.program {
                                program.stop();
                            }
                        }
                        Some(Command::LogLevel(level)) => {
                            info!(?level, "Changing log level");
                            config.loglevel = level;
                            config.reload_tracing_level()?;
                        }
                        Some(Command::Reload(mut path)) => {
                            if path.is_empty() {
                                path = config_path.clone()
                            }
                            info!(path, "Reloading configuration");
                            match Config::load(&path) {
                                Ok(new_config) => config.update(new_config)?,
                                Err(e) => {
                                    error!(path, error = e, "reloading the configuration file")
                                }
                            }
                        }
                        Some(Command::Start(name)) => {
                            if name.is_empty() {
                                info!("Starting all programs");
                                for program in &mut config.program {
                                    program.start();
                                }
                            } else if let Some(p) =
                                config.program.iter_mut().find(|p| p.name == name)
                            {
                                info!(name, "Starting program");
                                p.start()
                            } else {
                                error!(name, "Program not found");
                            }
                        }
                        Some(Command::Stop(name)) => {
                            if name.is_empty() {
                                info!(name, "Stopping all programs");
                                for program in &mut config.program {
                                    program.start();
                                }
                            } else if let Some(p) =
                                config.program.iter_mut().find(|p| p.name == name)
                            {
                                info!(name, "Stopping program");
                                p.start()
                            } else {
                                error!(name, "Program not found");
                            }
                        }
                        None => (),
                    },
                    KeyCode::Up => tui.history_up(),
                    KeyCode::Down => tui.history_down(),
                    _ => tui.handle_other_event(&Event::Key(key)),
                }
            }
        }
    }
    Ok(())
}
