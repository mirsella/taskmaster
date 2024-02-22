/* ************************************************************************** */
/*                                                                            */
/*                                                        :::      ::::::::   */
/*   mod.rs                                             :+:      :+:    :+:   */
/*                                                    +:+ +:+         +:+     */
/*   By: nguiard <nguiard@student.42.fr>            +#+  +:+       +#+        */
/*                                                +#+#+#+#+#+   +#+           */
/*   Created: 2024/02/22 10:40:09 by nguiard           #+#    #+#             */
/*   Updated: 2024/02/22 11:19:26 by nguiard          ###   ########.fr       */
/*                                                                            */
/* ************************************************************************** */

use std::{collections::HashMap, process::{self, exit}};
use libc::pid_t;

use crate::config::types::Program;

pub fn launch(prog: Program) -> HashMap<String, i32>{
	let mut pid_map: HashMap<String, pid_t> = HashMap::new();
	println!("Launching program {}", prog.name);
	
	for proccess_number in 1..prog.processes + 1 {
		match unsafe { libc::fork() } {
			-1 => {
				eprintln!("Fork failed");
			}
			0 => {
				println!("I am the child process with PID: {}", process::id());
				// Perform child process tasks here
				exit(0);
			}
			child_pid => {
				let mut identifier = prog.name.clone();
				identifier.push(':');
				identifier.push_str(&proccess_number.to_string());

				println!("Id: {identifier}");

				pid_map.insert(identifier, child_pid);
			}
		}
	}

	pid_map
}