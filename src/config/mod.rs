/* ************************************************************************** */
/*                                                                            */
/*                                                        :::      ::::::::   */
/*   mod.rs                                             :+:      :+:    :+:   */
/*                                                    +:+ +:+         +:+     */
/*   By: nguiard <nguiard@student.42.fr>            +#+  +:+       +#+        */
/*                                                +#+#+#+#+#+   +#+           */
/*   Created: 2024/02/21 10:26:32 by nguiard           #+#    #+#             */
/*   Updated: 2024/02/21 16:03:34 by nguiard          ###   ########.fr       */
/*                                                                            */
/* ************************************************************************** */

pub mod data_type;
pub mod signal;

use std::{fs, path::Path};
use data_type::Config;

/// Returns the configuration found in the TOML configuration file
///
/// `Ok()` -> `parsing_conf::Config` with the configuration parsed
///
/// `Err()` -> `String` that describes the problem
pub fn get_config(file_path: &Path) -> Result<Config, String> {	
	let raw_file = match fs::read_to_string(file_path) {
		Ok(v) => v,
		Err(e) => return Err(e.to_string()),
	};

	let conf = match toml::from_str(&raw_file) {
		Ok(v) => v,
		Err(e) => return Err(e.to_string())
	};

	Ok(conf)
}

#[cfg(test)]
mod parsing_tests {
    use std::path::Path;

    use super::get_config;

	#[test]
	fn basic_config() {
		match get_config(Path::new("taskmaster.toml")) {
			Ok(var) => println!("No problem"),
			Err(e) => panic!()
		}
	}
}