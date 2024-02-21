use serde::Deserialize;

#[allow(clippy::upper_case_acronyms)]
#[derive(Deserialize, Debug)]
pub enum Signal {
    SIGHUP,
    SIGINT,
    SIGQUIT,
    SIGILL,
    SIGTRAP,
    SIGABRT,
    SIGBUS,
    SIGFPE,
    SIGKILL,
    SIGUSR1,
    SIGSEGV,
    SIGUSR2,
    SIGPIPE,
    SIGALRM,
    SIGTERM,
    SIGSTKFLT,
    SIGCHLD,
    SIGCONT,
    SIGSTOP,
    SIGTSTP,
    SIGTTIN,
    SIGTTOU,
    SIGURG,
    SIGXCPU,
    SIGXFSZ,
    SIGVTALRM,
    SIGPROF,
    SIGWINCH,
    SIGIO,
    SIGPOLL,
    SIGPWR,
    SIGSYS,
    SIGUNUSED,
}

/// implementing From also implements Into, and permit us to use eg. `u8.into()` and vice versa
impl From<u8> for Signal {
    fn from(value: u8) -> Self {
        match value {
            1 => Signal::SIGHUP,
            2 => Signal::SIGINT,
            3 => Signal::SIGQUIT,
            4 => Signal::SIGILL,
            5 => Signal::SIGTRAP,
            6 => Signal::SIGABRT,
            7 => Signal::SIGBUS,
            8 => Signal::SIGFPE,
            9 => Signal::SIGKILL,
            10 => Signal::SIGUSR1,
            11 => Signal::SIGSEGV,
            12 => Signal::SIGUSR2,
            13 => Signal::SIGPIPE,
            14 => Signal::SIGALRM,
            15 => Signal::SIGTERM,
            16 => Signal::SIGSTKFLT,
            17 => Signal::SIGCHLD,
            18 => Signal::SIGCONT,
            19 => Signal::SIGSTOP,
            20 => Signal::SIGTSTP,
            21 => Signal::SIGTTIN,
            22 => Signal::SIGTTOU,
            23 => Signal::SIGURG,
            24 => Signal::SIGXCPU,
            25 => Signal::SIGXFSZ,
            26 => Signal::SIGVTALRM,
            27 => Signal::SIGPROF,
            28 => Signal::SIGWINCH,
            29 => Signal::SIGIO,
            30 => Signal::SIGPOLL,
            31 => Signal::SIGPWR,
            32 => Signal::SIGSYS,
            33 => Signal::SIGUNUSED,
            _ => panic!("Invalid signal value: {}", value),
        }
    }
}
