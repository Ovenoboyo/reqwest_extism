#![allow(dead_code)]
use std::fmt;

/// Dummy no-op Proxy implementation for WASI targets.
#[derive(Clone)]
pub struct Proxy;

impl Proxy {
    /// No-op HTTP proxy constructor.
    pub fn http(_url: &str) -> Proxy {
        Proxy
    }
    /// No-op HTTPS proxy constructor.
    pub fn https(_url: &str) -> Proxy {
        Proxy
    }
    /// No-op proxy constructor for all schemes.
    pub fn all(_url: &str) -> Proxy {
        Proxy
    }
    /// No-op custom proxy constructor.
    pub fn custom<F>(_func: F) -> Proxy
    where
        F: Fn(&str) -> Option<String> + Send + Sync + 'static,
    {
        Proxy
    }
    /// No-op basic auth setter.
    pub fn basic_auth(&self, _username: &str, _password: &str) -> &Self {
        self
    }
    /// No-op custom HTTP auth setter.
    pub fn custom_http_auth(&self, _auth: &str) -> &Self {
        self
    }
    /// No-op no_proxy setter.
    pub fn no_proxy(&self) -> &Self {
        self
    }
}

/// Dummy ProxyScheme for WASI.
#[derive(Clone, Debug)]
#[allow(dead_code)]
pub enum ProxyScheme {
    Http,
    Https,
    Socks5,
}

/// Dummy IntoProxyScheme trait for WASI.
#[allow(dead_code)]
pub trait IntoProxyScheme {
    /// No-op conversion to ProxyScheme.
    fn into_proxy_scheme(self) -> ProxyScheme;
}

impl IntoProxyScheme for ProxyScheme {
    /// No-op implementation.
    fn into_proxy_scheme(self) -> ProxyScheme {
        self
    }
}

impl fmt::Debug for Proxy {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Proxy (no-op)")
    }
}
