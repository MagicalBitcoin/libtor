#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

/// Flags to change the behavior of the control port
///
/// Currently, all of the possible flags are only available on Unix systems since they only
/// apply to the "unix" type of ControlPort
#[derive(Debug, Clone, Copy)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum ControlPortFlag {
    #[cfg(target_family = "unix")]
    GroupWritable,
    #[cfg(target_family = "unix")]
    WorldWritable,
    #[cfg(target_family = "unix")]
    RelaxDirModeCheck,
}

/// Flags to change the behavior of the socks port
#[derive(Debug, Clone, Copy)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
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

/// Flags to change the isolation of clients connected to the control port
#[derive(Debug, Clone, Copy)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum SocksPortIsolationFlag {
    IsolateClientAddr,
    IsolateSOCKSAuth,
    IsolateClientProtocol,
    IsolateDestPort,
    IsolateDestAddr,
    KeepAliveIsolateSOCKSAuth,
}

display_like_debug!(ControlPortFlag);
display_like_debug!(SocksPortFlag);
display_like_debug!(SocksPortIsolationFlag);
