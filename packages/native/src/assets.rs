use blitz_shell::BlitzShellProxy;
use std::sync::Arc;

use blitz_traits::net::{CompletionHandler, NetError, NetHandler, NetProvider, Request};

#[path = "asset_access.rs"]
mod asset_access;
pub use asset_access::AssetAccess;

pub struct AetherNativeNetProvider {
    asset_access: AssetAccess,
    max_bytes: usize,
    inner_net_provider: Option<Arc<dyn NetProvider + 'static>>,
}

#[allow(unused)]
impl AetherNativeNetProvider {
    pub fn shared(proxy: BlitzShellProxy) -> Arc<dyn NetProvider> {
        Arc::new(Self::new(proxy)) as Arc<dyn NetProvider>
    }

    pub fn new(proxy: BlitzShellProxy) -> Self {
        #[cfg(any(
            feature = "data-uri",
            all(feature = "net", not(target_arch = "wasm32"))
        ))]
        let net_waker = Some(Arc::new(proxy) as _);

        #[cfg(all(feature = "net", not(target_arch = "wasm32")))]
        let inner_net_provider = Some(blitz_net::Provider::shared(net_waker.clone()));
        #[cfg(all(
            feature = "data-uri",
            any(not(feature = "net"), target_arch = "wasm32")
        ))]
        let inner_net_provider = Some(blitz_shell::DataUriNetProvider::shared(net_waker.clone()));
        #[cfg(all(
            not(feature = "data-uri"),
            any(not(feature = "net"), target_arch = "wasm32")
        ))]
        let inner_net_provider = None;

        Self {
            inner_net_provider,
            max_bytes: aether_asset_resolver::native::DEFAULT_ASSET_MAX_BYTES,
            asset_access: AssetAccess::Packaged,
        }
    }

    pub fn with_inner(proxy: BlitzShellProxy, inner: Arc<dyn NetProvider>) -> Self {
        Self {
            inner_net_provider: Some(inner),
            max_bytes: aether_asset_resolver::native::DEFAULT_ASSET_MAX_BYTES,
            asset_access: AssetAccess::Packaged,
        }
    }

    pub fn with_asset_access(mut self, access: AssetAccess) -> Self {
        self.asset_access = access;
        self
    }

    pub fn with_asset_max_bytes(mut self, max_bytes: usize) -> Self {
        self.max_bytes = max_bytes;
        self
    }

    pub fn shared_with_access(
        proxy: BlitzShellProxy,
        access: AssetAccess,
        max_bytes: usize,
    ) -> Arc<dyn NetProvider> {
        Arc::new(
            Self::new(proxy)
                .with_asset_access(access)
                .with_asset_max_bytes(max_bytes),
        )
    }

    pub fn inner(&self) -> Option<&Arc<dyn NetProvider>> {
        self.inner_net_provider.as_ref()
    }
}

impl NetProvider for AetherNativeNetProvider {
    fn fetch(&self, doc_id: usize, request: Request, handler: Box<dyn NetHandler>) {
        // Even a legacy delegated transport that drops its handler completes the
        // DOM request, and a late success cannot escape an observed cancellation.
        let handler = CompletionHandler::with_signal(
            request.url.to_string(),
            request.signal.clone(),
            handler,
        );
        if request
            .signal
            .as_ref()
            .is_some_and(|signal| signal.aborted())
        {
            handler.failed(request.url.to_string(), NetError::Cancelled);
            return;
        }
        if request.url.scheme() == "aether" {
            #[allow(clippy::single_match)] // cfg'd code
            let response = match self.asset_access {
                AssetAccess::Packaged => aether_asset_resolver::native::serve_asset_with_limit(
                    request.url.path(),
                    self.max_bytes,
                ),
                AssetAccess::TrustedApplicationFiles => {
                    aether_asset_resolver::native::serve_trusted_asset_with_limit(
                        request.url.path(),
                        self.max_bytes,
                    )
                }
            };
            match response {
                Ok(res) if res.status().is_success() => {
                    #[cfg(feature = "tracing")]
                    tracing::trace!("fetching asset from file system success {request:#?}");
                    handler.bytes(request.url.to_string(), res.into_body().into())
                }
                Ok(res) => {
                    handler.failed(
                        request.url.to_string(),
                        NetError::HttpStatus(res.status().as_u16()),
                    );
                }
                Err(error) => {
                    #[cfg(feature = "tracing")]
                    tracing::warn!("fetching asset from file system error {request:#?}");
                    handler.failed(
                        request.url.to_string(),
                        NetError::Transport(error.to_string()),
                    );
                }
            }
        } else if !self
            .asset_access
            .allows_delegated_scheme(request.url.scheme())
        {
            // Do not let the contained profile acquire file or host-service authority
            // merely because an inner transport happens to implement another scheme.
            #[cfg(feature = "tracing")]
            tracing::warn!(
                "asset transport scheme is not allowed: {}",
                request.url.scheme()
            );
            handler.failed(request.url.to_string(), NetError::AccessDenied);
        } else if let Some(inner) = &self.inner_net_provider {
            inner.fetch(doc_id, request, handler);
        } else {
            #[cfg(feature = "tracing")]
            tracing::warn!("net feature not enabled, cannot fetch {request:#?}");
            handler.failed(request.url.to_string(), NetError::Unavailable);
        }
    }
}

#[cfg(test)]
#[path = "asset_transport_tests.rs"]
mod tests;
