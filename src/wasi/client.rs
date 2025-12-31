use extism_pdk::HttpRequest;
use http::header::USER_AGENT;
use http::{HeaderMap, HeaderValue, Method};
use std::convert::TryInto;
use std::future::Future;
use std::net::IpAddr;
use std::{fmt, sync::Arc};

#[cfg(feature = "cookies")]
use crate::cookie;

use super::{Request, RequestBuilder, Response};
use crate::{IntoUrl, Proxy};

/// dox
#[derive(Clone)]
pub struct Client {
    config: Arc<Config>,
}

/// dox
pub struct ClientBuilder {
    config: Config,
}

impl Client {
    /// dox
    pub fn new() -> Self {
        Client::builder().build().expect("Client::new()")
    }

    /// dox
    pub fn builder() -> ClientBuilder {
        ClientBuilder::new()
    }

    /// Convenience method to make a `GET` request to a URL.
    ///
    /// # Errors
    ///
    /// This method fails whenever supplied `Url` cannot be parsed.
    pub fn get<U: IntoUrl>(&self, url: U) -> RequestBuilder {
        self.request(Method::GET, url)
    }

    /// Convenience method to make a `POST` request to a URL.
    ///
    /// # Errors
    ///
    /// This method fails whenever supplied `Url` cannot be parsed.
    pub fn post<U: IntoUrl>(&self, url: U) -> RequestBuilder {
        self.request(Method::POST, url)
    }

    /// Convenience method to make a `PUT` request to a URL.
    ///
    /// # Errors
    ///
    /// This method fails whenever supplied `Url` cannot be parsed.
    pub fn put<U: IntoUrl>(&self, url: U) -> RequestBuilder {
        self.request(Method::PUT, url)
    }

    /// Convenience method to make a `PATCH` request to a URL.
    ///
    /// # Errors
    ///
    /// This method fails whenever supplied `Url` cannot be parsed.
    pub fn patch<U: IntoUrl>(&self, url: U) -> RequestBuilder {
        self.request(Method::PATCH, url)
    }

    /// Convenience method to make a `DELETE` request to a URL.
    ///
    /// # Errors
    ///
    /// This method fails whenever supplied `Url` cannot be parsed.
    pub fn delete<U: IntoUrl>(&self, url: U) -> RequestBuilder {
        self.request(Method::DELETE, url)
    }

    /// Convenience method to make a `HEAD` request to a URL.
    ///
    /// # Errors
    ///
    /// This method fails whenever supplied `Url` cannot be parsed.
    pub fn head<U: IntoUrl>(&self, url: U) -> RequestBuilder {
        self.request(Method::HEAD, url)
    }

    /// Start building a `Request` with the `Method` and `Url`.
    ///
    /// Returns a `RequestBuilder`, which will allow setting headers and
    /// request body before sending.
    ///
    /// # Errors
    ///
    /// This method fails whenever supplied `Url` cannot be parsed.
    pub fn request<U: IntoUrl>(&self, method: Method, url: U) -> RequestBuilder {
        let req = url.into_url().map(move |url| Request::new(method, url));
        RequestBuilder::new(self.clone(), req)
    }

    /// Executes a `Request`.
    ///
    /// A `Request` can be built manually with `Request::new()` or obtained
    /// from a RequestBuilder with `RequestBuilder::build()`.
    ///
    /// You should prefer to use the `RequestBuilder` and
    /// `RequestBuilder::send()`.
    ///
    /// # Errors
    ///
    /// This method fails if there was an error while sending request,
    /// redirect loop was detected or redirect limit was exhausted.
    pub fn execute(
        &self,
        request: Request,
    ) -> impl Future<Output = Result<Response, crate::Error>> {
        self.execute_request(request)
    }

    // merge request headers with Client default_headers, prior to external http fetch
    fn merge_headers(&self, req: &mut Request) {
        use http::header::Entry;
        let headers: &mut HeaderMap = req.headers_mut();
        // insert default headers in the request headers
        // without overwriting already appended headers.
        for (key, value) in self.config.headers.iter() {
            if let Entry::Vacant(entry) = headers.entry(key) {
                entry.insert(value.clone());
            }
        }
    }

    pub(super) fn execute_request(
        &self,
        mut req: Request,
    ) -> impl Future<Output = Result<Response, crate::Error>> {
        self.merge_headers(&mut req);

        // Add cookies from the cookie store.
        #[cfg(feature = "cookies")]
        {
            if let Some(cookie_store) = self.config.cookie_store.as_ref() {
                if req.headers.get(crate::header::COOKIE).is_none() {
                    if let Some(header) = cookie_store.cookies(req.url()) {
                        req.headers.insert(crate::header::COOKIE, header);
                    }
                }
            }
        }

        fetch(req)
    }
}

impl Default for Client {
    fn default() -> Self {
        Self::new()
    }
}

impl fmt::Debug for Client {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut builder = f.debug_struct("Client");
        self.config.fmt_fields(&mut builder);
        builder.finish()
    }
}

impl fmt::Debug for ClientBuilder {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut builder = f.debug_struct("ClientBuilder");
        self.config.fmt_fields(&mut builder);
        builder.finish()
    }
}

async fn fetch(req: Request) -> crate::Result<Response> {
    let mut request = HttpRequest::new(req.url().as_str()).with_method(req.method().as_str());
    for (header_name, header_val) in req.headers().into_iter() {
        request = request.with_header(
            header_name.as_str(),
            header_val.to_str().map_err(crate::error::builder)?,
        );
    }

    let body = req.body().map(|b| b.as_bytes()).flatten();
    let resp = extism_pdk::http::request(&request, body).map_err(crate::error::builder)?;

    Ok(self::Response::new(req.url().clone(), resp))
}

// ===== impl ClientBuilder =====

impl ClientBuilder {
    /// dox
    pub fn new() -> Self {
        ClientBuilder {
            config: Config::default(),
        }
    }

    /// Returns a 'Client' that uses this ClientBuilder configuration
    pub fn build(mut self) -> Result<Client, crate::Error> {
        if let Some(err) = self.config.error {
            return Err(err);
        }

        let config = std::mem::take(&mut self.config);
        Ok(Client {
            config: Arc::new(config),
        })
    }

    /// Sets the `User-Agent` header to be used by this client.
    pub fn user_agent<V>(mut self, value: V) -> ClientBuilder
    where
        V: TryInto<HeaderValue>,
        V::Error: Into<http::Error>,
    {
        match value.try_into() {
            Ok(value) => {
                self.config.headers.insert(USER_AGENT, value);
            }
            Err(e) => {
                self.config.error = Some(crate::error::builder(e.into()));
            }
        }
        self
    }

    /// Sets the default headers for every request
    pub fn default_headers(mut self, headers: HeaderMap) -> ClientBuilder {
        for (key, value) in headers.iter() {
            self.config.headers.insert(key, value.clone());
        }
        self
    }

    /// no-op
    pub fn local_address<T>(self, _: T) -> ClientBuilder
    where
        T: Into<Option<IpAddr>>,
    {
        self
    }

    /// no-op
    pub fn proxy(self, _: Proxy) -> ClientBuilder {
        self
    }

    /// Enable a persistent cookie store for the client.
    ///
    /// Cookies received in responses will be preserved and included in
    /// additional requests.
    ///
    /// By default, no cookie store is used.
    ///
    /// # Optional
    ///
    /// This requires the optional `cookies` feature to be enabled.
    #[cfg(feature = "cookies")]
    #[cfg_attr(docsrs, doc(cfg(feature = "cookies")))]
    pub fn cookie_store(mut self, enable: bool) -> ClientBuilder {
        if enable {
            self.cookie_provider(Arc::new(cookie::Jar::default()))
        } else {
            self.config.cookie_store = None;
            self
        }
    }

    /// Set the persistent cookie store for the client.
    ///
    /// Cookies received in responses will be passed to this store, and
    /// additional requests will query this store for cookies.
    ///
    /// By default, no cookie store is used.
    ///
    /// # Optional
    ///
    /// This requires the optional `cookies` feature to be enabled.
    #[cfg(feature = "cookies")]
    #[cfg_attr(docsrs, doc(cfg(feature = "cookies")))]
    pub fn cookie_provider<C: cookie::CookieStore + 'static>(
        mut self,
        cookie_store: Arc<C>,
    ) -> ClientBuilder {
        self.config.cookie_store = Some(cookie_store as _);
        self
    }
}

impl Default for ClientBuilder {
    fn default() -> Self {
        Self::new()
    }
}

struct Config {
    headers: HeaderMap,
    #[cfg(feature = "cookies")]
    cookie_store: Option<Arc<dyn cookie::CookieStore>>,
    error: Option<crate::Error>,
}

impl std::fmt::Debug for Config {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let mut build = f.debug_struct("Config");

        #[cfg(feature = "cookies")] 
        build.field("cookie_store", &self.cookie_store.is_some());

        build.field("headers", &self.headers).finish()
    }
}

impl Default for Config {
    fn default() -> Config {
        Config {
            headers: HeaderMap::new(),
            error: None,
            #[cfg(feature = "cookies")]
            cookie_store: None,
        }
    }
}

impl Config {
    fn fmt_fields(&self, f: &mut fmt::DebugStruct<'_, '_>) {
        f.field("default_headers", &self.headers);
    }
}

#[cfg(test)]
mod tests {
    // TODO
}
