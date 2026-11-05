#![doc = include_str!("../README.md")]
//!
//! ## Aether Crate Features
//!
//! This crate has several features that can be enabled to change the active renderer and enable various integrations:
//!
//! - `signals`: (default) re-exports `aether-signals`
//! - `macro`: (default) re-exports `aether-macro`
//! - `html`: (default) exports `aether-html` as the default elements to use in rsx
//! - `hooks`: (default) re-exports `aether-hooks`
//! - `hot-reload`: (default) enables hot rsx reloading in all renderers that support it
//! - `router`: exports the [router](https://aetherlabs.com/learn/0.7/essentials/router/) and enables any router features for the current platform
//! - `third-party-renderer`: Just disables warnings about no active platform when no renderers are enabled
//! - `logger`: Enable the default tracing subscriber for Aether apps
//!
//! Platform features (the current platform determines what platform the [`launch()`] function runs):
//!
//! - `fullstack`: enables the fullstack platform. This must be used in combination with the `web` feature for wasm builds and `server` feature for server builds
//! - `desktop`: enables the desktop platform
//! - `mobile`: enables the mobile platform
//! - `web`: enables the web platform. If the fullstack platform is enabled, this will set the fullstack platform to client mode
//! - `liveview`: enables the liveview platform
//! - `server`: enables the server variant of aether
#![doc(html_logo_url = "https://avatars.githubusercontent.com/u/79236386")]
#![doc(html_favicon_url = "https://avatars.githubusercontent.com/u/79236386")]
#![cfg_attr(docsrs, feature(doc_cfg))]

pub use aether_core;
#[doc(inline)]
pub use aether_core::{CapturedError, Ok, Result};

#[cfg(feature = "launch")]
#[cfg_attr(docsrs, doc(cfg(feature = "launch")))]
mod launch;

pub use aether_core as core;

#[cfg(feature = "launch")]
#[cfg_attr(docsrs, doc(cfg(feature = "launch")))]
pub use crate::launch::*;

#[cfg(feature = "hooks")]
#[cfg_attr(docsrs, doc(cfg(feature = "hooks")))]
pub use aether_hooks as hooks;

#[cfg(feature = "signals")]
#[cfg_attr(docsrs, doc(cfg(feature = "signals")))]
pub use aether_signals as signals;

#[cfg(feature = "signals")]
#[cfg_attr(docsrs, doc(cfg(feature = "signals")))]
pub use aether_stores as stores;

pub mod events {
    #[cfg(feature = "html")]
    #[cfg_attr(docsrs, doc(cfg(feature = "html")))]
    pub use aether_html::events::*;
}

#[cfg(feature = "document")]
#[cfg_attr(docsrs, doc(cfg(feature = "document")))]
pub use aether_document as document;

#[cfg(feature = "document")]
#[cfg_attr(docsrs, doc(cfg(feature = "document")))]
pub use aether_history as history;

#[cfg(feature = "html")]
#[cfg_attr(docsrs, doc(cfg(feature = "html")))]
pub use aether_html as html;

#[cfg(feature = "macro")]
#[cfg_attr(docsrs, doc(cfg(feature = "macro")))]
pub use aether_core_macro as core_macro;

#[cfg(feature = "logger")]
#[cfg_attr(docsrs, doc(cfg(feature = "logger")))]
pub use aether_logger as logger;

#[cfg(feature = "cli-config")]
#[cfg_attr(docsrs, doc(cfg(feature = "cli-config")))]
pub use aether_cli_config as cli_config;

#[cfg(feature = "server")]
#[cfg_attr(docsrs, doc(cfg(feature = "server")))]
pub use aether_server as server;

#[cfg(feature = "server")]
pub use aether_server::serve;

#[cfg(feature = "devtools")]
#[cfg_attr(docsrs, doc(cfg(feature = "devtools")))]
pub use aether_devtools as devtools;

#[cfg(feature = "web")]
#[cfg_attr(docsrs, doc(cfg(feature = "web")))]
pub use aether_web as web;

#[cfg(feature = "router")]
#[cfg_attr(docsrs, doc(cfg(feature = "router")))]
pub use aether_router as router;

#[cfg(feature = "fullstack")]
#[cfg_attr(docsrs, doc(cfg(feature = "fullstack")))]
pub use aether_fullstack as fullstack;

#[cfg(feature = "desktop")]
#[cfg_attr(docsrs, doc(cfg(feature = "desktop")))]
pub use aether_desktop as desktop;

#[cfg(feature = "mobile")]
#[cfg_attr(docsrs, doc(cfg(feature = "mobile")))]
pub use aether_desktop as mobile;

#[cfg(feature = "native")]
#[cfg_attr(docsrs, doc(cfg(feature = "native")))]
pub use aether_native as native;

#[cfg(feature = "liveview")]
#[cfg_attr(docsrs, doc(cfg(feature = "liveview")))]
pub use aether_liveview as liveview;

#[cfg(feature = "ssr")]
#[cfg_attr(docsrs, doc(cfg(feature = "ssr")))]
pub use aether_ssr as ssr;

#[cfg(feature = "warnings")]
#[cfg_attr(docsrs, doc(cfg(feature = "warnings")))]
pub use warnings;

pub use aether_config_macros as config_macros;

#[cfg(feature = "wasm-split")]
#[cfg_attr(docsrs, doc(cfg(feature = "wasm-split")))]
pub use wasm_splitter as wasm_split;

pub use subsecond;

#[cfg(feature = "asset")]
#[cfg_attr(docsrs, doc(cfg(feature = "asset")))]
#[doc(inline)]
pub use aether_asset_resolver as asset_resolver;

pub mod prelude {
    pub use aether_config_macros;

    #[cfg(feature = "document")]
    #[cfg_attr(docsrs, doc(cfg(feature = "document")))]
    #[doc(inline)]
    pub use aether_document::{self as document, Meta, Stylesheet, Title};

    #[cfg(feature = "document")]
    #[cfg_attr(docsrs, doc(cfg(feature = "document")))]
    #[doc(inline)]
    pub use aether_history::{History, history};

    #[cfg(feature = "launch")]
    #[cfg_attr(docsrs, doc(cfg(feature = "launch")))]
    #[doc(inline)]
    pub use crate::launch::*;

    #[cfg(feature = "hooks")]
    #[cfg_attr(docsrs, doc(cfg(feature = "hooks")))]
    #[doc(inline)]
    pub use crate::hooks::*;

    #[cfg(feature = "signals")]
    #[cfg_attr(docsrs, doc(cfg(feature = "signals")))]
    #[doc(inline)]
    pub use aether_signals::{self, *};

    #[cfg(feature = "signals")]
    pub use aether_stores::{self, GlobalStore, ReadStore, Store, WriteStore, store, use_store};

    #[cfg(feature = "macro")]
    #[cfg_attr(docsrs, doc(cfg(feature = "macro")))]
    #[allow(deprecated)]
    #[doc(inline)]
    pub use aether_core_macro::{Props, component, rsx};

    #[cfg(feature = "launch")]
    #[cfg_attr(docsrs, doc(cfg(feature = "launch")))]
    pub use aether_config_macro::*;

    // The element-vocabulary root: `html::div`, `html::main`, etc. resolve as associated
    // consts on this type. The crate module is still reachable as the fully-qualified
    // `aether::html` for `aether::html::define_elements!` and friends.
    #[cfg(feature = "html")]
    #[cfg_attr(docsrs, doc(cfg(feature = "html")))]
    pub use aether_html::{self, html};

    #[cfg(feature = "html")]
    #[cfg_attr(docsrs, doc(cfg(feature = "html")))]
    #[doc(inline)]
    pub use aether_html::{Code, Key, Location, Modifiers, NamedKey};

    // Element associated-const traits (`html::div`, `html::main`, …) come from the single
    // generated `prelude` module, not from flattening every per-element module into scope.
    #[cfg(feature = "html")]
    #[cfg_attr(docsrs, doc(cfg(feature = "html")))]
    #[doc(no_inline)]
    pub use aether_html::{
        GlobalAttributesExtension, SvgAttributesExtension, elements::prelude::*, events::*,
        extensions::*, keyboard_types, traits::*,
    };

    #[cfg(feature = "desktop")]
    #[cfg_attr(docsrs, doc(cfg(feature = "desktop")))]
    #[doc(inline)]
    pub use aether_desktop::{Window, WindowConfig, WindowProps};

    #[cfg(feature = "devtools")]
    #[cfg_attr(docsrs, doc(cfg(feature = "devtools")))]
    pub use aether_devtools;

    pub use aether_core;

    #[cfg(feature = "fullstack")]
    #[cfg_attr(docsrs, doc(cfg(feature = "fullstack")))]
    #[doc(inline)]
    pub use aether_fullstack::{
        self as aether_fullstack, HttpError, OrHttpError, ServerFnError, ServerFnResult,
        StatusCode, delete, get, patch, post, put, server, use_loader, use_server_cached,
        use_server_future,
    };

    #[cfg(feature = "server")]
    #[cfg_attr(docsrs, doc(cfg(feature = "server")))]
    #[doc(inline)]
    pub use aether_server::{self, AetherRouterExt, ServeConfig, ServerFunction, serve};

    #[cfg(feature = "router")]
    #[cfg_attr(docsrs, doc(cfg(feature = "router")))]
    pub use aether_router;

    #[cfg(feature = "router")]
    #[cfg_attr(docsrs, doc(cfg(feature = "router")))]
    #[doc(inline)]
    pub use aether_router::{
        GoBackButton, GoForwardButton, Link, NavigationTarget, Outlet, Routable, Router, hooks::*,
        navigator, use_navigator,
    };

    #[cfg(feature = "asset")]
    #[cfg_attr(docsrs, doc(cfg(feature = "asset")))]
    #[doc(inline)]
    pub use manganis::{self, *};

    #[cfg(feature = "wasm-split")]
    #[cfg_attr(docsrs, doc(cfg(feature = "wasm-split")))]
    pub use wasm_splitter as wasm_split;

    #[doc(inline)]
    pub use aether_core::{
        AnyhowContext, Attribute, Callback, Component, ComponentFunctionExt, Element,
        ErrorBoundary, ErrorContext, Event, EventHandler, Fragment, HasAttributes, IntoDynNode,
        Portal, PortalProps, RenderError, RenderTargetId, Result, ScopeId, SuspenseBoundary,
        SuspenseContext, VNode, VirtualDom, consume_context, provide_context, spawn, suspend,
        try_consume_context, use_drop, use_hook,
    };

    #[cfg(feature = "logger")]
    pub use aether_logger::tracing::{debug, error, info, trace, warn};
}
