#[macro_use]
extern crate libtor_derive;
extern crate tor_sys;

pub trait Expand: std::fmt::Debug {
    fn expand(&self) -> String;
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

// TODO: make as a macro, DisplayLikeDebug
macro_rules! display_like_debug {
    ($type:ty) => {
        impl std::fmt::Display for $type {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                write!(f, "{:?}", self)
            }
        }
    };
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
        use TorBool::*;

        let val = match self {
            True | Enabled => 1,
            False | Disabled => 0,
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

#[derive(Debug, Clone, Copy)]
pub enum ControlPortFlag {
    GroupWritable,
    WorldWritable,
    RelaxDirModeCheck,
}

display_like_debug!(ControlPortFlag);
display_like_debug!(SocksPortFlag);
display_like_debug!(SocksPortIsolationFlag);

#[derive(Debug, Clone, Copy)]
pub enum LogLevel {
    Debug,
    Info,
    Notice,
    Warn,
    Err,
}

#[derive(Debug, Clone)]
pub enum LogDestination {
    Stdout,
    Stderr,
    Syslog, // TODO: only unix
    File(String),
    Android, // TODO: only android
}

#[derive(Debug, Clone, Copy)]
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

fn log_expand(flag: &TorFlag) -> String {
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

    format!("Log \"{}{}\"", levels_str, dest_str)
}

pub trait Joiner: std::fmt::Debug + std::clone::Clone {
    fn joiner(&self) -> String;
    fn new() -> Self;
}

#[derive(Debug, Clone)]
pub struct CommaJoiner {}
impl Joiner for CommaJoiner {
    fn joiner(&self) -> String {
        ",".to_string()
    }

    fn new() -> Self {
        CommaJoiner {}
    }
}

#[derive(Debug, Clone)]
pub struct SpaceJoiner {}
impl Joiner for SpaceJoiner {
    fn joiner(&self) -> String {
        " ".to_string()
    }

    fn new() -> Self {
        SpaceJoiner {}
    }
}

#[derive(Debug, Clone)]
pub struct DisplayVec<T: std::fmt::Debug + std::fmt::Display, J: Joiner> {
    vec: Vec<T>,
    joiner: J,
}

impl<T: std::fmt::Debug + std::fmt::Display, J: Joiner> std::fmt::Display for DisplayVec<T, J> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let joined: String = self
            .vec
            .iter()
            .map(|v| format!("{}", v))
            .collect::<Vec<String>>()
            .join(&self.joiner.joiner());
        write!(f, "{}", joined)
    }
}

impl<T: std::fmt::Debug + std::fmt::Display, J: Joiner> From<Vec<T>> for DisplayVec<T, J> {
    fn from(vec: Vec<T>) -> DisplayVec<T, J> {
        DisplayVec {
            vec,
            joiner: J::new(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct DisplayOption<T: std::fmt::Debug + std::fmt::Display> {
    option: Option<T>,
}

impl<T: std::fmt::Debug + std::fmt::Display> std::fmt::Display for DisplayOption<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self.option {
            Some(val) => write!(f, "{}", val),
            None => Ok(()),
        }
    }
}

impl<T: std::fmt::Debug + std::fmt::Display> From<Option<T>> for DisplayOption<T> {
    fn from(option: Option<T>) -> DisplayOption<T> {
        DisplayOption { option }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum SocksPortFlag {
    NoIPv4Traffic,
    IPv6Traffic,
    PreferIPv6,
    NoDNSRequest,
    NoOnionTraffic,
    OnionTrafficOnly,
    CacheIPv4DNS,
    CacheIPv6DNS,
    GroupWritable,
    WorldWritable,
    CacheDNS,
    UseIPv4Cache,
    UseIPv6Cache,
    UseDNSCache,
    PreferIPv6Automap,
    PreferSOCKSNoAuth,
}

#[derive(Debug, Clone, Copy)]
pub enum SocksPortIsolationFlag {
    IsolateClientAddr,
    IsolateSOCKSAuth,
    IsolateClientProtocol,
    IsolateDestPort,
    IsolateDestAddr,
    KeepAliveIsolateSOCKSAuth,
}

trait OptionVecToString {
    fn option_vec_to_string(&self) -> String;
}

impl<T: std::fmt::Debug> OptionVecToString for Option<Vec<T>> {
    fn option_vec_to_string(&self) -> String {
        self.as_ref()
            .map(|flags| {
                flags
                    .iter()
                    .map(|f| format!("{:?} ", f))
                    .collect::<String>()
            })
            .unwrap_or_default()
    }
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

#[derive(Debug, Clone, Copy)]
pub enum HiddenServiceVersion {
    V2 = 2,
    V3 = 3,
}

impl std::fmt::Display for HiddenServiceVersion {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", *self as u8)
    }
}

#[derive(Debug, Clone, Copy)]
pub enum HiddenServiceAuthType {
    Basic,
    Stealth,
}

#[derive(Debug, Clone, Expand)]
pub enum TorFlag {
    #[expand_to("-f {}")]
    #[expand_to(test = ("filename".into()) => "-f filename")]
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
    #[expand_to("ControlPort \"{} {}\"")]
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
    #[expand_to("HTTPSProxyAuthenticator \"{}:{}\"")]
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
}

#[derive(Debug, Expand)]
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

pub struct Tor {
    _subcommand: Option<TorSubcommand>,
    _flags: Vec<TorFlag>,
}

#[cfg(test)]
mod tests {
    use crate::*;

    #[test]
    #[ignore]
    fn it_works() {
        let a = TorFlag::SocksPortAddress(
            TorAddress::AddressPort("[2001:0db8:85a3:0000:0000:8a2e:0370:7334]:8080".into()),
            Some(vec![SocksPortFlag::NoIPv4Traffic].into()).into(),
            None.into(),
        );
        println!("{}", a.expand());
    }
}
