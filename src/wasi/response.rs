use std::{fmt, str::FromStr};

use bytes::Bytes;
use extism_pdk::FromBytes;
use http::{HeaderMap, HeaderName, HeaderValue, StatusCode};
use url::Url;

#[cfg(feature = "json")]
use serde::de::DeserializeOwned;

/// A Response to a submitted `Request`.
pub struct Response {
    status_code: StatusCode,
    headers: HeaderMap,
    response: extism_pdk::HttpResponse,
    // Boxed to save space (11 words to 1 word), and it's not accessed
    // frequently internally.
    url: Box<Url>,
}

impl Response {
    pub(super) fn new(url: Url, response: extism_pdk::HttpResponse) -> Response {
        let mut header_map = HeaderMap::new();
        for (key, value) in response.headers().clone() {
            header_map.append(
                HeaderName::from_str(key.as_str()).expect("Invalid header name"),
                HeaderValue::from_str(value.as_str()).expect("Invalid header value"),
            );
        }

        Response {
            url: Box::new(url),
            status_code: StatusCode::from_u16(response.status_code()).expect("Invalid status code"),
            headers: header_map,
            response,
        }
    }

    /// Get the `StatusCode` of this `Response`.
    #[inline]
    pub fn status(&self) -> StatusCode {
        self.status_code
    }

    /// Get the `Headers` of this `Response`.
    #[inline]
    pub fn headers(&self) -> &HeaderMap {
        &self.headers
    }

    /// Get a mutable reference to the `Headers` of this `Response`.
    #[inline]
    pub fn headers_mut(&mut self) -> &mut HeaderMap {
        &mut self.headers
    }

    /// Get the content-length of this response, if known.
    ///
    /// Reasons it may not be known:
    ///
    /// - The server didn't send a `content-length` header.
    /// - The response is compressed and automatically decoded (thus changing
    ///   the actual decoded length).
    pub fn content_length(&self) -> Option<u64> {
        self.headers()
            .get(http::header::CONTENT_LENGTH)?
            .to_str()
            .ok()?
            .parse()
            .ok()
    }

    /// Get the final `Url` of this `Response`.
    #[inline]
    pub fn url(&self) -> &Url {
        &self.url
    }

    /// Try to deserialize the response body as JSON.
    #[cfg(feature = "json")]
    #[cfg_attr(docsrs, doc(cfg(feature = "json")))]
    pub fn json<T: DeserializeOwned>(self) -> crate::Result<T> {
        self.response.json()
    }

    /// Get the response text.
    pub fn text(self) -> crate::Result<String> {
        let bytes = self.bytes()?;
        let string = String::from_bytes(&bytes).map_err(crate::error::builder)?;
        Ok(string)
    }

    /// Get the response as bytes
    pub fn bytes(self) -> crate::Result<Bytes> {
        Ok(self.response.body().into())
    }

    /// Turn a response into an error if the server returned an error.
    pub fn error_for_status(self) -> crate::Result<Self> {
        let status = self.status();
        if status.is_client_error() || status.is_server_error() {
            Err(crate::error::status_code(*self.url, status))
        } else {
            Ok(self)
        }
    }

    /// Turn a reference to a response into an error if the server returned an error.
    pub fn error_for_status_ref(&self) -> crate::Result<&Self> {
        let status = self.status();
        if status.is_client_error() || status.is_server_error() {
            Err(crate::error::status_code(*self.url.clone(), status))
        } else {
            Ok(self)
        }
    }
}

impl fmt::Debug for Response {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("Response")
            //.field("url", self.url())
            .field("status", &self.status())
            .field("headers", self.headers())
            .finish()
    }
}
