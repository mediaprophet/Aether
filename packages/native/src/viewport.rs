// Copyright (c) 2026 Timothy Charles Holborn.
// SPDX-License-Identifier: CC-BY-NC-ND-4.0

//! Custom-widget rendering using the native renderer's Etalon device.

use anyrender::{RenderContext, Scene};
use blitz_dom::{IntrinsicSizes, Widget, node::ComputedStyles};
use blitz_traits::events::UiEvent;
use std::{cell::RefCell, rc::Rc};

#[cfg(any(feature = "vello", feature = "vello-hybrid"))]
use anyrender::{PaintRef, PaintScene, ResourceId};
#[cfg(any(feature = "vello", feature = "vello-hybrid"))]
use etalon_context::DeviceHandle;
#[cfg(any(feature = "vello", feature = "vello-hybrid"))]
use peniko::{
    Fill, ImageBrush,
    kurbo::{Affine, Rect},
};

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum EtalonSurfaceError {
    NoGpuContextAvailable,
    TextureAllocationFailed(String),
    ResourceRegistrationFailed(String),
    DeviceLost(String),
    RenderPassFailed(String),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RenderOutcome {
    Static,
    ContinuousRedraw,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum EtalonViewportState {
    Suspended,
    Active,
    CpuFallbackNotice(String),
    Faulted(EtalonSurfaceError),
}

#[cfg(any(feature = "vello", feature = "vello-hybrid"))]
pub struct EtalonFrameContext<'a> {
    pub device: &'a etalon::Device,
    pub queue: &'a etalon::Queue,
    pub target_view: &'a etalon::TextureView,
    pub width: u32,
    pub height: u32,
    pub scale: f64,
    pub frame_index: u64,
}

/// Client code painting into the renderer's texture or recording a CPU scene.
pub trait EtalonSurfaceDelegate: 'static {
    #[cfg(any(feature = "vello", feature = "vello-hybrid"))]
    fn prepare(
        &mut self,
        device: &etalon::Device,
        queue: &etalon::Queue,
    ) -> Result<(), EtalonSurfaceError>;

    #[cfg(any(feature = "vello", feature = "vello-hybrid"))]
    fn render_frame(
        &mut self,
        ctx: &EtalonFrameContext<'_>,
    ) -> Result<RenderOutcome, EtalonSurfaceError>;

    fn paint_fallback(&mut self, _width: u32, _height: u32, _scale: f64) -> Scene {
        Scene::new()
    }
    fn handle_event(&mut self, _event: &UiEvent, _scale: f64) {}
    fn focus_changed(&mut self, _focused: bool) {}
    fn intrinsic_sizes(&self) -> IntrinsicSizes {
        IntrinsicSizes::default()
    }
    fn destroy(&mut self) {}

    /// Bijective 3D pick: map a widget-local pixel to a semantic id.
    fn pick_semantic_id(
        &self,
        _local_x: f32,
        _local_y: f32,
        _width: f32,
        _height: f32,
    ) -> Option<u64> {
        None
    }
}

#[cfg(any(feature = "vello", feature = "vello-hybrid"))]
struct FrameTarget {
    id: ResourceId,
    view: etalon::TextureView,
}

/// A replaced DOM widget with renderer-scoped Etalon texture ownership.
pub struct EtalonSurfaceWidget {
    delegate: Box<dyn EtalonSurfaceDelegate>,
    state: Rc<RefCell<EtalonViewportState>>,
    last_scale: f64,
    redraw: bool,
    #[cfg(any(feature = "vello", feature = "vello-hybrid"))]
    gpu: Option<Box<DeviceHandle>>,
    #[cfg(any(feature = "vello", feature = "vello-hybrid"))]
    targets: Option<([FrameTarget; 2], (u32, u32))>,
    #[cfg(any(feature = "vello", feature = "vello-hybrid"))]
    frame_index: u64,
}

impl EtalonSurfaceWidget {
    pub fn new(delegate: impl EtalonSurfaceDelegate) -> Self {
        Self {
            delegate: Box::new(delegate),
            state: Rc::new(RefCell::new(EtalonViewportState::Suspended)),
            last_scale: 1.0,
            redraw: false,
            #[cfg(any(feature = "vello", feature = "vello-hybrid"))]
            gpu: None,
            #[cfg(any(feature = "vello", feature = "vello-hybrid"))]
            targets: None,
            #[cfg(any(feature = "vello", feature = "vello-hybrid"))]
            frame_index: 0,
        }
    }

    pub fn state(&self) -> EtalonViewportState {
        self.state.borrow().clone()
    }

    /// Read the status of a widget after handing its ownership to the DOM.
    pub fn state_reader(&self) -> impl Fn() -> EtalonViewportState + use<> {
        let state = self.state.clone();
        move || state.borrow().clone()
    }

    #[cfg(any(feature = "vello", feature = "vello-hybrid"))]
    fn release_targets(&mut self, context: &mut dyn RenderContext) {
        if let Some((targets, _)) = self.targets.take() {
            for target in targets {
                context.unregister_resource(target.id);
            }
        }
    }

    #[cfg(any(feature = "vello", feature = "vello-hybrid"))]
    fn ensure_targets(
        &mut self,
        context: &mut dyn RenderContext,
        width: u32,
        height: u32,
    ) -> Result<(), EtalonSurfaceError> {
        if self
            .targets
            .as_ref()
            .is_some_and(|(_, extent)| *extent == (width, height))
        {
            return Ok(());
        }
        self.release_targets(context);
        let gpu = self
            .gpu
            .as_ref()
            .ok_or(EtalonSurfaceError::NoGpuContextAvailable)?;
        let first = allocate_target(context, &gpu.device, width, height)?;
        let second = match allocate_target(context, &gpu.device, width, height) {
            Ok(second) => second,
            Err(error) => {
                context.unregister_resource(first.id);
                return Err(error);
            }
        };
        self.targets = Some(([first, second], (width, height)));
        Ok(())
    }

    #[cfg(any(feature = "vello", feature = "vello-hybrid"))]
    fn paint_gpu(
        &mut self,
        context: &mut dyn RenderContext,
        width: u32,
        height: u32,
    ) -> Result<Scene, EtalonSurfaceError> {
        self.ensure_targets(context, width, height)?;
        let gpu = self
            .gpu
            .as_ref()
            .ok_or(EtalonSurfaceError::NoGpuContextAvailable)?;
        let (targets, _) = self
            .targets
            .as_ref()
            .ok_or(EtalonSurfaceError::NoGpuContextAvailable)?;
        let target = &targets[(self.frame_index & 1) as usize];
        let outcome = self.delegate.render_frame(&EtalonFrameContext {
            device: &gpu.device,
            queue: &gpu.queue,
            target_view: &target.view,
            width,
            height,
            scale: self.last_scale,
            frame_index: self.frame_index,
        })?;
        self.frame_index = self.frame_index.wrapping_add(1);
        self.redraw = outcome == RenderOutcome::ContinuousRedraw;
        let mut scene = Scene::new();
        scene.fill(
            Fill::NonZero,
            Affine::IDENTITY,
            PaintRef::Resource(ImageBrush {
                image: target.id,
                sampler: Default::default(),
            }),
            None,
            &Rect::new(0.0, 0.0, f64::from(width), f64::from(height)),
        );
        Ok(scene)
    }
}

#[cfg(any(feature = "vello", feature = "vello-hybrid"))]
fn allocate_target(
    context: &mut dyn RenderContext,
    device: &etalon::Device,
    width: u32,
    height: u32,
) -> Result<FrameTarget, EtalonSurfaceError> {
    let maximum = device.limits().max_texture_dimension_2d;
    if width == 0 || height == 0 || width > maximum || height > maximum {
        return Err(EtalonSurfaceError::TextureAllocationFailed(format!(
            "invalid extent {width}x{height} for limit {maximum}"
        )));
    }
    #[cfg(not(target_arch = "wasm32"))]
    let memory = device.push_error_scope(etalon::ErrorFilter::OutOfMemory);
    #[cfg(not(target_arch = "wasm32"))]
    let validation = device.push_error_scope(etalon::ErrorFilter::Validation);
    let texture = device.create_texture(&etalon::TextureDescriptor {
        label: Some("Aether viewport"),
        size: etalon::Extent3d {
            width,
            height,
            depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: etalon::TextureDimension::D2,
        format: etalon::TextureFormat::Rgba8Unorm,
        usage: etalon::TextureUsages::RENDER_ATTACHMENT | etalon::TextureUsages::TEXTURE_BINDING,
        view_formats: &[],
    });
    #[cfg(not(target_arch = "wasm32"))]
    {
        use futures_util::FutureExt as _;
        let validation = validation.pop().now_or_never();
        let memory = memory.pop().now_or_never();
        for result in [validation, memory] {
            match result {
                Some(Some(error)) => {
                    return Err(EtalonSurfaceError::TextureAllocationFailed(
                        error.to_string(),
                    ));
                }
                None => {
                    return Err(EtalonSurfaceError::TextureAllocationFailed(
                        "Etalon did not resolve the allocation scope".into(),
                    ));
                }
                Some(None) => {}
            }
        }
    }
    let view = texture.create_view(&Default::default());
    let id = context
        .try_register_custom_resource(Box::new(texture))
        .map_err(|error| {
            EtalonSurfaceError::ResourceRegistrationFailed(
                error.message.unwrap_or_else(|| format!("{:?}", error.kind)),
            )
        })?;
    Ok(FrameTarget { id, view })
}

impl Widget for EtalonSurfaceWidget {
    fn can_create_surfaces(&mut self, context: &mut dyn RenderContext) {
        #[cfg(any(feature = "vello", feature = "vello-hybrid"))]
        if let Some(gpu) = context
            .renderer_specific_context()
            .and_then(|value| value.downcast::<DeviceHandle>().ok())
        {
            match self.delegate.prepare(&gpu.device, &gpu.queue) {
                Ok(()) => {
                    self.gpu = Some(gpu);
                    *self.state.borrow_mut() = EtalonViewportState::Active;
                }
                Err(error) => {
                    *self.state.borrow_mut() = EtalonViewportState::Faulted(error);
                }
            }
            return;
        }
        let _ = context;
        *self.state.borrow_mut() =
            EtalonViewportState::CpuFallbackNotice("CPU raster backend active".into());
    }

    fn disconnected(&mut self) {
        self.destroy_surfaces();
    }

    fn destroy_surfaces(&mut self) {
        #[cfg(any(feature = "vello", feature = "vello-hybrid"))]
        {
            // Blitz owns the registrations and drains them after this callback.
            self.gpu = None;
            self.targets = None;
            self.frame_index = 0;
        }
        self.redraw = false;
        let previous = self.state.replace(EtalonViewportState::Suspended);
        if previous != EtalonViewportState::Suspended {
            self.delegate.destroy();
        }
    }

    fn requires_redraw(&self) -> bool {
        self.redraw
    }
    fn handle_event(&mut self, event: &UiEvent) {
        self.delegate.handle_event(event, self.last_scale);
    }
    fn focus_changed(&mut self, focused: bool) {
        self.delegate.focus_changed(focused);
    }
    fn intrinsic_sizes(&self) -> IntrinsicSizes {
        self.delegate.intrinsic_sizes()
    }

    fn paint(
        &mut self,
        context: &mut dyn RenderContext,
        _: &ComputedStyles,
        width: u32,
        height: u32,
        scale: f64,
    ) -> Scene {
        self.last_scale = if scale.is_finite() && scale > 0.0 {
            scale
        } else {
            1.0
        };
        if width == 0 || height == 0 {
            self.redraw = false;
            return Scene::new();
        }
        #[cfg(any(feature = "vello", feature = "vello-hybrid"))]
        if self.state() == EtalonViewportState::Active {
            match self.paint_gpu(context, width, height) {
                Ok(scene) => return scene,
                Err(error) => {
                    self.release_targets(context);
                    self.redraw = false;
                    *self.state.borrow_mut() = EtalonViewportState::Faulted(error);
                }
            }
        }
        let _ = context;
        self.delegate.paint_fallback(width, height, self.last_scale)
    }
}

/// Frame parameters passed to an [`EtalonVectorDelegate`] when recording a scene
/// directly into the window's active Vello Hybrid / AnyRender GPU pass.
pub struct EtalonVectorFrame<'a> {
    pub styles: &'a ComputedStyles,
    pub width: u32,
    pub height: u32,
    pub scale: f64,
    pub frame_index: u64,
    pub gpu_active: bool,
}

/// Zero-texture procedural vector & 2.5D/3D projected phase-space delegate that
/// records geometry directly into the renderer's shared [`anyrender::Scene`].
///
/// Unlike [`EtalonSurfaceDelegate`] (which allocates double-buffered offscreen
/// [`etalon::Texture`] targets for custom WGSL raymarching), [`EtalonVectorDelegate`]
/// composites vector curves, 3D wireframe projections, harmonic phase-space rings,
/// and VibeScript-driven kinetic glyphs directly in the main Vello Hybrid pass with
/// zero per-widget texture allocation overhead.
pub trait EtalonVectorDelegate: 'static {
    fn paint_scene(&mut self, frame: &EtalonVectorFrame<'_>, scene: &mut Scene) -> RenderOutcome;

    fn handle_event(&mut self, _event: &UiEvent, _scale: f64) {}
    fn focus_changed(&mut self, _focused: bool) {}
    fn intrinsic_sizes(&self) -> IntrinsicSizes {
        IntrinsicSizes::default()
    }
    fn destroy(&mut self) {}

    /// Bijective 3D pick: map a widget-local pixel to a semantic id.
    fn pick_semantic_id(
        &self,
        _local_x: f32,
        _local_y: f32,
        _width: f32,
        _height: f32,
    ) -> Option<u64> {
        None
    }
}

/// A lightweight DOM widget that records procedural vector and 3D-projected geometry
/// directly into the window's [`anyrender::Scene`] without offscreen texture allocation.
pub struct EtalonVectorWidget {
    delegate: Box<dyn EtalonVectorDelegate>,
    state: Rc<RefCell<EtalonViewportState>>,
    last_scale: f64,
    redraw: bool,
    frame_index: u64,
    gpu_active: bool,
}

impl EtalonVectorWidget {
    pub fn new(delegate: impl EtalonVectorDelegate) -> Self {
        Self {
            delegate: Box::new(delegate),
            state: Rc::new(RefCell::new(EtalonViewportState::Suspended)),
            last_scale: 1.0,
            redraw: false,
            frame_index: 0,
            gpu_active: false,
        }
    }

    pub fn state(&self) -> EtalonViewportState {
        self.state.borrow().clone()
    }

    pub fn state_reader(&self) -> impl Fn() -> EtalonViewportState + use<> {
        let state = self.state.clone();
        move || state.borrow().clone()
    }
}

impl Widget for EtalonVectorWidget {
    fn can_create_surfaces(&mut self, context: &mut dyn RenderContext) {
        #[cfg(any(feature = "vello", feature = "vello-hybrid"))]
        if context
            .renderer_specific_context()
            .and_then(|value| value.downcast::<DeviceHandle>().ok())
            .is_some()
        {
            self.gpu_active = true;
            *self.state.borrow_mut() = EtalonViewportState::Active;
            return;
        }
        let _ = context;
        self.gpu_active = false;
        *self.state.borrow_mut() =
            EtalonViewportState::CpuFallbackNotice("CPU vector scene active".into());
    }

    fn disconnected(&mut self) {
        self.destroy_surfaces();
    }

    fn destroy_surfaces(&mut self) {
        self.gpu_active = false;
        self.redraw = false;
        self.frame_index = 0;
        let previous = self.state.replace(EtalonViewportState::Suspended);
        if previous != EtalonViewportState::Suspended {
            self.delegate.destroy();
        }
    }

    fn requires_redraw(&self) -> bool {
        self.redraw
    }

    fn handle_event(&mut self, event: &UiEvent) {
        self.delegate.handle_event(event, self.last_scale);
    }

    fn focus_changed(&mut self, focused: bool) {
        self.delegate.focus_changed(focused);
    }

    fn intrinsic_sizes(&self) -> IntrinsicSizes {
        self.delegate.intrinsic_sizes()
    }

    fn paint(
        &mut self,
        _context: &mut dyn RenderContext,
        styles: &ComputedStyles,
        width: u32,
        height: u32,
        scale: f64,
    ) -> Scene {
        self.last_scale = if scale.is_finite() && scale > 0.0 {
            scale
        } else {
            1.0
        };
        if width == 0 || height == 0 {
            self.redraw = false;
            return Scene::new();
        }
        let mut scene = Scene::new();
        let outcome = self.delegate.paint_scene(
            &EtalonVectorFrame {
                styles,
                width,
                height,
                scale: self.last_scale,
                frame_index: self.frame_index,
                gpu_active: self.gpu_active,
            },
            &mut scene,
        );
        self.frame_index = self.frame_index.wrapping_add(1);
        self.redraw = outcome == RenderOutcome::ContinuousRedraw;
        scene
    }
}

