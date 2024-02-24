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

use config::get_config;
use crossterm::event::{self, Event, KeyCode};
use std::{env::args, error::Error, path::Path, process::exit, time::Duration};
use tracing::{error, info};
use tracing_subscriber::EnvFilter;
use tui::{Command, Tui};

fn main() -> Result<(), Box<dyn Error>> {
    let mut tui = Tui::new()?;
    let (tracing_filter_handle, _file_guard) =
        logger::init_logger(Path::new("log.txt")).map_err(|e| format!("starting tracing: {e}"))?;
    let config_path = args().nth(1).unwrap_or("config/default.toml".to_string());
    let mut config = match get_config(&config_path) {
        Ok(v) => v,
        Err(e) => {
            error!("parsing the configuration file {config_path:?}: {e}",);
            exit(1);
        }
    };
    tracing_filter_handle
        .reload(EnvFilter::try_from_default_env().unwrap_or(config.loglevel.as_str().into()))?;

    // FIXME: should be in program.update
    for program in &mut config.program {
        // TODO: if program is set to autostart
        program.launch();
    }

    // TODO: will be in the loop of the tui
    // for program in &config.program {
    //     program.status(false);
    // }

    loop {
        tui.draw(&config.program)?;
        if event::poll(Duration::from_millis(10))? {
            if let Event::Key(key) = event::read()? {
                match key.code {
                    KeyCode::Enter => match tui.handle_enter() {
                        Some(Command::Quit) => break,
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

    for mut program in config.program {
        // TODO: shoud gracefully stop the program with the timeout or kill it
        program.kill();
    }
    Ok(())
}
