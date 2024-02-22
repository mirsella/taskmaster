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

use config::get_config;
use std::{env::args, process::exit, time::Duration};
use tracing::{error, info, Level};

fn main() {
    let config_path = args().nth(1).unwrap_or("config/default.toml".to_string());
    let mut config = match get_config(&config_path) {
        Ok(v) => v,
        Err(e) => {
            let _ = logger::init_logger("log.txt", &Level::INFO).unwrap();
            error!("Error while parsing the configuration file {config_path:?}: {e}",);
            exit(1);
        }
    };
    let _file_guard = logger::init_logger(&config.logfile, &config.loglevel).unwrap();
    info!("Starting...");

    for program in &mut config.program {
        program.launch();
    }

    std::thread::sleep(Duration::from_secs(1));

    for program in &config.program {
        println!("---");
        program.status(false);
    }
    println!("---");

    for mut program in config.program {
        program.kill();
    }
}
