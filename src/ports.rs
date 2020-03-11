#[derive(Debug, Clone, Copy)]
pub enum ControlPortFlag {
    GroupWritable,
    WorldWritable,
    RelaxDirModeCheck,
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

display_like_debug!(ControlPortFlag);
display_like_debug!(SocksPortFlag);
display_like_debug!(SocksPortIsolationFlag);
