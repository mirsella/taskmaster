/* ************************************************************************** */
/*                                                                            */
/*                                                        :::      ::::::::   */
/*   types.rs                                           :+:      :+:    :+:   */
/*                                                    +:+ +:+         +:+     */
/*   By: nguiard <nguiard@student.42.fr>            +#+  +:+       +#+        */
/*                                                +#+#+#+#+#+   +#+           */
/*   Created: 2024/02/21 13:39:16 by nguiard           #+#    #+#             */
/*   Updated: 2024/02/22 11:34:38 by nguiard          ###   ########.fr       */
/*                                                                            */
/* ************************************************************************** */

use serde::Deserialize;
use crate::program::Program;

#[derive(Deserialize, Debug)]
pub struct Config {
    pub user: Option<String>,
	pub logfile: Option<String>,
	pub loglevel: Option<String>,
    pub program: Vec<Program>,
}
