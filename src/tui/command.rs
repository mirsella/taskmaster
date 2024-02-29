use std::str::FromStr;
use tracing::Level;

#[derive(Debug)]
pub enum Command {
    Quit,
    Start(String),
    Stop(String),
    Restart(String),
    Reload(String),
    LogLevel(Level),
}
impl FromStr for Command {
    type Err = ();
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let lower = s.to_lowercase();
        let mut s = lower.split_whitespace();
        let cmd = s.next().ok_or(())?;
        let arg = s.next().unwrap_or_default().to_string();
        if s.next().is_some() {
            return Err(());
        }
        if "quit".starts_with(cmd) {
            return Ok(Self::Quit);
        } else if "start".starts_with(cmd) {
            return Ok(Self::Start(arg));
        } else if "stop".starts_with(cmd) {
            return Ok(Self::Stop(arg));
        } else if "restart".starts_with(cmd) {
            return Ok(Self::Restart(arg));
        } else if "reload".starts_with(cmd) {
            return Ok(Self::Reload(arg));
        } else if "loglevel".starts_with(cmd) && !arg.is_empty() {
            return Ok(Self::LogLevel(Level::from_str(&arg).map_err(|_| ())?));
        }
        Err(())
    }
}
impl Command {
    pub const HELP: &'static str =
        "quit (2x to force) | start <name?> | stop <name?> | restart <name?> | reload <path?> | loglevel <level>";
}
