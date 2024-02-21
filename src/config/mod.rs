/* ************************************************************************** */
/*                                                                            */
/*                                                        :::      ::::::::   */
/*   mod.rs                                             :+:      :+:    :+:   */
/*                                                    +:+ +:+         +:+     */
/*   By: nguiard <nguiard@student.42.fr>            +#+  +:+       +#+        */
/*                                                +#+#+#+#+#+   +#+           */
/*   Created: 2024/02/21 10:26:32 by nguiard           #+#    #+#             */
/*   Updated: 2024/02/21 16:52:40 by nguiard          ###   ########.fr       */
/*                                                                            */
/* ************************************************************************** */

pub mod data_type;
pub mod signal;

use data_type::Config;
use std::{fs, path::PathBuf};

/// Returns the configuration found in the TOML configuration file
///
/// `Ok()` -> `parsing_conf::Config` with the configuration parsed
///
/// `Err()` -> `String` that describes the problem
pub fn get_config(file_path: PathBuf) -> Result<Config, String> {
    let raw_file = match fs::read_to_string(file_path) {
        Ok(v) => v,
        Err(e) => return Err(e.to_string()),
    };

    let conf = match toml::from_str(&raw_file) {
        Ok(v) => v,
        Err(e) => return Err(e.to_string()),
    };

    Ok(conf)
}

#[cfg(test)]
mod parsing_tests {
    use super::get_config;

    #[test]
    fn basic_config() {
        get_config("taskmaster.toml".into()).unwrap();
    }
}
