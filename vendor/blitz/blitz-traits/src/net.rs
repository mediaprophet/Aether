//! Abstractions of networking so that custom networking implementations can be provided

pub use bytes::Bytes;
pub use http::{self, HeaderMap, Method};
use serde::{
    Serialize,
    ser::{SerializeSeq, SerializeTuple},
};
use std::sync::{
    Arc,
    atomic::{AtomicBool, Ordering},
};
use std::{ops::Deref, path::PathBuf};
pub use url::Url;

/// A type that fetches resources for a Document.
///
/// This may be over the network via http(s), via the filesystem, or some other method.
pub trait NetProvider: Send + Sync + 'static {
    fn fetch(&self, doc_id: usize, request: Request, handler: Box<dyn NetHandler>);

    /// Whether this provider is a no-op (e.g. `DummyNetProvider`) that will never
    /// deliver resources. When true, callers must NOT register resources as
    /// "pending critical" — doing so blocks painting forever, since the
    /// completion callback never fires. Used by integrations that feed a
    /// pre-rendered DOM and perform no sub-fetches (e.g. aginxbrowser).
    fn is_noop(&self) -> bool {
        false
    }
}

/// A type that parses raw bytes from a network request into a Data and then calls
/// the NetCallack with the result.
pub trait NetHandler: Send + Sync + 'static {
    fn bytes(self: Box<Self>, resolved_url: String, bytes: Bytes);

    /// Terminal transport failure. Implementations tracking pending resources
    /// must complete them here. The default preserves legacy external handlers.
    fn failed(self: Box<Self>, _url: String, _error: NetError) {}
}

/// A terminal fetch outcome; no response body is presented as successful bytes.
/// Added for Aether by Timothy Charles Holborn, copyright 2026.
#[derive(Clone, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub enum NetError {
    Cancelled,
    HttpStatus(u16),
    AccessDenied,
    Unavailable,
    Transport(String),
}

impl std::fmt::Display for NetError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Cancelled => f.write_str("request cancelled"),
            Self::HttpStatus(status) => write!(f, "HTTP {status}"),
            Self::AccessDenied => f.write_str("asset access denied"),
            Self::Unavailable => f.write_str("transport unavailable"),
            Self::Transport(message) => f.write_str(message),
        }
    }
}
impl std::error::Error for NetError {}

/// Report terminal cancellation when a provider drops a request, including
/// runtime shutdown. Does not grant authority or change successful payloads.
pub struct CompletionHandler {
    url: String,
    inner: Option<Box<dyn NetHandler>>,
    signal: Option<AbortSignal>,
}
impl CompletionHandler {
    pub fn boxed(url: String, handler: Box<dyn NetHandler>) -> Box<dyn NetHandler> {
        Self::with_signal(url, None, handler)
    }
    pub fn with_signal(
        url: String,
        signal: Option<AbortSignal>,
        handler: Box<dyn NetHandler>,
    ) -> Box<dyn NetHandler> {
        Box::new(Self {
            url,
            inner: Some(handler),
            signal,
        })
    }
}
impl NetHandler for CompletionHandler {
    fn bytes(mut self: Box<Self>, url: String, bytes: Bytes) {
        if let Some(handler) = self.inner.take() {
            if self.signal.as_ref().is_some_and(AbortSignal::aborted) {
                handler.failed(url, NetError::Cancelled);
            } else {
                handler.bytes(url, bytes);
            }
        }
    }
    fn failed(mut self: Box<Self>, url: String, error: NetError) {
        if let Some(handler) = self.inner.take() {
            handler.failed(url, error);
        }
    }
}
impl Drop for CompletionHandler {
    fn drop(&mut self) {
        if let Some(handler) = self.inner.take() {
            handler.failed(std::mem::take(&mut self.url), NetError::Cancelled);
        }
    }
}

#[cfg(test)]
#[path = "net_tests.rs"]
mod tests;

/// A callback which gets called every time a network request completes
// Q: Should we use std::task::Waker for this?
pub trait NetWaker: Send + Sync + 'static {
    fn wake(&self, client_id: usize);
}

impl<F: Fn(usize) + Send + Sync + 'static> NetWaker for F {
    fn wake(&self, doc_id: usize) {
        self(doc_id)
    }
}

#[non_exhaustive]
#[derive(Debug, Clone)]
/// A request type loosely representing <https://fetch.spec.whatwg.org/#requests>
pub struct Request {
    pub url: Url,
    pub method: Method,
    pub content_type: Option<String>,
    pub headers: HeaderMap,
    pub body: Body,
    pub signal: Option<AbortSignal>,
}
impl Request {
    /// A get request to the specified Url and an empty body
    pub fn get(url: Url) -> Self {
        Self {
            url,
            method: Method::GET,
            content_type: None,
            headers: HeaderMap::new(),
            body: Body::Empty,
            signal: None,
        }
    }

    pub fn signal(mut self, signal: AbortSignal) -> Self {
        self.signal = Some(signal);
        self
    }
}

#[derive(Debug, Clone)]
pub enum Body {
    Bytes(Bytes),
    Form(FormData),
    Empty,
}

/// A list of form entries used for form submission
#[derive(Debug, Clone, PartialEq, Default)]
pub struct FormData(pub Vec<Entry>);
impl FormData {
    /// Creates a new empty FormData
    pub fn new() -> Self {
        FormData(Vec::new())
    }
}
impl Serialize for FormData {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut seq_serializer = serializer.serialize_seq(Some(self.len()))?;
        for entry in &self.0 {
            seq_serializer.serialize_element(entry)?;
        }
        seq_serializer.end()
    }
}
impl Deref for FormData {
    type Target = Vec<Entry>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

/// A single form entry consisting of a name and value
#[derive(Debug, Clone, PartialEq)]
pub struct Entry {
    pub name: String,
    pub value: EntryValue,
}
impl Serialize for Entry {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut serializer = serializer.serialize_tuple(2)?;
        serializer.serialize_element(&self.name)?;
        match &self.value {
            EntryValue::String(s) => serializer.serialize_element(s)?,
            EntryValue::File(p) => serializer.serialize_element(p.to_str().unwrap_or_default())?,
            EntryValue::EmptyFile => serializer.serialize_element("")?,
        }
        serializer.end()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum EntryValue {
    String(String),
    File(PathBuf),
    EmptyFile,
}
impl AsRef<str> for EntryValue {
    fn as_ref(&self) -> &str {
        match self {
            EntryValue::String(s) => s,
            EntryValue::File(p) => p.to_str().unwrap_or_default(),
            EntryValue::EmptyFile => "",
        }
    }
}

impl From<&str> for EntryValue {
    fn from(value: &str) -> Self {
        EntryValue::String(value.to_string())
    }
}
impl From<PathBuf> for EntryValue {
    fn from(value: PathBuf) -> Self {
        EntryValue::File(value)
    }
}

/// A default noop NetProvider
#[derive(Default)]
pub struct DummyNetProvider;
impl NetProvider for DummyNetProvider {
    fn fetch(&self, _doc_id: usize, _request: Request, _handler: Box<dyn NetHandler>) {}
    fn is_noop(&self) -> bool {
        true
    }
}

/// The AbortController interface represents a controller object that
/// allows you to abort one or more Web requests as and when desired.
///
/// <https://developer.mozilla.org/en-US/docs/Web/API/AbortController>
#[derive(Debug, Default)]
pub struct AbortController {
    pub signal: AbortSignal,
}

impl AbortController {
    /// The abort() method of the AbortController interface aborts
    /// an asynchronous operation before it has completed.
    /// This is able to abort fetch requests.
    ///
    /// <https://developer.mozilla.org/en-US/docs/Web/API/AbortController/abort>
    pub fn abort(self) {
        self.signal.0.aborted.store(true, Ordering::SeqCst);
        // Never execute or drop application wakers while the registry is locked.
        let waiters = std::mem::take(&mut *self.signal.0.waiters.lock().unwrap());
        for waker in waiters.into_iter().flatten() {
            waker.wake();
        }
    }
}

/// The AbortSignal interface represents a signal object that allows you to
/// communicate with an asynchronous operation (such as a fetch request) and
/// abort it if required via an AbortController object.
///
/// <https://developer.mozilla.org/en-US/docs/Web/API/AbortSignal>
#[derive(Debug, Default, Clone)]
pub struct AbortSignal(Arc<AbortState>);

#[derive(Debug, Default)]
struct AbortState {
    aborted: AtomicBool,
    waiters: std::sync::Mutex<Vec<Option<std::task::Waker>>>,
}

/// A cancellation waiter that unregisters itself when its request finishes.
pub struct AbortFuture {
    signal: AbortSignal,
    slot: Option<usize>,
}

impl std::future::Future for AbortFuture {
    type Output = ();
    fn poll(
        mut self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<()> {
        let waker = cx.waker().clone();
        let signal = self.signal.clone();
        let mut waiters = signal.0.waiters.lock().unwrap();
        if signal.aborted() {
            drop(waiters);
            return std::task::Poll::Ready(());
        }
        let slot = match self.slot {
            Some(slot) => slot,
            None => {
                let slot = waiters
                    .iter()
                    .position(Option::is_none)
                    .unwrap_or(waiters.len());
                if slot == waiters.len() {
                    waiters.push(None);
                }
                self.slot = Some(slot);
                slot
            }
        };
        let old = waiters[slot].replace(waker);
        drop(waiters);
        drop(old);
        std::task::Poll::Pending
    }
}

impl Drop for AbortFuture {
    fn drop(&mut self) {
        if let Some(slot) = self.slot {
            let mut waiters = self.signal.0.waiters.lock().unwrap();
            let old = waiters.get_mut(slot).and_then(Option::take);
            while waiters.last().is_some_and(Option::is_none) {
                waiters.pop();
            }
            drop(waiters);
            drop(old);
        }
    }
}

impl AbortSignal {
    /// The aborted read-only property returns a value that indicates whether
    /// the asynchronous operations the signal is communicating with are
    /// aborted (true) or not (false).
    ///
    /// <https://developer.mozilla.org/en-US/docs/Web/API/AbortSignal/aborted>
    pub fn aborted(&self) -> bool {
        self.0.aborted.load(Ordering::SeqCst)
    }

    /// Wake a pending operation immediately when this signal is aborted.
    pub fn cancelled(&self) -> AbortFuture {
        AbortFuture {
            signal: self.clone(),
            slot: None,
        }
    }
}
