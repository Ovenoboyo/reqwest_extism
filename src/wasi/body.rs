#[cfg(feature = "multipart")]
use super::multipart::Form;
/// dox
use bytes::Bytes;
use std::{borrow::Cow, fmt};

/// The body of a `Request`.
///
/// In most cases, this is not needed directly, as the
/// [`RequestBuilder.body`][builder] method uses `Into<Body>`, which allows
/// passing many things (like a string or vector of bytes).
///
/// [builder]: ./struct.RequestBuilder.html#method.body
pub struct Body {
    inner: Inner,
}

enum Inner {
    Single(Single),
    /// MultipartForm holds a multipart/form-data body.
    #[cfg(feature = "multipart")]
    MultipartForm(Form),
}

#[derive(Clone)]
pub(crate) enum Single {
    Bytes(Bytes),
    Text(Cow<'static, str>),
}

impl Single {
    fn as_bytes(&self) -> &[u8] {
        match self {
            Single::Bytes(bytes) => bytes.as_ref(),
            Single::Text(text) => text.as_bytes(),
        }
    }
}

impl Body {
    /// Returns a reference to the internal data of the `Body`.
    ///
    /// `None` is returned, if the underlying data is a multipart form.
    #[inline]
    pub fn as_bytes(&self) -> Option<&[u8]> {
        match &self.inner {
            Inner::Single(single) => Some(single.as_bytes()),
            #[cfg(feature = "multipart")]
            Inner::MultipartForm(_) => None,
        }
    }

    #[cfg(feature = "multipart")]
    pub(crate) fn as_single(&self) -> Option<&Single> {
        match &self.inner {
            Inner::Single(single) => Some(single),
            Inner::MultipartForm(_) => None,
        }
    }

    #[inline]
    #[cfg(feature = "multipart")]
    pub(crate) fn from_form(f: Form) -> Body {
        Self {
            inner: Inner::MultipartForm(f),
        }
    }

    /// into_part turns a regular body into the body of a multipart/form-data part.
    #[cfg(feature = "multipart")]
    pub(crate) fn into_part(self) -> Body {
        match self.inner {
            Inner::Single(single) => Self {
                inner: Inner::Single(single),
            },
            Inner::MultipartForm(form) => Self {
                inner: Inner::MultipartForm(form),
            },
        }
    }

    pub(crate) fn try_clone(&self) -> Option<Body> {
        match &self.inner {
            Inner::Single(single) => Some(Self {
                inner: Inner::Single(single.clone()),
            }),
            #[cfg(feature = "multipart")]
            Inner::MultipartForm(_) => None,
        }
    }
}

impl From<Bytes> for Body {
    #[inline]
    fn from(bytes: Bytes) -> Body {
        Body {
            inner: Inner::Single(Single::Bytes(bytes)),
        }
    }
}

impl From<Vec<u8>> for Body {
    #[inline]
    fn from(vec: Vec<u8>) -> Body {
        Body {
            inner: Inner::Single(Single::Bytes(vec.into())),
        }
    }
}

impl From<&'static [u8]> for Body {
    #[inline]
    fn from(s: &'static [u8]) -> Body {
        Body {
            inner: Inner::Single(Single::Bytes(Bytes::from_static(s))),
        }
    }
}

impl From<String> for Body {
    #[inline]
    fn from(s: String) -> Body {
        Body {
            inner: Inner::Single(Single::Text(s.into())),
        }
    }
}

impl From<&'static str> for Body {
    #[inline]
    fn from(s: &'static str) -> Body {
        Body {
            inner: Inner::Single(Single::Text(s.into())),
        }
    }
}

impl fmt::Debug for Body {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("Body").finish()
    }
}

// Can use new methods in web-sys when requiring v0.2.93.
// > `init.method(m)` to `init.set_method(m)`
// For now, ignore their deprecation.
#[allow(deprecated)]
#[cfg(test)]
mod tests {
    // TODO
}
