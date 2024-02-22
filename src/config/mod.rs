/* ************************************************************************** */
/*                                                                            */
/*                                                        :::      ::::::::   */
/*   mod.rs                                             :+:      :+:    :+:   */
/*                                                    +:+ +:+         +:+     */
/*   By: nguiard <nguiard@student.42.fr>            +#+  +:+       +#+        */
/*                                                +#+#+#+#+#+   +#+           */
/*   Created: 2024/02/21 10:26:32 by nguiard           #+#    #+#             */
/*   Updated: 2024/02/22 17:30:46 by nguiard          ###   ########.fr       */
/*                                                                            */
/* ************************************************************************** */

pub mod signal;
pub mod types;

use std::{fs, path::Path};
use types::Config;

/// Returns the configuration found in the TOML configuration file
///
/// `Ok()` -> `parsing_conf::Config` with the configuration parsed
///
/// `Err()` -> `String` that describes the problem
pub fn get_config(file_path: impl AsRef<Path>) -> Result<Config, String> {
    let raw_file = fs::read_to_string(file_path).map_err(|e| e.to_string())?;
    let config: Config = match toml::from_str(&raw_file).map_err(|e| e.to_string()) {
		Err(e) => return Err(e),
		Ok(config) => config,
	};
	let mut used_names: Vec<String> = vec![];
	for prog in &config.program {
		for used in &used_names {
			if prog.name.eq_ignore_ascii_case(&used) {
				return Err(format!("Error: Program name {} used twice or more", used))
			}
		}
		used_names.push(prog.name.clone())
	}
	Ok(config)
}

#[cfg(test)]
mod parsing_tests {
    use super::get_config;
    use crate::config::signal::Signal;
    use crate::program::{StartPolicy, RestartPolicy};
    use std::path::Path;
    const CONFIG: &str = "config/tests.toml";

    #[test]
    fn default() {
        get_config("config/default.toml").unwrap();
    }
    #[test]
    fn tests() {
        get_config(CONFIG).unwrap();
    }
    #[test]
    fn test_user() {
        let c = get_config(CONFIG).unwrap();
        assert_eq!(c.user, Some("nonrootuser".to_string()));
    }
    #[test]
    fn test_program() {
        let c = get_config(CONFIG).unwrap();
        assert_eq!(c.program.len(), 1);
    }
    #[test]
    fn test_command() {
        let c = get_config(CONFIG).unwrap();
        assert_eq!(c.program[0].command, "ls".to_string());
    }
    #[test]
    fn test_start() {
        let c = get_config(CONFIG).unwrap();
        assert_eq!(c.program[0].start_policy, StartPolicy::Manual);
    }
    #[test]
    fn test_exit_signals() {
        let c = get_config(Path::new(CONFIG)).unwrap();
        assert_eq!(c.program[0].stop_signal, Signal::SIGKILL);

    }
    #[test]
    fn test_restart_policy() {
        let c = get_config(CONFIG).unwrap();
        assert_eq!(c.program[0].restart_policy, RestartPolicy::Never);
    }

    #[test]
    #[should_panic]
    fn invalid_config() {
        get_config("invalid.toml").unwrap();
    }
}
