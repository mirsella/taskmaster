use std::fmt;

use serde::Deserialize;

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

impl Signal {
    pub fn as_code(&self) -> u8 {
        *self as u8
    }
}

impl fmt::Display for Signal {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Signal::SIGHUP => write!(f, "SIGHUP (1)"),
            Signal::SIGINT => write!(f, "SIGINT (2)"),
            Signal::SIGQUIT => write!(f, "SIGQUIT (3)"),
            Signal::SIGILL => write!(f, "SIGILL (4)"),
            Signal::SIGTRAP => write!(f, "SIGTRAP (5)"),
            Signal::SIGABRT => write!(f, "SIGABRT (6)"),
            Signal::SIGBUS => write!(f, "SIGBUS (7)"),
            Signal::SIGFPE => write!(f, "SIGFPE (8)"),
            Signal::SIGKILL => write!(f, "SIGKILL (9)"),
            Signal::SIGUSR1 => write!(f, "SIGUSR1 (10)"),
            Signal::SIGSEGV => write!(f, "SIGSEGV (11)"),
            Signal::SIGUSR2 => write!(f, "SIGUSR2 (12)"),
            Signal::SIGPIPE => write!(f, "SIGPIPE (13)"),
            Signal::SIGALRM => write!(f, "SIGALRM (14)"),
            Signal::SIGTERM => write!(f, "SIGTERM (15)"),
            Signal::SIGSTKFLT => write!(f, "SIGSTKFLT (16)"),
            Signal::SIGCHLD => write!(f, "SIGCHLD (17)"),
            Signal::SIGCONT => write!(f, "SIGCONT (18)"),
            Signal::SIGSTOP => write!(f, "SIGSTOP (19)"),
            Signal::SIGTSTP => write!(f, "SIGTSTP (20)"),
            Signal::SIGTTIN => write!(f, "SIGTTIN (21)"),
            Signal::SIGTTOU => write!(f, "SIGTTOU (22)"),
            Signal::SIGURG => write!(f, "SIGURG (23)"),
            Signal::SIGXCPU => write!(f, "SIGXCPU (24)"),
            Signal::SIGXFSZ => write!(f, "SIGXFSZ (25)"),
            Signal::SIGVTALRM => write!(f, "SIGVTALRM (26)"),
            Signal::SIGPROF => write!(f, "SIGPROF (27)"),
            Signal::SIGWINCH => write!(f, "SIGWINCH (28)"),
            Signal::SIGIO => write!(f, "SIGIO (29)"),
            Signal::SIGPWR => write!(f, "SIGPWR (30)"),
            Signal::SIGSYS => write!(f, "SIGSYS (31)"),
        }
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
}
