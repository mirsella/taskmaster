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

mod parsing_conf;

use log::warn;
use parsing_conf::{get_config, Config};
use std::{env::args, error::Error, os::unix::process::CommandExt, process::Command};
use users::get_current_uid;

fn privilege_descalation(user: &str) -> Result<(), String> {
    if get_current_uid() != 0 {
        return Ok(());
    }
    warn!("This program should not be run as root. relaunching as {user}");
    let user = users::get_user_by_name(user).ok_or(format!("User {} not found", user))?;
    Err(Command::new(args().next().unwrap())
        .uid(user.uid())
        .gid(user.primary_group_id())
        .exec()
        .to_string())
}

fn main() -> Result<(), Box<dyn Error>> {
	let conf: Config = match get_config() {
		Ok(v) => v,
		Err(e) => {
			eprintln!("Error while parsing the configuration file: {:#?}", e);
			return e;
		}
	};
	dbg!(conf);

    privilege_descalation("mirsella")?;
    // privilege_descalation(conf.user);
    Ok(())
}
