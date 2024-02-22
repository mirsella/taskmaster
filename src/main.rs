/* ************************************************************************** */
/*                                                                            */
/*                                                        :::      ::::::::   */
/*   main.rs                                            :+:      :+:    :+:   */
/*                                                    +:+ +:+         +:+     */
/*   By: nguiard <nguiard@student.42.fr>            +#+  +:+       +#+        */
/*                                                +#+#+#+#+#+   +#+           */
/*   Created: 2024/02/21 10:09:10 by nguiard           #+#    #+#             */
/*   Updated: 2024/02/21 22:28:18 by nguiard          ###   ########.fr       */
/*                                                                            */
/* ************************************************************************** */

mod config;
mod logger;

use config::get_config;
use std::{env::args, error::Error, process::exit};
use tracing::{error, info, Level};

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
    // while the guard is in scope, the file is open
    let _file_guard = logger::init_logger(&config.logfile, &config.loglevel)?;
    info!("Starting...");
    Ok(())
}
