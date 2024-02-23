/* ************************************************************************** */
/*                                                                            */
/*                                                        :::      ::::::::   */
/*   main.rs                                            :+:      :+:    :+:   */
/*                                                    +:+ +:+         +:+     */
/*   By: nguiard <nguiard@student.42.fr>            +#+  +:+       +#+        */
/*                                                +#+#+#+#+#+   +#+           */
/*   Created: 2024/02/21 10:09:10 by nguiard           #+#    #+#             */
/*   Updated: 2024/02/23 18:20:58 by nguiard          ###   ########.fr       */
/*                                                                            */
/* ************************************************************************** */

mod config;
mod logger;
mod program;

use config::get_config;
use std::{env::args, path::Path, process::exit, time::Duration};
use tracing::{error, info, Level};

fn main() {
    let config_path = args().nth(1).unwrap_or("config/default.toml".to_string());
    let mut config = match get_config(&config_path) {
        Ok(v) => v,
        Err(e) => {
            let _ = logger::init_logger(Path::new("log.txt"), &Level::INFO).unwrap();
            error!("Error while parsing the configuration file {config_path:?}: {e}",);
            exit(1);
        }
    };
    let _file_guard = logger::init_logger(&config.logfile, &config.loglevel).unwrap();
    info!("Starting...");

    for program in &mut config.program {
        program.launch();
    }

	for program in &config.program {
        println!("---");
        program.status(false);
    }
    println!("---");

    std::thread::sleep(Duration::from_millis(500));

	for program in &mut config.program {
        program.kill();
    }
	std::thread::sleep(Duration::from_millis(50));
	for program in &mut config.program {
        program.update();
    }
	
	std::thread::sleep(Duration::from_millis(500));
	for program in &mut config.program {
        program.update();
    }

    for program in &config.program {
        println!("---");
        program.status(false);
    }
    println!("---");

    for mut program in config.program {
        program.kill();
    }
}
