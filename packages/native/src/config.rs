use aether_core::LaunchConfig;
use anyrender::CompositeAlphaMode;
use blitz_dom::FontContext;
use peniko::Color;
use winit::window::WindowAttributes;

use crate::{RendererFallbackPolicy, WindowClosePolicy, WindowRole};

/// Launch-time configuration for a aether-native application.
pub struct Config {
    pub(crate) asset_access: crate::AssetAccess,
    pub(crate) asset_max_bytes: usize,
    pub(crate) window_attributes: WindowAttributes,
    pub(crate) font_ctx: Option<FontContext>,
    pub(crate) alpha_mode: Option<CompositeAlphaMode>,
    pub(crate) base_color: Option<Color>,
    pub(crate) role: WindowRole,
    pub(crate) close_policy: WindowClosePolicy,
    pub(crate) fallback_policy: RendererFallbackPolicy,
}

impl LaunchConfig for Config {}

impl Default for Config {
    fn default() -> Self {
        let window_attributes = WindowAttributes::default()
            .with_title(aether_cli_config::app_title().unwrap_or_default());

        // On WASM, append the canvas to the document body. Surface size is
        // intentionally not seeded: winit-web's with_surface_size writes inline
        // canvas.style.width/height that overrides host CSS. View::init reads
        // the canvas's CSS layout box when winit reports a 0×0 initial size.
        // To target an existing `<canvas>`, replace these attributes via
        // [`Config::with_window_attributes`].
        #[cfg(target_arch = "wasm32")]
        let window_attributes = {
            use winit::platform::web::WindowAttributesWeb;
            window_attributes.with_platform_attributes(Box::new(
                WindowAttributesWeb::default().with_append(true),
            ))
        };

        Self {
            asset_max_bytes: aether_asset_resolver::native::DEFAULT_ASSET_MAX_BYTES,
            // Match Manganis's source-path contract for ordinary `cargo run`.
            // Packaged applications use contained assets; remote-content hosts must
            // explicitly select Packaged even during development.
            asset_access: if aether_core::is_bundled_app() {
                crate::AssetAccess::Packaged
            } else {
                crate::AssetAccess::TrustedApplicationFiles
            },
            window_attributes,
            font_ctx: None,
            alpha_mode: None,
            base_color: None,
            role: WindowRole::default(),
            close_policy: WindowClosePolicy::default(),
            fallback_policy: RendererFallbackPolicy::default(),
        }
    }
}

impl Config {
    pub fn new() -> Self {
        Self::default()
    }

    /// Select local asset authority. Packaged apps default to `Packaged`; ordinary
    /// unbundled `cargo run` uses trusted application source files like Manganis.
    /// A directly launched unbundled binary with Manganis placeholder paths can
    /// explicitly select `TrustedApplicationFiles` to retain source-file assets.
    /// Hosts rendering untrusted content must explicitly use `Packaged` in all builds.
    pub fn with_asset_access(mut self, access: crate::AssetAccess) -> Self {
        self.asset_access = access;
        self
    }

    /// Bound each local asset response. Defaults to 64 MiB; increase explicitly
    /// for admitted large assets. This does not bound aggregate or decoder memory.
    pub fn with_asset_max_bytes(mut self, max_bytes: usize) -> Self {
        self.asset_max_bytes = max_bytes;
        self
    }

    /// Set the configuration for the window.
    pub fn with_window_attributes(mut self, attrs: WindowAttributes) -> Self {
        self.window_attributes = attrs;
        self
    }

    /// Select the window's presentation role.
    pub fn with_window_role(mut self, role: WindowRole) -> Self {
        self.role = role;
        self
    }

    /// Select the behavior for both OS and application close requests.
    pub fn with_close_policy(mut self, policy: WindowClosePolicy) -> Self {
        self.close_policy = policy;
        self
    }

    /// Select how the renderer handles an initialization or device failure.
    pub fn with_renderer_fallback(mut self, policy: RendererFallbackPolicy) -> Self {
        self.fallback_policy = policy;
        self
    }

    /// Set a custom [`FontContext`] for the document.
    ///
    /// On WASM, browsers don't expose system fonts, so a `FontContext` with
    /// bundled fonts must be provided. Use [`crate::build_single_font_ctx`] for
    /// the common one-font case.
    pub fn with_font_ctx(mut self, font_ctx: FontContext) -> Self {
        self.font_ctx = Some(font_ctx);
        self
    }

    /// Set the alpha mode used when compositing the window surface.
    ///
    /// This controls how the rendered content is blended with the background,
    /// e.g. to enable transparent windows. Not all renderers support every
    /// mode; unsupported modes are ignored by the active renderer.
    pub fn with_alpha_mode(mut self, alpha_mode: CompositeAlphaMode) -> Self {
        self.alpha_mode = Some(alpha_mode);
        self
    }

    /// Set the base (background) color used to clear each frame.
    pub fn with_base_color(mut self, base_color: Color) -> Self {
        self.base_color = Some(base_color);
        self
    }
}
