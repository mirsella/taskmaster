/* ************************************************************************** */
/*                                                                            */
/*                                                        :::      ::::::::   */
/*   main.rs                                            :+:      :+:    :+:   */
/*                                                    +:+ +:+         +:+     */
/*   By: nguiard <nguiard@student.42.fr>            +#+  +:+       +#+        */
/*                                                +#+#+#+#+#+   +#+           */
/*   Created: 2024/02/21 10:09:10 by nguiard           #+#    #+#             */
/*   Updated: 2024/02/22 18:33:44 by nguiard          ###   ########.fr       */
/*                                                                            */
/* ************************************************************************** */

mod config;
mod logger;
mod program;

use config::{get_config, types::Config};
use std::{env::args, error::Error, process::exit};
use tracing::{error, info, Level};
use log::{debug, error, info, warn};
use std::{env::args, error::Error, os::unix::process::CommandExt, path::Path,
      process::exit, process::Command, time::Duration};
use users::get_current_uid;

fn _privilege_descalation(name: Option<&str>) -> Result<(), String> {
    if get_current_uid() != 0 {
        debug!(
            "running as non-privilaged user {}",
            users::get_current_username()
                .unwrap_or("deleted".into())
                .to_string_lossy()
        );
        return Ok(());
    }
    warn!("This program should not be run as root");
    let Some(name) = name else {
        return Err("Please specify a user to relaunch into".into());
    };
    let user =
        users::get_user_by_name(name).ok_or(format!("User {} not found on the system", name))?;
    info!("Relaunching as {}", name);
    Err(Command::new(args().next().unwrap())
        .uid(user.uid())
        .gid(user.primary_group_id())
        .exec()
        .to_string())
}

fn main() -> Result<(), Box<dyn Error>> {
    let config_path = args().nth(1).unwrap_or("config/default.toml".to_string());
    let config = match get_config(&config_path) {
        Ok(v) => v,
        Err(e) => {
            let _ = logger::init_logger("log.txt", &Level::INFO)?;
            error!("Error while parsing the configuration file {config_path:?}: {e}",);
            exit(1);
        }
    };
    let _file_guard = logger::init_logger(&config.logfile, &config.loglevel)?;
    info!("Starting...");
    Ok(())
    if let Err(e) = _privilege_descalation(conf.user.as_deref()) {
        error!("de-escalating privileges: {:#?}", e);
        return;
    };

	for program in &mut conf.program {
		program.launch();
	}

	std::thread::sleep(Duration::from_secs(1));

	for program in &conf.program {
		println!("---");
		program.status(false);
	}
	println!("---");

	for mut program in conf.program {
		program.kill();
	}
}
