/* ************************************************************************** */
/*                                                                            */
/*                                                        :::      ::::::::   */
/*   parsing_conf.rs                                    :+:      :+:    :+:   */
/*                                                    +:+ +:+         +:+     */
/*   By: nguiard <nguiard@student.42.fr>            +#+  +:+       +#+        */
/*                                                +#+#+#+#+#+   +#+           */
/*   Created: 2024/02/21 10:26:32 by nguiard           #+#    #+#             */
/*   Updated: 2024/02/21 13:13:12 by nguiard          ###   ########.fr       */
/*                                                                            */
/* ************************************************************************** */

use std::fs;

#[derive(serde::Deserialize, Debug)]
struct Global {
	userid: u32,
}

#[derive(serde::Deserialize, Debug)]
struct Program {
	command: String,
	processes: u8,
	exit_codes: Vec<u8>,
}

#[derive(serde::Deserialize, Debug)]
pub struct Config {
	global: Global,
	program: Vec<Program>,
}

pub fn get_config() -> Result<Config, String> {
	let conf: Config;

	let raw_file = fs::read_to_string("config/default.toml").unwrap();

	conf = toml::from_str(&raw_file).unwrap();

	println!("{:#?}", conf);

	return Ok(conf);
}

#[cfg(test)]
mod parsing_tests {
    use super::get_config;

	#[test]
	fn basic_config() {
		match get_config() {
			Ok(var) => println!("No problem"),
			Err(e) => panic!()
		}
	}
}