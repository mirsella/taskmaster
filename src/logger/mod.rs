/* ************************************************************************** */
/*                                                                            */
/*                                                        :::      ::::::::   */
/*   mod.rs                                             :+:      :+:    :+:   */
/*                                                    +:+ +:+         +:+     */
/*   By: nguiard <nguiard@student.42.fr>            +#+  +:+       +#+        */
/*                                                +#+#+#+#+#+   +#+           */
/*   Created: 2024/02/21 16:53:03 by nguiard           #+#    #+#             */
/*   Updated: 2024/02/21 17:30:07 by nguiard          ###   ########.fr       */
/*                                                                            */
/* ************************************************************************** */

use std::path::Path;
use simple_logging;

use crate::config::data_type::Config;
use log::LevelFilter;

pub fn init_logger(config: Config) {
	let level: LevelFilter = match config.loglevel {
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
	
	let file_logger = match config.logfile {
		None => None,
		Some(path_string) => {
			let path = Path::new(&path_string);
			Some(simple_logging::log_to_file(path, level))
		}
	};

	dbg!(file_logger);
}