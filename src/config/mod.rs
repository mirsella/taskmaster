/* ************************************************************************** */
/*                                                                            */
/*                                                        :::      ::::::::   */
/*   mod.rs                                             :+:      :+:    :+:   */
/*                                                    +:+ +:+         +:+     */
/*   By: nguiard <nguiard@student.42.fr>            +#+  +:+       +#+        */
/*                                                +#+#+#+#+#+   +#+           */
/*   Created: 2024/02/21 10:26:32 by nguiard           #+#    #+#             */
/*   Updated: 2024/02/27 10:39:05 by nguiard          ###   ########.fr       */
/*                                                                            */
/* ************************************************************************** */

pub mod signal;

use crate::program::{generate_name, Program};
use serde::Deserialize;
use serde_with::{serde_as, DisplayFromStr};
pub use signal::Signal;
use std::{collections::HashSet, error::Error, fs, path::Path};
use tracing::{error, info, instrument, warn, Level};
use tracing_subscriber::{reload::Handle, EnvFilter, Registry};

#[serde_as]
#[derive(Deserialize, Debug)]
pub struct Config {
    #[serde(default = "default_loglevel")]
    #[serde_as(as = "DisplayFromStr")]
    pub loglevel: Level,
    pub program: Vec<Program>,

    #[serde(skip)]
    pub tracing_filter_handle: Option<Handle<EnvFilter, Registry>>,
}
fn default_loglevel() -> Level {
    Level::INFO
}

impl Config {
    pub fn reload_tracing_level(&mut self) -> Result<(), Box<dyn Error>> {
        self.tracing_filter_handle
            .as_ref()
            .ok_or("tracing not initialized")?
            .reload(EnvFilter::new(self.loglevel.as_str()))?;
        Ok(())
    }

    #[instrument(skip_all, fields(path = %file_path.as_ref().display()))]
    pub fn load(file_path: impl AsRef<Path>) -> Result<Config, Box<dyn Error>> {
        info!("Loading configuration file");
        let raw_file = fs::read_to_string(file_path)?;
        let mut config: Config = toml::from_str(&raw_file)?;
        let mut names = HashSet::new();
        for prog in &mut config.program {
            prog.name = prog
                .name
                .replace(" ", "_")
                .trim_matches(['_', ' '])
                .to_string();
            if !prog.name.is_empty() && names.insert(prog.name.clone()) {
                continue;
            }
            // there is 1124 power of 981 combinaisons, we are safe
            let new = generate_name();
            warn!(
                "Renaming Program with command `{}` as `{}` because `{}` is already taken",
                prog.cmd.display(),
                new,
                prog.name
            );
            prog.name = new;
        }
        info!(
            "Configuration file loaded with {} programs",
            config.program.len()
        );
        Ok(config)
    }
    pub fn update(&mut self, new: Config) -> Result<(), Box<dyn Error>> {
        if self.loglevel != new.loglevel {
            self.loglevel = new.loglevel;
            self.reload_tracing_level()?;
        }
        for program in &mut self.program {
            if !new.program.iter().any(|p| p.name == program.name) {
                program.stop();
            }
        }
        for mut new in new.program.into_iter() {
            if let Some(old) = self.program.iter_mut().find(|p| p.name == new.name) {
                old.update(new);
            } else {
                if let Err(e) = new.start() {
                    error!(error = e, "starting program");
                }
                self.program.push(new);
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::{Config, Signal};
    use crate::program::{RestartPolicy, StartPolicy};
    use std::path::Path;
    const CONFIG: &str = "config/tests.toml";

    #[test]
    fn default() {
        Config::load("config/default.toml").unwrap();
    }
    #[test]
    fn tests() {
        Config::load(CONFIG).unwrap();
    }
    #[test]
    fn test_program() {
        let c = Config::load(CONFIG).unwrap();
        assert_eq!(c.program.len(), 1);
    }
    #[test]
    fn test_command() {
        let c = Config::load(CONFIG).unwrap();
        assert_eq!(c.program[0].cmd.display().to_string(), "ls");
    }
    #[test]
    fn test_start() {
        let c = Config::load(CONFIG).unwrap();
        assert_eq!(c.program[0].start_policy, StartPolicy::Manual);
    }
    #[test]
    fn test_exit_codes() {
        let c = Config::load(Path::new(CONFIG)).unwrap();
        assert_eq!(c.program[0].valid_exit_codes, vec![0])
    }
    #[test]
    fn test_exit_signals() {
        let c = Config::load(Path::new(CONFIG)).unwrap();
        assert_eq!(c.program[0].stop_signal, Signal::SIGKILL);
    }
    #[test]
    fn test_restart_policy() {
        let c = Config::load(CONFIG).unwrap();
        assert_eq!(c.program[0].restart_policy, RestartPolicy::Never);
    }
    #[test]
    fn test_umask() {
        let c = Config::load(CONFIG).unwrap();
        assert_eq!(c.program[0].umask.unwrap(), 0o002);
    }
    #[test]
    fn test_random_name() {
        let c = Config::load(CONFIG).unwrap();
        assert!(!c.program[0].name.is_empty());
    }
    #[test]
    #[should_panic]
    fn invalid_config() {
        Config::load("config/invalid.toml").unwrap();
    }
    #[test]
    fn invalid_config_no_command() {
        dbg!(Config::load("config/no_command.toml"))
            .unwrap_err()
            .to_string()
            .contains("missing field `command`")
            .then_some(())
            .unwrap();
    }
    #[test]
    fn different_configs() {
        let base = Config::load("config/default.toml").unwrap();
        let diff = Config::load("config/default_diff.toml").unwrap();

        for i in 0..base.program.len() {
            assert_ne!(base.program[i], diff.program[i])
        }
    }
    #[test]
    fn equal_configs() {
        let base = Config::load("config/default_same1.toml").unwrap();
        let link = Config::load("config/default_same2.toml").unwrap();

        for i in 0..base.program.len() {
            assert_eq!(base.program[i], link.program[i])
        }
    }
}
