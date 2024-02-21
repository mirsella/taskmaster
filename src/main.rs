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

use log::{debug, error, warn};
use std::{env::args, error::Error, os::unix::process::CommandExt, process::Command};
use users::get_current_uid;

use crate::config::get_config;

fn privilege_descalation(user: &str) -> Result<(), String> {
    if get_current_uid() != 0 {
        debug!(
            "running as non-privilaged user {}",
            users::get_current_username()
                .unwrap_or("deleted".into())
                .to_string_lossy()
        );
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
    // TODO: start syslog and simple_logger
    let conf = match get_config() {
        Ok(v) => v,
        Err(e) => {
            error!("parsing the configuration file: {:#?}", e);
            return Err(e.into());
        }
    };
    if let Err(e) = privilege_descalation(&conf.user) {
        error!("descalating privileges: {:#?}", e);
        return Err(e.into());
    }
    Ok(())
}
