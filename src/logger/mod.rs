/* ************************************************************************** */
/*                                                                            */
/*                                                        :::      ::::::::   */
/*   mod.rs                                             :+:      :+:    :+:   */
/*                                                    +:+ +:+         +:+     */
/*   By: nguiard <nguiard@student.42.fr>            +#+  +:+       +#+        */
/*                                                +#+#+#+#+#+   +#+           */
/*   Created: 2024/02/21 16:53:03 by nguiard           #+#    #+#             */
/*   Updated: 2024/02/22 09:49:47 by nguiard          ###   ########.fr       */
/*                                                                            */
/* ************************************************************************** */

use log::{debug, error, info, LevelFilter};
use simplelog::*;
use std::fs::File;

pub fn init_logger(log_file: &str, log_level: &LevelFilter) {
    CombinedLogger::init(vec![
        TermLogger::new(
            *log_level,
            simplelog::Config::default(),
            TerminalMode::Mixed,
            ColorChoice::Always,
        ),
        WriteLogger::new(
            *log_level,
            simplelog::Config::default(),
            File::create(log_file).unwrap(),
        ),
    ])
    .unwrap();

    error!("Bright red error");
    info!("This only appears in the log file");
    debug!("This is debugggggggggg ohhhh a buggggggggg");
}
