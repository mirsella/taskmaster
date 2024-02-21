/* ************************************************************************** */
/*                                                                            */
/*                                                        :::      ::::::::   */
/*   main.rs                                            :+:      :+:    :+:   */
/*                                                    +:+ +:+         +:+     */
/*   By: nguiard <nguiard@student.42.fr>            +#+  +:+       +#+        */
/*                                                +#+#+#+#+#+   +#+           */
/*   Created: 2024/02/21 10:09:10 by nguiard           #+#    #+#             */
/*   Updated: 2024/02/21 14:57:25 by nguiard          ###   ########.fr       */
/*                                                                            */
/* ************************************************************************** */

mod config;
use config::get_config;
use config::data_type::Config;

fn main() {
	let conf: Config = match get_config() {
		Ok(v) => v,
		Err(e) => {
			eprintln!("Error while parsing the configuration file: {:#?}", e);
			return ;
		}
	};

	dbg!(conf);
}
