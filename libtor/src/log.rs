#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

/// Log level
#[derive(Debug, Clone, Copy)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum LogLevel {
    Debug,
    Info,
    Notice,
    Warn,
    Err,
}

/// Log destination
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum LogDestination {
    Stdout,
    Stderr,
    #[cfg(target_family = "unix")]
    Syslog,
    File(String),
    #[cfg(target_os = "android")]
    Android,
}
impl std::fmt::Display for LogDestination {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            LogDestination::File(path) => write!(f, "file {}", path),
            _ => write!(f, "{:?}", self),
        }
    }
}

/// Log domain, for fine grained control
#[derive(Debug, Clone, Copy)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum LogDomain {
    General,
    Crypto,
    Net,
    Config,
    Fs,
    Protocol,
    Mm,
    Http,
    App,
    Control,
    Circ,
    Rend,
    Bug,
    Dir,
    Dirserv,
    Or,
    Edge,
    Acct,
    Hist,
    Handshake,
    Heartbeat,
    Channel,
    Sched,
    Guard,
    Consdiff,
    Dos,
    Process,
    Pt,
    Btrack,
    Mesg,
}
