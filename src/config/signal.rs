use serde::Deserialize;
use std::fmt;

/// To get the associated signal number, cast the enum to u8: `*Signal::SIGHUP as u8`
#[derive(Deserialize, Debug, PartialEq, Eq, Copy, Clone, Default)]
#[allow(clippy::upper_case_acronyms)]
pub enum Signal {
    SIGHUP = 1,
    SIGINT = 2,
    SIGQUIT = 3,
    SIGILL = 4,
    SIGTRAP = 5,
    SIGABRT = 6,
    SIGBUS = 7,
    SIGFPE = 8,
    SIGKILL = 9,
    SIGUSR1 = 10,
    SIGSEGV = 11,
    SIGUSR2 = 12,
    SIGPIPE = 13,
    SIGALRM = 14,
    #[default]
    SIGTERM = 15,
    SIGSTKFLT = 16,
    SIGCHLD = 17,
    SIGCONT = 18,
    SIGSTOP = 19,
    SIGTSTP = 20,
    SIGTTIN = 21,
    SIGTTOU = 22,
    SIGURG = 23,
    SIGXCPU = 24,
    SIGXFSZ = 25,
    SIGVTALRM = 26,
    SIGPROF = 27,
    SIGWINCH = 28,
    SIGIO = 29,
    SIGPWR = 30,
    SIGSYS = 31,
}

impl TryFrom<i32> for Signal {
    type Error = ();

    fn try_from(num: i32) -> Result<Self, Self::Error> {
        match num {
            1 => Ok(Signal::SIGHUP),
            2 => Ok(Signal::SIGINT),
            3 => Ok(Signal::SIGQUIT),
            4 => Ok(Signal::SIGILL),
            5 => Ok(Signal::SIGTRAP),
            6 => Ok(Signal::SIGABRT),
            7 => Ok(Signal::SIGBUS),
            8 => Ok(Signal::SIGFPE),
            9 => Ok(Signal::SIGKILL),
            10 => Ok(Signal::SIGUSR1),
            11 => Ok(Signal::SIGSEGV),
            12 => Ok(Signal::SIGUSR2),
            13 => Ok(Signal::SIGPIPE),
            14 => Ok(Signal::SIGALRM),
            15 => Ok(Signal::SIGTERM),
            16 => Ok(Signal::SIGSTKFLT),
            17 => Ok(Signal::SIGCHLD),
            18 => Ok(Signal::SIGCONT),
            19 => Ok(Signal::SIGSTOP),
            20 => Ok(Signal::SIGTSTP),
            21 => Ok(Signal::SIGTTIN),
            22 => Ok(Signal::SIGTTOU),
            23 => Ok(Signal::SIGURG),
            24 => Ok(Signal::SIGXCPU),
            25 => Ok(Signal::SIGXFSZ),
            26 => Ok(Signal::SIGVTALRM),
            27 => Ok(Signal::SIGPROF),
            28 => Ok(Signal::SIGWINCH),
            29 => Ok(Signal::SIGIO),
            30 => Ok(Signal::SIGPWR),
            31 => Ok(Signal::SIGSYS),
            _ => Err(()),
        }
    }
}

impl fmt::Display for Signal {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{self:?} ({})", *self as u8)
    }
}

#[cfg(test)]
mod tests {
    use super::Signal;

    #[test]
    fn sighup() {
        assert_eq!(Signal::SIGHUP as u8, 1);
    }
    #[test]
    fn sigstop() {
        assert_eq!(Signal::SIGSTOP as u8, 19);
    }
    #[test]
    fn sigsys() {
        assert_eq!(Signal::SIGSYS as u8, 31);
    }
    #[test]
    fn display_sighup() {
        assert_eq!(format!("{}", Signal::SIGHUP), "SIGHUP (1)");
    }
    #[test]
    fn display_sigstop() {
        assert_eq!(format!("{}", Signal::SIGSTOP), "SIGSTOP (19)");
    }
}
