/* ************************************************************************** */
/*                                                                            */
/*                                                        :::      ::::::::   */
/*   main.rs                                            :+:      :+:    :+:   */
/*                                                    +:+ +:+         +:+     */
/*   By: nguiard <nguiard@student.42.fr>            +#+  +:+       +#+        */
/*                                                +#+#+#+#+#+   +#+           */
/*   Created: 2024/02/21 10:09:10 by nguiard           #+#    #+#             */
/*   Updated: 2024/02/21 15:59:57 by nguiard          ###   ########.fr       */
/*                                                                            */
/* ************************************************************************** */

mod config;
use std::env;
use std::path::Path;

use config::get_config;
use config::data_type::Config;

fn main() {
	let location: Vec<String> = env::args().collect();

	let conf_file: &Path = match location.len() {
		1 => Path::new("taskmaster.toml"),
		_ => Path::new(&location[1])
	};

	let conf: Config = match get_config(conf_file) {
		Ok(v) => v,
		Err(e) => {
			eprintln!("Error while parsing the configuration file {:?}: {:#?}",
						conf_file, e);
			return ;
		}
	};

	dbg!(conf);
}
