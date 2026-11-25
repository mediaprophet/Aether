use std::rc::Rc;
use std::sync::Arc;
use std::{
    any::Any,
    cell::{Cell, RefCell},
};

use anyrender::{CompositeAlphaMode, RenderContext, RendererConfig, WindowRenderer};
use peniko::Color;

#[cfg(all(feature = "gpu-cpu-fallback", not(target_arch = "wasm32")))]
#[path = "renderer_fallback.rs"]
mod fallback;
#[cfg(all(feature = "gpu-cpu-fallback", not(target_arch = "wasm32")))]
pub use fallback::FallbackScenePainter;
#[cfg(all(feature = "gpu-cpu-fallback", not(target_arch = "wasm32")))]
use fallback::{CpuFallback, draw_recorded};

// Keep GPU configuration types available whenever their feature is enabled, even
// when a higher-priority CPU/Skia renderer overrides the default hybrid renderer.
#[cfg(any(feature = "vello", feature = "vello-hybrid"))]
pub use etalon::{Features, Limits};

// Renderer priority: Vello GPU, Vello CPU, Skia GPU, Skia raster, Vello hybrid.
// CPU/Skia features intentionally override the default hybrid renderer.
cfg_if::cfg_if! {
    if #[cfg(feature = "vello")] {
        pub use anyrender_vello::{
            VelloRendererOptions as InnerRendererOptions, VelloWindowRenderer as InnerRenderer,
        };
    } else if #[cfg(feature = "vello-cpu-base")] {
        use anyrender_vello_cpu::VelloCpuWindowRenderer as InnerRenderer;
    } else if #[cfg(feature = "skia")] {
        use anyrender_skia::SkiaWindowRenderer as InnerRenderer;
    } else if #[cfg(feature = "skia-raster-base")] {
        use anyrender_skia::raster::SkiaRasterWindowRenderer as InnerRenderer;
    } else if #[cfg(feature = "vello-hybrid")] {
        pub use anyrender_vello_hybrid::{
            VelloHybridRendererOptions as InnerRendererOptions, VelloHybridWindowRenderer as InnerRenderer,
        };
    } else {
        compile_error!("At least one renderer feature must be enabled");
    }
}

/// Policy for a failed GPU renderer.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum RendererFallbackPolicy {
    /// Permit the CPU renderer where the active backend supplies it.
    #[default]
    CpuOnFailure,
    /// Suppress painting after a reported renderer fault.
    FailClosed,
}

#[derive(Clone, Debug)]
pub struct RendererHealth {
    pub backend_name: &'static str,
    pub gpu_active: bool,
    pub faulted: bool,
    pub last_error: Option<String>,
}

/// Renderer configuration for [`AetherNativeWindowRenderer`].
///
/// Fields that only apply to the GPU-backed `vello`/`vello-hybrid` renderers
/// (`features`/`limits`) are only present when one of those features is enabled.
/// If CPU or Skia features override the default hybrid renderer, these fields
/// remain available for source compatibility but are ignored.
#[derive(Default)]
pub struct RendererOptions {
    /// Base (background) color used to clear each frame.
    pub base_color: Option<Color>,
    pub fallback_policy: RendererFallbackPolicy,
    /// Alpha mode used when compositing the window surface.
    pub alpha_mode: Option<CompositeAlphaMode>,
    /// Etalon features to request from the GPU adapter.
    #[cfg(any(feature = "vello", feature = "vello-hybrid"))]
    pub features: Option<Features>,
    /// Etalon limits to request from the GPU adapter.
    #[cfg(any(feature = "vello", feature = "vello-hybrid"))]
    pub limits: Option<Limits>,
}

#[derive(Clone)]
pub struct AetherNativeWindowRenderer {
    inner: Rc<RefCell<InnerRenderer>>,
    last_error: Rc<RefCell<Option<String>>>,
    active_backend: Rc<Cell<Option<ActiveBackend>>>,
    fallback_policy: RendererFallbackPolicy,
    #[cfg(all(feature = "gpu-cpu-fallback", not(target_arch = "wasm32")))]
    fallback: Rc<RefCell<CpuFallback>>,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum ActiveBackend {
    Primary,
    #[cfg(all(feature = "gpu-cpu-fallback", not(target_arch = "wasm32")))]
    Software,
}

impl Default for AetherNativeWindowRenderer {
    fn default() -> Self {
        Self::new()
    }
}

impl AetherNativeWindowRenderer {
    pub fn new() -> Self {
        Self::with_options(RendererOptions::default())
    }

    #[cfg(any(feature = "vello-hybrid", feature = "vello"))]
    pub fn with_features_and_limits(features: Option<Features>, limits: Option<Limits>) -> Self {
        Self::with_options(RendererOptions {
            features,
            limits,
            ..Default::default()
        })
    }

    /// Build a renderer from the given [`RendererOptions`].
    ///
    /// `base_color` and `alpha_mode` are forwarded to the active renderer via
    /// [`anyrender::RendererConfig`]; renderers that don't support a particular
    /// option simply ignore it. `features`/`limits` are only applied by the
    /// GPU-backed `vello`/`vello-hybrid` renderers.
    pub fn with_options(options: RendererOptions) -> Self {
        let mut config = RendererConfig::default();
        config.base_color = options.base_color;
        config.composite_alpha_mode = options.alpha_mode;

        #[cfg(all(feature = "gpu-cpu-fallback", not(target_arch = "wasm32")))]
        let fallback = Rc::new(RefCell::new(CpuFallback::new(config.clone())));
        cfg_if::cfg_if! {
            if #[cfg(any(
                feature = "vello",
                all(
                    feature = "vello-hybrid",
                    not(any(
                        feature = "vello-cpu-base",
                        feature = "skia",
                        feature = "skia-raster-base",
                    )),
                ),
            ))] {
                let mut inner_options: InnerRendererOptions = config.into();
                if let Some(features) = options.features {
                    inner_options = inner_options.features(features);
                }
                if let Some(limits) = options.limits {
                    inner_options = inner_options.limits(limits);
                }
                let inner_renderer = InnerRenderer::with_options(inner_options);
            } else {
                let inner_renderer = InnerRenderer::with_options(config);
            }
        }

        Self {
            inner: Rc::new(RefCell::new(inner_renderer)),
            last_error: Rc::new(RefCell::new(None)),
            active_backend: Rc::new(Cell::new(None)),
            fallback_policy: options.fallback_policy,
            #[cfg(all(feature = "gpu-cpu-fallback", not(target_arch = "wasm32")))]
            fallback,
        }
    }

    /// Snapshot the current renderer and its most recent reported fault.
    pub fn health(&self) -> RendererHealth {
        let last_error = self.last_error.borrow().clone();
        #[cfg(all(feature = "gpu-cpu-fallback", not(target_arch = "wasm32")))]
        if self.active_backend.get() == Some(ActiveBackend::Software) {
            return RendererHealth {
                backend_name: "Vello CPU (softbuffer)",
                gpu_active: false,
                faulted: last_error.is_some(),
                last_error,
            };
        }
        let (backend_name, gpu) = if cfg!(feature = "vello") {
            ("Vello", true)
        } else if cfg!(feature = "vello-cpu-base") {
            ("Vello CPU", false)
        } else if cfg!(feature = "skia") {
            ("Skia", true)
        } else if cfg!(feature = "skia-raster-base") {
            ("Skia raster", false)
        } else {
            ("Vello hybrid", true)
        };
        RendererHealth {
            backend_name,
            gpu_active: gpu
                && self.active_backend.get() == Some(ActiveBackend::Primary)
                && last_error.is_none(),
            faulted: last_error.is_some(),
            last_error,
        }
    }

    /// The configured window policy, independent of whether fallback has occurred.
    pub fn fallback_policy(&self) -> RendererFallbackPolicy {
        self.fallback_policy
    }

    /// Whether the renderer is currently painting through its software fallback.
    pub fn fallback_engaged(&self) -> bool {
        #[cfg(all(feature = "gpu-cpu-fallback", not(target_arch = "wasm32")))]
        {
            self.active_backend.get() == Some(ActiveBackend::Software)
        }
        #[cfg(not(all(feature = "gpu-cpu-fallback", not(target_arch = "wasm32"))))]
        {
            false
        }
    }

    /// Record a backend or device fault reported by an application integration.
    pub fn record_fault(&self, reason: impl Into<String>) {
        *self.last_error.borrow_mut() = Some(reason.into());
    }

    fn painting_allowed(&self) -> bool {
        self.fallback_policy != RendererFallbackPolicy::FailClosed
            || self.last_error.borrow().is_none()
    }
}

impl RenderContext for AetherNativeWindowRenderer {
    fn try_register_custom_resource(
        &mut self,
        resource: Box<dyn Any>,
    ) -> Result<anyrender::ResourceId, anyrender::RegisterResourceError> {
        self.inner
            .borrow_mut()
            .try_register_custom_resource(resource)
    }

    fn unregister_resource(&mut self, resource_id: anyrender::ResourceId) {
        self.inner.borrow_mut().unregister_resource(resource_id)
    }

    fn renderer_specific_context(&self) -> Option<Box<dyn Any>> {
        self.inner.try_borrow().ok()?.renderer_specific_context()
    }
}
impl WindowRenderer for AetherNativeWindowRenderer {
    #[cfg(all(feature = "gpu-cpu-fallback", not(target_arch = "wasm32")))]
    type ScenePainter<'a>
        = FallbackScenePainter<'a>
    where
        Self: 'a;
    #[cfg(not(all(feature = "gpu-cpu-fallback", not(target_arch = "wasm32"))))]
    type ScenePainter<'a>
        = <InnerRenderer as WindowRenderer>::ScenePainter<'a>
    where
        Self: 'a;

    fn resume<F: FnOnce() + 'static>(
        &mut self,
        window: Arc<dyn anyrender::WindowHandle>,
        width: u32,
        height: u32,
        on_ready: F,
    ) {
        // A fail-closed renderer can still own an active backend. A redundant
        // resume must not clear the fault or reactivate painting.
        if self.active_backend.get().is_some()
            || self.inner.borrow().is_active()
            || self.is_pending()
        {
            return;
        }
        self.last_error.borrow_mut().take();
        #[cfg(not(target_arch = "wasm32"))]
        {
            // Native adapters initialize inline. Dispatch readiness after the
            // active surface (GPU or software) has finished initializing.
            let ready = Rc::new(RefCell::new(Some(on_ready)));
            let notify = Rc::clone(&ready);
            let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                self.inner
                    .borrow_mut()
                    .resume(Arc::clone(&window), width, height, move || {
                        let callback = notify.borrow_mut().take();
                        if let Some(callback) = callback {
                            callback();
                        }
                    });
            }));
            if let Err(error) = result {
                self.inner.borrow_mut().suspend();
                let reason = panic_reason(&*error).to_string();
                self.record_fault(reason.clone());

                #[cfg(all(feature = "gpu-cpu-fallback", not(target_arch = "wasm32")))]
                if self.fallback_policy == RendererFallbackPolicy::CpuOnFailure {
                    if !self.fallback.borrow().supports_window_alpha() {
                        self.record_fault(format!(
                            "{reason}; software presentation requires an opaque surface"
                        ));
                        return;
                    }
                    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                        self.fallback.borrow_mut().resume(window, width, height);
                    }));
                    match result {
                        Ok(()) => {
                            let callback = ready.borrow_mut().take();
                            if let Some(callback) = callback {
                                callback();
                            }
                        }
                        Err(error) => self.record_fault(format!(
                            "{reason}; software surface: {}",
                            panic_reason(&*error)
                        )),
                    }
                }
            }
        }
        #[cfg(target_arch = "wasm32")]
        self.inner
            .borrow_mut()
            .resume(window, width, height, on_ready);
    }

    fn complete_resume(&mut self) -> bool {
        #[cfg(all(feature = "gpu-cpu-fallback", not(target_arch = "wasm32")))]
        if self.fallback.borrow().active() {
            self.active_backend.set(Some(ActiveBackend::Software));
            return true;
        }
        let active = self.inner.borrow_mut().complete_resume();
        if active {
            self.active_backend.set(Some(ActiveBackend::Primary));
        }
        active
    }

    fn suspend(&mut self) {
        self.active_backend.set(None);
        #[cfg(all(feature = "gpu-cpu-fallback", not(target_arch = "wasm32")))]
        self.fallback.borrow_mut().suspend();
        self.inner.borrow_mut().suspend()
    }

    fn is_active(&self) -> bool {
        self.painting_allowed() && self.active_backend.get().is_some()
    }

    fn is_pending(&self) -> bool {
        self.inner.borrow().is_pending()
    }

    fn set_size(&mut self, width: u32, height: u32) {
        #[cfg(all(feature = "gpu-cpu-fallback", not(target_arch = "wasm32")))]
        if let Some(renderer) = self.fallback.borrow_mut().renderer.as_mut() {
            renderer.set_size(width, height);
            return;
        }
        self.inner.borrow_mut().set_size(width, height)
    }

    fn render<F: FnOnce(&mut Self::ScenePainter<'_>)>(&mut self, draw_fn: F) {
        if !self.painting_allowed() {
            return;
        }
        #[cfg(all(feature = "gpu-cpu-fallback", not(target_arch = "wasm32")))]
        {
            if let Some(renderer) = self.fallback.borrow_mut().renderer.as_mut() {
                renderer.render(|painter| draw_recorded(painter, draw_fn));
                return;
            }
            self.inner
                .borrow_mut()
                .render(|painter| draw_recorded(painter, draw_fn));
        }
        #[cfg(not(all(feature = "gpu-cpu-fallback", not(target_arch = "wasm32"))))]
        self.inner.borrow_mut().render(draw_fn);
    }
}

#[cfg(not(target_arch = "wasm32"))]
fn panic_reason(error: &(dyn Any + Send)) -> &str {
    error
        .downcast_ref::<String>()
        .map(String::as_str)
        .or_else(|| error.downcast_ref::<&str>().copied())
        .unwrap_or("native renderer initialization failed")
}
