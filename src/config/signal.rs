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
    #[allow(dead_code)]
    pub fn as_code(&self) -> u8 {
        *self as u8
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
