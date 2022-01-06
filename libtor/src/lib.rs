//! # libtor
//!
//! Bundle and run Tor in your own project with this library!
//!
//! # Example
//!
//! ```no_run
//! use libtor::{Tor, TorFlag, TorAddress, HiddenServiceVersion};
//!
//! Tor::new()
//!     .flag(TorFlag::DataDirectory("/tmp/tor-rust".into()))
//!     .flag(TorFlag::SocksPort(19050))
//!     .flag(TorFlag::HiddenServiceDir("/tmp/tor-rust/hs-dir".into()))
//!     .flag(TorFlag::HiddenServiceVersion(HiddenServiceVersion::V3))
//!     .flag(TorFlag::HiddenServicePort(TorAddress::Port(8000), None.into()))
//!     .start()?;
//! # Ok::<(), libtor::Error>(())
//! ```

#[macro_use]
extern crate libtor_derive;
extern crate log as log_crate;
extern crate rand;
extern crate sha1;
extern crate tor_sys;

#[cfg(feature = "serde")]
extern crate serde;

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

use std::ffi::CString;
use std::thread::{self, JoinHandle};

use rand::Rng;

#[allow(unused_imports)]
use log_crate::{debug, error, info, trace};

#[macro_use]
mod utils;
/// Hidden services related flags
pub mod hs;
/// Log related flags
pub mod log;
/// ControlPort and SocksPort related flags
pub mod ports;

pub use crate::hs::*;
pub use crate::log::*;
pub use crate::ports::*;
use crate::utils::*;

trait Expand: std::fmt::Debug {
    fn expand(&self) -> Vec<String>;

    fn expand_cli(&self) -> String {
        let mut parts = self.expand();
        if parts.len() > 1 {
            let args = parts.drain(1..).collect::<Vec<_>>().join(" ");
            parts.push(format!("\"{}\"", args));
        }

        parts.join(" ")
    }
}

/// Enum that represents the size unit both in bytes and bits
#[derive(Debug, Clone, Copy)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum SizeUnit {
    Bytes,
    KBytes,
    MBytes,
    GBytes,
    TBytes,
    Bits,
    KBits,
    MBits,
    GBits,
    TBits,
}

display_like_debug!(SizeUnit);

/// Enum that represents a bool, rendered as `1` for true/enabled and `0` for false/disabled
#[derive(Debug, Clone, Copy)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum TorBool {
    True,
    False,
    Enabled,
    Disabled,
}

impl std::fmt::Display for TorBool {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let val = match self {
            TorBool::True | TorBool::Enabled => 1,
            TorBool::False | TorBool::Disabled => 0,
        };
        write!(f, "{}", val)
    }
}

impl From<bool> for TorBool {
    fn from(other: bool) -> TorBool {
        if other {
            TorBool::True
        } else {
            TorBool::False
        }
    }
}

fn log_expand(flag: &TorFlag) -> Vec<String> {
    let (levels, dest) = match flag {
        TorFlag::Log(level) => (vec![(vec![], *level)], None),
        TorFlag::LogTo(level, dest) => (vec![(vec![], *level)], Some(dest)),
        TorFlag::LogExpanded(expanded_level, dest) => (expanded_level.clone(), Some(dest)),
        _ => unimplemented!(),
    };

    let levels_str = levels
        .iter()
        .map(|(domains, level)| {
            let mut concat_str = domains
                .iter()
                .map(|(enabled, domain)| {
                    let enabled_char = if *enabled { "" } else { "~" };
                    format!("{}{:?}", enabled_char, domain)
                })
                .collect::<Vec<String>>()
                .join(",");
            if !concat_str.is_empty() {
                concat_str = format!("[{}]", concat_str);
            }

            format!("{}{:?}", concat_str, level).to_lowercase()
        })
        .collect::<Vec<String>>()
        .join(" ");
    let dest_str = dest
        .map(|d| format!(" {}", d).to_lowercase())
        .unwrap_or_default();

    vec!["Log".into(), format!("{}{}", levels_str, dest_str)]
}

/// Enum used to represent the generic concept of an "Address"
///
/// It can also represent Unix sockets on platforms that support them.
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum TorAddress {
    /// Shorthand to only encode the port
    Port(u16),
    /// Shorthand to only encode the address
    Address(String),
    /// Explicit version that encodes both the address and the port
    AddressPort(String, u16),
    /// Path to a Unix socket
    #[cfg(target_family = "unix")]
    Unix(String),
}

impl std::fmt::Display for TorAddress {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TorAddress::Port(port) => write!(f, "{}", port),
            TorAddress::Address(addr) => write!(f, "{}", addr),
            TorAddress::AddressPort(addr, port) => write!(f, "{}:{}", addr, port),
            #[cfg(target_family = "unix")]
            TorAddress::Unix(path) => write!(f, "unix:{}", path),
        }
    }
}

/// Enum that represents a subset of the options supported by Tor
///
/// Generally speaking, all the server-only features have not been mapped since this crate is
/// targeted more to a client-like usage. Arbitrary flags can still be added using the
/// `TorFlag::Custom(String)` variant.
#[derive(Debug, Clone, Expand)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum TorFlag {
    #[expand_to("-f {}")]
    #[expand_to(test = ("filename".into()) => "-f \"filename\"")]
    ConfigFile(String),
    #[expand_to("--passphrase-fd {}")]
    PassphraseFD(u32),

    #[expand_to(test = (256, SizeUnit::MBits) => "BandwidthRate \"256 MBits\"")]
    BandwidthRate(usize, SizeUnit),
    BandwidthBurst(usize, SizeUnit),
    #[expand_to(test = (true.into()) => "DisableNetwork \"1\"")]
    DisableNetwork(TorBool),

    ControlPort(u16),
    #[expand_to("ControlPort auto")]
    ControlPortAuto,
    #[expand_to("ControlPort {} {}")]
    #[cfg_attr(target_family = "unix", expand_to(test = (TorAddress::Unix("/tmp/tor-cp".into()), Some(vec![ControlPortFlag::GroupWritable].into()).into()) => "ControlPort \"unix:/tmp/tor-cp GroupWritable\""))]
    #[cfg_attr(target_family = "unix", expand_to(test = (TorAddress::Unix("/tmp/tor-cp".into()), Some(vec![ControlPortFlag::GroupWritable, ControlPortFlag::RelaxDirModeCheck].into()).into()) => "ControlPort \"unix:/tmp/tor-cp GroupWritable RelaxDirModeCheck\""))]
    ControlPortAddress(
        TorAddress,
        DisplayOption<DisplayVec<ControlPortFlag, SpaceJoiner>>,
    ),

    #[cfg(target_family = "unix")]
    ControlSocket(String),
    #[cfg(target_family = "unix")]
    ControlSocketsGroupWritable(TorBool),

    HashedControlPassword(String),
    CookieAuthentication(TorBool),
    CookieAuthFile(String),
    CookieAuthFileGroupReadable(TorBool),
    ControlPortWriteToFile(String),
    ControlPortFileGroupReadable(TorBool),

    DataDirectory(String),
    DataDirectoryGroupReadable(TorBool),
    CacheDirectory(String),
    CacheDirectoryGroupReadable(String),

    HTTPSProxy(String),
    #[expand_to("HTTPSProxyAuthenticator {}:{}")]
    #[expand_to(test = ("user".into(), "pass".into()) => "HTTPSProxyAuthenticator \"user:pass\"")]
    HTTPSProxyAuthenticator(String, String),
    Socks4Proxy(String),
    Socks5Proxy(String),
    Socks5ProxyUsername(String),
    Socks5ProxyPassword(String),

    UnixSocksGroupWritable(TorBool),

    KeepalivePeriod(usize),

    #[expand_to(with = "log_expand")]
    #[expand_to(test = (LogLevel::Notice) => "Log \"notice\"")]
    Log(LogLevel),
    #[expand_to(with = "log_expand")]
    #[expand_to(test = (LogLevel::Notice, LogDestination::File("/dev/null".into())) => "Log \"notice file /dev/null\"")]
    #[expand_to(test = (LogLevel::Notice, LogDestination::Stdout) => "Log \"notice stdout\"")]
    LogTo(LogLevel, LogDestination),
    #[expand_to(with = "log_expand")]
    #[expand_to(test = (vec![(vec![(true, LogDomain::Handshake)], LogLevel::Debug), (vec![(false, LogDomain::Net), (false, LogDomain::Mm)], LogLevel::Info), (vec![], LogLevel::Notice)], LogDestination::Stdout) => "Log \"[handshake]debug [~net,~mm]info notice stdout\"")]
    LogExpanded(Vec<(Vec<(bool, LogDomain)>, LogLevel)>, LogDestination),
    LogMessageDomains(TorBool),

    LogTimeGranularity(usize),
    TruncateLogFile(TorBool),
    SyslogIdentityTag(String),
    AndroidIdentityTag(String),
    SafeLogging(TorBool), // TODO: 'relay' unsupported at the moment

    PidFile(String),
    ProtocolWarnings(TorBool),

    User(String),
    NoExec(TorBool),

    Bridge(String, String, String),

    ConnectionPadding(TorBool), // TODO: 'auto' not supported at the moment
    ReducedConnectionPadding(TorBool),
    CircuitPadding(TorBool),
    ReducedCircuitPadding(TorBool),

    ExcludeNodes(DisplayVec<String, CommaJoiner>),
    ExcludeExitNodes(DisplayVec<String, CommaJoiner>),
    ExitNodes(DisplayVec<String, CommaJoiner>),
    MiddleNodes(DisplayVec<String, CommaJoiner>),
    EntryNodes(DisplayVec<String, CommaJoiner>),
    StrictNodes(TorBool),

    FascistFirewall(TorBool),
    FirewallPorts(DisplayVec<u16, CommaJoiner>),

    MapAddress(String, String),
    NewCircuitPeriod(usize),

    SocksPort(u16),
    #[expand_to("SocksPort auto")]
    SocksPortAuto,
    #[expand_to(rename = "SocksPort")]
    SocksPortAddress(
        TorAddress,
        DisplayOption<DisplayVec<SocksPortFlag, SpaceJoiner>>,
        DisplayOption<DisplayVec<SocksPortIsolationFlag, SpaceJoiner>>,
    ),
    SocksTimeout(usize),
    SafeSocks(TorBool),
    TestSocks(TorBool),

    UpdateBridgesFromAuthority(TorBool),
    UseBridges(TorBool),

    HiddenServiceDir(String),
    HiddenServicePort(TorAddress, DisplayOption<TorAddress>),
    HiddenServiceVersion(HiddenServiceVersion),
    #[expand_to("HiddenServiceAuthorizeClient {:?} {}")]
    HiddenServiceAuthorizeClient(HiddenServiceAuthType, DisplayVec<String, CommaJoiner>),
    HiddenServiceAllowUnknownPorts(TorBool),
    HiddenServiceMaxStreams(usize),
    HiddenServiceMaxStreamsCloseCircuit(TorBool),

    /// Custom argument, expanded as `<first_word> "<second_word> <third_word> ..."`
    #[expand_to("{}")]
    Custom(String),

    /// Don't log anything to the console
    #[expand_to("--quiet")]
    Quiet(),
    /// Only log warnings and errors to the console
    #[expand_to("--hush")]
    Hush(),
}

/// Error enum
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum Error {
    NotRunning,
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::NotRunning => write!(f, "Tor service is not running"),
        }
    }
}

impl std::error::Error for Error {}

/// Configuration builder for a Tor daemon
///
/// Offers the ability to set multiple flags and then start the daemon either in the current
/// thread, or in a new one
#[derive(Debug, Clone, Default)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct Tor {
    flags: Vec<TorFlag>,
}

impl Tor {
    /// Create a new instance
    pub fn new() -> Tor {
        Default::default()
    }

    /// Add a configuration flag
    pub fn flag(&mut self, flag: TorFlag) -> &mut Tor {
        self.flags.push(flag);
        self
    }

    /// Start the Tor daemon in the current thread
    pub fn start(&self) -> Result<u8, Error> {
        unsafe {
            let config = tor_sys::tor_main_configuration_new();
            let mut argv = vec![String::from("tor")];
            argv.extend_from_slice(
                &self
                    .flags
                    .iter()
                    .map(TorFlag::expand)
                    .flatten()
                    .collect::<Vec<String>>(),
            );

            debug!("Starting tor with args: {:#?}", argv);

            let argv: Vec<_> = argv.into_iter().map(|s| CString::new(s).unwrap()).collect();
            let argv: Vec<_> = argv.iter().map(|s| s.as_ptr()).collect();
            tor_sys::tor_main_configuration_set_command_line(
                config,
                argv.len() as i32,
                argv.as_ptr(),
            );

            let result = tor_sys::tor_run_main(config);

            tor_sys::tor_main_configuration_free(config);

            Ok(result as u8)
        }
    }

    /// Starts the Tor daemon in a background detached thread and return its handle
    pub fn start_background(&self) -> JoinHandle<Result<u8, Error>> {
        let cloned = self.clone();
        thread::spawn(move || cloned.start())
    }
}

/// Generate a hashed password to use HashedControlPassword
pub fn generate_hashed_password(secret: &str) -> String {
    // This code is rewrite of
    // https://gist.github.com/s4w3d0ff/9d65ec5866d78842547183601b2fa4d5
    // s4w3d0ff and jamesacampbell, Thank you!

    let c: usize = 96;
    const EXPBIAS: usize = 6;
    let mut count = (16_usize + (c & 15_usize)) << ((c >> 4_usize) + EXPBIAS);
    let mut d = sha1::Sha1::new();

    let slen = 8 + (secret.as_bytes().len());
    let mut tmp = Vec::with_capacity(slen);
    tmp.extend_from_slice(&rand::rngs::OsRng.gen::<u64>().to_ne_bytes());
    tmp.extend_from_slice(secret.as_bytes());

    while count != 0 {
        if count > slen {
            d.update(&tmp);
            count -= slen;
        } else {
            d.update(&tmp[..count]);
            break;
        }
    }
    let hashed = d.digest().to_string();
    let tmp = tmp[..8]
        .iter()
        .map(|n| format!("{:02x}", n))
        .collect::<String>();

    format!("16:{}{:02x}{}", tmp, c as u8, hashed)
}

#[cfg(test)]
mod tests {
    use crate::*;

    #[test]
    #[ignore]
    fn test_run() {
        Tor::new()
            .flag(TorFlag::DataDirectory("/tmp/tor-rust".into()))
            .flag(TorFlag::HiddenServiceDir("/tmp/tor-rust/hs-dir".into()))
            .flag(TorFlag::HiddenServiceVersion(HiddenServiceVersion::V3))
            .flag(TorFlag::HiddenServicePort(
                TorAddress::Port(80),
                Some(TorAddress::AddressPort("example.org".into(), 80)).into(),
            ))
            .flag(TorFlag::SocksPort(0))
            .start_background();

        std::thread::sleep(std::time::Duration::from_secs(10));
    }

    #[test]
    fn test_generate_hashed_password() {
        for i in 0..0xff {
            let secret = i.to_string();
            let hashed_pwd = generate_hashed_password(&secret);
            assert_eq!(hashed_pwd.len(), 61);
        }
    }
}
