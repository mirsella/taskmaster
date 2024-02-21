/* ************************************************************************** */
/*                                                                            */
/*                                                        :::      ::::::::   */
/*   mod.rs                                             :+:      :+:    :+:   */
/*                                                    +:+ +:+         +:+     */
/*   By: nguiard <nguiard@student.42.fr>            +#+  +:+       +#+        */
/*                                                +#+#+#+#+#+   +#+           */
/*   Created: 2024/02/21 16:53:03 by nguiard           #+#    #+#             */
/*   Updated: 2024/02/21 23:03:08 by nguiard          ###   ########.fr       */
/*                                                                            */
/* ************************************************************************** */

use std::fs::File;

use crate::config::data_type::Config;
use log::{info, error, debug, LevelFilter};
use simplelog::*;

pub fn init_logger(config: &Config) {
	let level: LevelFilter = match &config.loglevel {
		Some(lvl) => match lvl.to_lowercase().as_str() {
			"off" => LevelFilter::Off,
			"error" => LevelFilter::Error,
			"info" => LevelFilter::Info,
			"debug" => LevelFilter::Debug,
			"trace" => LevelFilter::Trace,
			_ => LevelFilter::Warn,
		}
		_ => LevelFilter::Warn,
	};
	let file_path: String= match &config.logfile {
		Some(s) => s.into(),
		None => "/dev/null".into(),
	};
	
	let test = CombinedLogger::init(vec![
        TermLogger::new(
            level,
            simplelog::Config::default(),
            TerminalMode::Mixed,
            ColorChoice::Always,
        ),
        WriteLogger::new(
            level,
            simplelog::Config::default(),
            File::create(file_path).unwrap(),
        ),
    ]).unwrap();

	dbg!(test);

	error!("Bright red error");
    info!("This only appears in the log file");
    debug!("This level is currently not enabled for any logger");
}
