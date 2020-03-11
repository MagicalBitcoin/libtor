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
extern crate tor_sys;

use std::ffi::CString;

#[macro_use]
pub mod utils;
pub mod hs;
pub mod log;
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

#[derive(Debug, Clone, Copy)]
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

#[derive(Debug, Clone, Copy)]
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
        .map(|d| format!(" {:?}", d).to_lowercase())
        .unwrap_or_default();

    vec!["Log".into(), format!("{}{}", levels_str, dest_str)]
}

#[derive(Debug, Clone)]
pub enum TorAddress {
    Port(u16),
    Address(String),
    AddressPort(String),
    #[cfg(target_family = "unix")]
    Unix(String),
}

impl std::fmt::Display for TorAddress {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TorAddress::Port(port) => write!(f, "{}", port),
            TorAddress::Address(addr) => write!(f, "{}", addr),
            TorAddress::AddressPort(addr) => write!(f, "{}", addr),
            #[cfg(target_family = "unix")]
            TorAddress::Unix(path) => write!(f, "unix:{}", path),
        }
    }
}

#[derive(Debug, Clone, Expand)]
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
    #[expand_to(test = (TorAddress::Unix("/tmp/tor-cp".into()), Some(vec![ControlPortFlag::GroupWritable].into()).into()) => "ControlPort \"unix:/tmp/tor-cp GroupWritable\"")]
    #[expand_to(test = (TorAddress::Unix("/tmp/tor-cp".into()), Some(vec![ControlPortFlag::GroupWritable, ControlPortFlag::RelaxDirModeCheck].into()).into()) => "ControlPort \"unix:/tmp/tor-cp GroupWritable RelaxDirModeCheck\"")]
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

    #[expand_to("{}")]
    Custom(String),
}

#[derive(Debug, Clone, Expand)]
pub enum TorSubcommand {
    #[expand_to("--hash-password {password}")]
    HashPassword { password: String },
    #[expand_to("--verify-config")]
    VerifyConfig,
    #[expand_to("--list-fingerprint")]
    ListFingerprint,
    #[expand_to("--version")]
    Version,
    #[expand_to("--keygen")]
    Keygen {
        #[expand_to(ignore)]
        password: Option<String>,
    },
    #[expand_to("--keygen --newpass")]
    KeygenNewpass {
        #[expand_to(ignore)]
        old_password: Option<String>,
        #[expand_to(ignore)]
        new_password: Option<String>,
    },
}

#[derive(Debug, Clone)]
pub enum Error {
    DuplicatedSubcommand,
}

#[derive(Debug, Clone, Default)]
pub struct Tor {
    subcommand: Option<TorSubcommand>,
    flags: Vec<TorFlag>,
}

impl Tor {
    pub fn new() -> Tor {
        Default::default()
    }

    pub fn new_with_subcommand(subcommand: TorSubcommand) -> Tor {
        Tor {
            subcommand: Some(subcommand),
            ..Default::default()
        }
    }

    pub fn subcommand(&mut self, subcommand: TorSubcommand) -> Result<&mut Tor, Error> {
        if self.subcommand.is_some() {
            Err(Error::DuplicatedSubcommand)
        } else {
            self.subcommand = Some(subcommand);
            Ok(self)
        }
    }

    pub fn flag(&mut self, flag: TorFlag) -> &mut Tor {
        self.flags.push(flag);
        self
    }

    // TODO: password from stdin
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

            if let Some(subcommand) = &self.subcommand {
                argv.extend_from_slice(&subcommand.expand());
            }

            println!("{:#?}", argv);

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
                Some(TorAddress::AddressPort("example.org:80".into())).into(),
            ))
            .flag(TorFlag::SocksPort(0))
            .flag(TorFlag::LogExpanded(
                vec![
                    (vec![(true, LogDomain::Handshake)], LogLevel::Info),
                    (vec![], LogLevel::Notice),
                ],
                LogDestination::Stdout,
            ))
            .start()
            .unwrap();
    }
}
