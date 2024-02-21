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

use data_type::Config;
use std::{fs, path::Path};

/// Returns the configuration found in the TOML configuration file
///
/// `Ok()` -> `parsing_conf::Config` with the configuration parsed
///
/// `Err()` -> `String` that describes the problem
pub fn get_config(file_path: &Path) -> Result<Config, String> {
    let raw_file = fs::read_to_string(file_path).map_err(|e| e.to_string())?;
    toml::from_str(&raw_file).map_err(|e| e.to_string())
}

#[cfg(test)]
mod parsing_tests {
    use super::get_config;
    use crate::config::signal::Signal;
    use std::path::Path;
    const CONFIG: &str = "config/default.toml";

    #[test]
    fn basic_config() {
        get_config(Path::new(CONFIG)).unwrap();
    }
    #[test]
    fn test_user() {
        let c = get_config(Path::new(CONFIG)).unwrap();
        assert_eq!(c.user, Some("nonrootuser".to_string()));
    }
    #[test]
    fn test_program() {
        let c = get_config(Path::new(CONFIG)).unwrap();
        assert_eq!(c.program.len(), 1);
    }
    #[test]
    fn test_command() {
        let c = get_config(Path::new(CONFIG)).unwrap();
        assert_eq!(c.program[0].command, "ls".to_string());
    }
    #[test]
    fn test_exit_signals() {
        let c = get_config(Path::new(CONFIG)).unwrap();
        assert_eq!(c.program[0].exit_signals, vec![Signal::SIGKILL]);
    }

    #[test]
    #[should_panic]
    fn invalid_config() {
        get_config(Path::new("invalid.toml")).unwrap();
    }
}
