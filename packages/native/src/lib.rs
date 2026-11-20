#![cfg_attr(docsrs, feature(doc_cfg))]

//! Native window renderer for the **Aether toolkit**.
//!
//! Aether is a QualiaDB-native GUI toolkit (the Dioxus / Tauri analogue for
//! Webizen apps). This crate is not Webizen, not QualiaDB, and not a shipped
//! application. Host apps (`webizen-os-shell`, Synth operator UI, later Qaps)
//! consume it. Do not ship Aether window titles, tray items, or default icons
//! as product identity.
//!
//! Two webs: this crate paints **human-centric** native RSX (QualiaDB/VibeScript
//! hosts). **Chora** in Qualia is the spatial commons canvas, not this renderer.
//! Old-web https is the Webizen Web Browser Qap, using the Servo-derived MPL
//! engine as a copied-frame worker (`feature = "chora"` is a code id, B1b).
//! Not WebView, not Wry. B1a leaves that feature **off**. `Document::eval` may
//! remain `NoOpDocument`.
//!
//! ## Feature flags
//!  - `default`: Enables the features listed below.
//!  - `accessibility`: Enables [`accesskit`](https://docs.rs/accesskit/latest/accesskit/) accessibility support.
//!  - `hot-reload`: Enables hot-reloading of Aether RSX.
//!  - `menu`: Enables the [`muda`](https://docs.rs/muda/latest/muda/) menubar.
//!  - `tracing`: Enables tracing support.

mod aether_application;
mod aether_renderer;
mod assets;
mod config;
#[cfg(all(feature = "chora", not(target_arch = "wasm32")))]
pub mod chora_frame;
#[cfg(all(feature = "chora", not(target_arch = "wasm32")))]
pub use aether_chora_client;
mod contexts;
mod event_handlers;
mod hooks;
mod link_handler;

#[cfg(feature = "prelude")]
pub mod prelude;

#[doc(inline)]
pub use aether_native_dom::*;
#[cfg(all(feature = "net", not(target_arch = "wasm32")))]
use blitz_traits::net::NetProvider;

pub use aether_application::{AetherNativeApplication, AetherNativeEvent};
pub use aether_renderer::{AetherNativeWindowRenderer, RendererOptions};
use assets::AetherNativeNetProvider;
pub use assets::AssetAccess;

#[doc(inline)]
pub use anyrender::CompositeAlphaMode;
#[doc(inline)]
pub use peniko::Color;

#[cfg(target_os = "android")]
#[cfg_attr(docsrs, doc(cfg(target_os = "android")))]
/// Set the current [`AndroidApp`](android_activity::AndroidApp).
pub fn set_android_app(app: android_activity::AndroidApp) {
    blitz_shell::set_android_app(app);
}

#[cfg(target_os = "android")]
#[cfg_attr(docsrs, doc(cfg(target_os = "android")))]
/// Get the current [`AndroidApp`](android_activity::AndroidApp).
/// This will panic if the android activity has not been setup with [`set_android_app`].
pub fn current_android_app() -> android_activity::AndroidApp {
    blitz_shell::current_android_app()
}

#[cfg(target_os = "android")]
#[cfg_attr(docsrs, doc(cfg(target_os = "android")))]
pub use android_activity::AndroidApp;

#[cfg(any(feature = "vello", feature = "vello-hybrid"))]
pub use {
    aether_renderer::{Features, Limits},
    etalon_context::DeviceHandle,
};

pub use blitz_dom::{FontContext, Widget, build_single_font_ctx};
pub use config::Config;
pub use event_handlers::WinitEventHandlerId;
pub use hooks::{use_back_button, use_window_event};
pub use winit;
pub use winit::dpi::{LogicalSize, PhysicalSize};
pub use winit::window::WindowAttributes;

use aether_core::{ComponentFunction, Element, VirtualDom, consume_context, use_hook};
use blitz_shell::{BlitzShellEvent, BlitzShellProxy, WindowConfig};
use link_handler::AetherNativeNavigationProvider;
use std::any::Any;
use std::sync::Arc;
use winit::{
    error::EventLoopError,
    event_loop::{ControlFlow, EventLoop},
    raw_window_handle::{HasWindowHandle as _, RawWindowHandle},
    window::Window,
};

pub fn use_window() -> Arc<dyn Window> {
    use_hook(consume_context::<Arc<dyn Window>>)
}

pub fn use_raw_window_handle() -> RawWindowHandle {
    use_hook(|| {
        consume_context::<Arc<dyn Window>>()
            .window_handle()
            .unwrap()
            .as_raw()
    })
}

/// A returned failure while creating or running a native application.
///
/// These variants preserve the underlying error and the launch stage. They do
/// not represent panics inside application callbacks, window creation, or GPU
/// initialization; see [`try_launch_cfg_with_props`] for the remaining limits.
#[derive(Debug)]
#[non_exhaustive]
pub enum LaunchError {
    /// Failed to create the networking runtime (`net` on native targets).
    Runtime(std::io::Error),
    /// Winit returned an error while creating the event loop.
    EventLoopCreation(EventLoopError),
    /// Winit returned an error while running the application.
    EventLoopRun(EventLoopError),
}

impl std::fmt::Display for LaunchError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Runtime(error) => {
                write!(f, "failed to create native networking runtime: {error}")
            }
            Self::EventLoopCreation(error) => {
                write!(f, "failed to create native event loop: {error}")
            }
            Self::EventLoopRun(error) => write!(f, "native event loop failed: {error}"),
        }
    }
}

impl std::error::Error for LaunchError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Runtime(error) => Some(error),
            Self::EventLoopCreation(error) | Self::EventLoopRun(error) => Some(error),
        }
    }
}

// Equivalent to Blitz's default builder setup, with its build error preserved.
// Keep Android activity forwarding and Wait control flow aligned with Blitz.
fn try_create_default_event_loop() -> Result<EventLoop, LaunchError> {
    let mut builder = EventLoop::builder();
    #[cfg(target_os = "android")]
    {
        use winit::platform::android::EventLoopBuilderExtAndroid;
        builder.with_android_app(current_android_app());
    }
    let event_loop = builder.build().map_err(LaunchError::EventLoopCreation)?;
    event_loop.set_control_flow(ControlFlow::Wait);
    Ok(event_loop)
}

/// Launch an interactive HTML/CSS renderer driven by the Aether virtualdom.
///
/// Panics on returned launch errors. Use [`try_launch`] to handle those errors.
pub fn launch(app: fn() -> Element) {
    launch_cfg(app, vec![], vec![])
}

/// Launch an application, returning runtime and event-loop errors.
///
/// See [`try_launch_cfg_with_props`] for platform and panic limitations.
pub fn try_launch(app: fn() -> Element) -> Result<(), LaunchError> {
    try_launch_cfg(app, vec![], vec![])
}

/// Launch with contexts and configuration, panicking on returned launch errors.
pub fn launch_cfg(
    app: fn() -> Element,
    contexts: Vec<Box<dyn Fn() -> Box<dyn Any> + Send + Sync>>,
    cfg: Vec<Box<dyn Any>>,
) {
    launch_cfg_with_props(app, (), contexts, cfg)
}

/// Launch with contexts and configuration, returning runtime/event-loop errors.
///
/// See [`try_launch_cfg_with_props`] for platform and panic limitations.
pub fn try_launch_cfg(
    app: fn() -> Element,
    contexts: Vec<Box<dyn Fn() -> Box<dyn Any> + Send + Sync>>,
    cfg: Vec<Box<dyn Any>>,
) -> Result<(), LaunchError> {
    try_launch_cfg_with_props(app, (), contexts, cfg)
}

// todo: props shouldn't have the clone bound - should try and match aether-desktop behavior
/// Launch with component props, panicking on returned launch errors.
pub fn launch_cfg_with_props<P: Clone + 'static, M: 'static>(
    app: impl ComponentFunction<P, M>,
    props: P,
    contexts: Vec<Box<dyn Fn() -> Box<dyn Any> + Send + Sync>>,
    configs: Vec<Box<dyn Any>>,
) {
    try_launch_cfg_with_props(app, props, contexts, configs)
        .expect("failed to launch Aether native application")
}

/// Launch with component props, returning failures exposed by the underlying APIs.
///
/// Returns [`LaunchError`] for event-loop creation, native networking-runtime
/// creation, and event-loop execution errors. The normal return behavior follows
/// Winit's platform event loop; on some targets control never returns to the caller.
/// On the web, returning success only means the event handler was registered;
/// subsequent asynchronous window or rendering failures are not returned here.
///
/// # Panics
///
/// Winit's main-thread requirements still apply. Android callers must first set
/// the activity with `set_android_app` (available on Android). Upstream window
/// and GPU initialization APIs can panic rather than return errors, and component
/// or context callbacks can also panic. Those failures are not converted into
/// [`LaunchError`]. This API does not provide device recovery or report deferred
/// GPU initialization failures through its return value.
pub fn try_launch_cfg_with_props<P: Clone + 'static, M: 'static>(
    app: impl ComponentFunction<P, M>,
    props: P,
    contexts: Vec<Box<dyn Fn() -> Box<dyn Any> + Send + Sync>>,
    configs: Vec<Box<dyn Any>>,
) -> Result<(), LaunchError> {
    // Macro to attempt to downcast a type out of a Box<dyn Any>
    macro_rules! try_read_config {
        ($input:ident, $store:ident, $kind:ty) => {
            // Try to downcast the Box<dyn Any> to type $kind
            match $input.downcast::<$kind>() {
                // If the type matches then write downcast value to variable $store
                Ok(value) => {
                    $store = Some(*value);
                    continue;
                }
                // Else extract the original Box<dyn Any> value out of the error type
                // and return it so that we can try again with a different type.
                Err(cfg) => cfg,
            }
        };
    }

    // Read config values
    #[cfg(any(feature = "vello", feature = "vello-hybrid"))]
    let (mut features, mut limits) = (None, None);
    let mut window_attributes = None;
    let mut config = None;
    for mut cfg in configs {
        #[cfg(any(feature = "vello", feature = "vello-hybrid"))]
        {
            cfg = try_read_config!(cfg, features, Features);
            cfg = try_read_config!(cfg, limits, Limits);
        }
        cfg = try_read_config!(cfg, window_attributes, WindowAttributes);
        cfg = try_read_config!(cfg, config, Config);
        let _ = cfg;
    }

    let mut config = config.unwrap_or_default();
    if let Some(window_attributes) = window_attributes {
        config.window_attributes = window_attributes;
    }
    let event_loop = try_create_default_event_loop()?;
    let winit_proxy = event_loop.create_proxy();
    let (proxy, event_queue) = BlitzShellProxy::new(winit_proxy);

    // Turn on the runtime and enter it
    #[cfg(feature = "net")]
    #[cfg(not(target_arch = "wasm32"))]
    let rt = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .map_err(LaunchError::Runtime)?;
    #[cfg(feature = "net")]
    #[cfg(not(target_arch = "wasm32"))]
    let _guard = rt.enter();

    // Setup hot-reloading if enabled.
    #[cfg(all(feature = "hot-reload", debug_assertions))]
    #[cfg(not(target_arch = "wasm32"))]
    {
        let proxy = proxy.clone();
        aether_devtools::connect(move |event| {
            let dxn_event = AetherNativeEvent::DevserverEvent(event);
            proxy.send_event(BlitzShellEvent::embedder_event(dxn_event));
        })
    }

    // Build the vdom first; the net provider, document, and other window-bound
    // contexts are attached below once the event-loop proxy exists.
    let mut vdom = VirtualDom::new_with_props(app, props);

    for context in contexts {
        vdom.insert_any_root_context(context());
    }

    #[cfg(all(feature = "net", not(target_arch = "wasm32")))]
    let net_provider = {
        let net_waker = Some(Arc::new(proxy.clone()) as _);
        let inner_net_provider = Arc::new(blitz_net::Provider::new(net_waker));
        vdom.provide_root_context(Arc::clone(&inner_net_provider));

        Arc::new(
            AetherNativeNetProvider::with_inner(proxy.clone(), inner_net_provider as _)
                .with_asset_access(config.asset_access)
                .with_asset_max_bytes(config.asset_max_bytes),
        ) as Arc<dyn NetProvider>
    };

    #[cfg(any(not(feature = "net"), target_arch = "wasm32"))]
    let net_provider = AetherNativeNetProvider::shared_with_access(
        proxy.clone(),
        config.asset_access,
        config.asset_max_bytes,
    );

    vdom.provide_root_context(Arc::clone(&net_provider));

    #[cfg(feature = "html")]
    let html_parser_provider = {
        let html_parser = Arc::new(blitz_html::HtmlProvider) as _;
        vdom.provide_root_context(Arc::clone(&html_parser));
        Some(html_parser)
    };
    #[cfg(not(feature = "html"))]
    let html_parser_provider = None;

    let navigation_provider = Some(Arc::new(AetherNativeNavigationProvider) as _);

    // Create document + window from the baked virtualdom
    let doc = AetherDocument::new(
        vdom,
        DocumentConfig {
            net_provider: Some(net_provider),
            html_parser_provider,
            navigation_provider,
            font_ctx: config.font_ctx,
            ..Default::default()
        },
    );
    #[cfg(any(feature = "vello", feature = "vello-hybrid"))]
    let renderer = AetherNativeWindowRenderer::with_options(RendererOptions {
        base_color: config.base_color,
        alpha_mode: config.alpha_mode,
        features,
        limits,
    });
    #[cfg(not(any(feature = "vello", feature = "vello-hybrid")))]
    let renderer = AetherNativeWindowRenderer::with_options(RendererOptions {
        base_color: config.base_color,
        alpha_mode: config.alpha_mode,
    });
    let config =
        WindowConfig::with_attributes(Box::new(doc) as _, renderer, config.window_attributes);

    // Create application
    let application = AetherNativeApplication::new(proxy, event_queue, config);

    // Run event loop
    event_loop
        .run_app(application)
        .map_err(LaunchError::EventLoopRun)
}
