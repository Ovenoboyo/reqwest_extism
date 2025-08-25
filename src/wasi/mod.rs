mod body;
mod client;
/// TODO
#[cfg(feature = "multipart")]
pub mod multipart;
mod proxy;
mod request;
mod response;

pub use self::body::Body;
pub use self::client::{Client, ClientBuilder};
pub use self::proxy::Proxy;
pub use self::request::{Request, RequestBuilder};
pub use self::response::Response;

// /// A guard that cancels a fetch request when dropped.
// struct AbortGuard {
//     ctrl: AbortController,
//     timeout: Option<(JsValue, Closure<dyn FnMut()>)>,
// }

// impl AbortGuard {
//     fn new() -> crate::Result<Self> {
//         Ok(AbortGuard {
//             ctrl: AbortController::new()
//                 .map_err(crate::error::wasm)
//                 .map_err(crate::error::builder)?,
//             timeout: None,
//         })
//     }

//     fn signal(&self) -> AbortSignal {
//         self.ctrl.signal()
//     }

//     fn timeout(&mut self, timeout: Duration) {
//         let ctrl = self.ctrl.clone();
//         let abort =
//             Closure::once(move || ctrl.abort_with_reason(&"reqwest::errors::TimedOut".into()));
//         let timeout = set_timeout(
//             abort.as_ref().unchecked_ref::<js_sys::Function>(),
//             timeout.as_millis().try_into().expect("timeout"),
//         );
//         if let Some((id, _)) = self.timeout.replace((timeout, abort)) {
//             clear_timeout(id);
//         }
//     }
// }

// impl Drop for AbortGuard {
//     fn drop(&mut self) {
//         self.ctrl.abort();
//         if let Some((id, _)) = self.timeout.take() {
//             clear_timeout(id);
//         }
//     }
// }
