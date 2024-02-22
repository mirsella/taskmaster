/* ************************************************************************** */
/*                                                                            */
/*                                                        :::      ::::::::   */
/*   mod.rs                                             :+:      :+:    :+:   */
/*                                                    +:+ +:+         +:+     */
/*   By: nguiard <nguiard@student.42.fr>            +#+  +:+       +#+        */
/*                                                +#+#+#+#+#+   +#+           */
/*   Created: 2024/02/21 10:26:32 by nguiard           #+#    #+#             */
/*   Updated: 2024/02/22 19:24:19 by nguiard          ###   ########.fr       */
/*                                                                            */
/* ************************************************************************** */

pub mod signal;

use crate::program::{generate_name, Program};
use serde::Deserialize;
use serde_with::{serde_as, DisplayFromStr};
pub use signal::Signal;
use std::{collections::HashSet, error::Error, fs, mem, path::Path};
use tracing::{warn, Level};

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

pub fn get_config(file_path: impl AsRef<Path>) -> Result<Config, Box<dyn Error>> {
    let raw_file = fs::read_to_string(file_path)?;
    let mut config: Config = toml::from_str(&raw_file)?;
    let mut names = HashSet::new();
    for prog in &mut config.program {
        if names.insert(prog.name.clone()) {
            continue;
        }
        let new = generate_name();
        warn!(
            "Renaming Program with command `{}` as `{}` because `{}` is already taken",
            prog.command, new, prog.name
        );
        prog.name = new;
    }
    Ok(config)
}

#[cfg(test)]
mod parsing_tests {
    use super::{get_config, Signal};
    use crate::program::{RestartPolicy, StartPolicy};
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
        assert_eq!(&c.program[0].command, "ls");
    }
    #[test]
    fn test_start() {
        let c = get_config(CONFIG).unwrap();
        assert_eq!(c.program[0].start_policy, StartPolicy::Manual);
    }
    #[test]
    fn test_exit_codes() {
        let c = get_config(Path::new(CONFIG)).unwrap();
        assert_eq!(c.program[0].valid_exit_codes, vec![0])
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
    fn test_umask() {
        let c = get_config(CONFIG).unwrap();
        assert_eq!(c.program[0].umask.unwrap(), 0o002);
    }
    #[test]
    fn test_random_name() {
        let c = get_config(CONFIG).unwrap();
        assert!(!c.program[0].name.is_empty());
    }

    #[test]
    #[should_panic]
    fn invalid_config() {
        get_config("config/invalid.toml").unwrap();
    }
    #[test]
    fn invalid_config_no_command() {
        dbg!(get_config("config/no_command.toml"))
            .unwrap_err()
            .to_string()
            .contains("missing field `command`")
            .then_some(())
            .unwrap();
    }
}
