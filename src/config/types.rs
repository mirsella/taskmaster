/* ************************************************************************** */
/*                                                                            */
/*                                                        :::      ::::::::   */
/*   types.rs                                           :+:      :+:    :+:   */
/*                                                    +:+ +:+         +:+     */
/*   By: nguiard <nguiard@student.42.fr>            +#+  +:+       +#+        */
/*                                                +#+#+#+#+#+   +#+           */
/*   Created: 2024/02/21 13:39:16 by nguiard           #+#    #+#             */
/*   Updated: 2024/02/22 17:24:28 by nguiard          ###   ########.fr       */
/*                                                                            */
/* ************************************************************************** */

use log::LevelFilter;
use serde::Deserialize;
use crate::program::Program;

#[serde_as]
#[derive(Deserialize, Debug)]
pub struct Config {
    pub user: Option<String>,
    #[serde(default = "default_logfile")]
    pub logfile: String,
    #[serde(default = "default_loglevel")]
    #[serde_as(as = "DisplayFromStr")]
    pub loglevel: Level,
    pub program: Vec<Program>,
}
fn default_logfile() -> String {
    "taskmaster.log".to_string()
}
fn default_loglevel() -> Level {
    Level::INFO
}
