// Copyright (c) 2026 Timothy Charles Holborn.
// SPDX-License-Identifier: CC-BY-NC-ND-4.0

use anyrender::{
    Filter, Glyph, NormalizedCoord, PaintRef, PaintScene, RegisterResourceError, RenderContext,
    RendererConfig, ResourceId, Scene, WindowHandle, WindowRenderer,
};
use anyrender_vello_cpu::{SoftbufferWindowRenderer, VelloCpuImageRenderer};
use peniko::{
    BlendMode, Color, Fill, FontData, StyleRef,
    kurbo::{Affine, Rect, Shape, Stroke, Vec2},
};
use std::{any::Any, sync::Arc};

pub(super) struct CpuFallback {
    pub(super) renderer: Option<SoftbufferWindowRenderer<VelloCpuImageRenderer>>,
    config: RendererConfig,
}

impl CpuFallback {
    pub(super) fn new(config: RendererConfig) -> Self {
        Self {
            renderer: None,
            config,
        }
    }

    pub(super) fn supports_window_alpha(&self) -> bool {
        // The pinned softbuffer adapter presents RGB pixels.
        self.config.composite_alpha_mode != Some(anyrender::CompositeAlphaMode::Transparent)
    }

    pub(super) fn active(&self) -> bool {
        self.renderer
            .as_ref()
            .is_some_and(WindowRenderer::is_active)
    }

    pub(super) fn resume(&mut self, window: Arc<dyn WindowHandle>, width: u32, height: u32) {
        let mut renderer = SoftbufferWindowRenderer::with_options(self.config.clone());
        renderer.resume(window, width, height, || {});
        self.renderer = Some(renderer);
    }

    pub(super) fn suspend(&mut self) {
        if let Some(mut renderer) = self.renderer.take() {
            renderer.suspend();
        }
    }
}

/// A scene recorder sharing the active backend's custom-resource context.
pub struct FallbackScenePainter<'a> {
    scene: Scene,
    context: &'a mut dyn RenderContext,
}

pub(super) fn draw_recorded(
    painter: &mut impl PaintScene,
    draw: impl FnOnce(&mut FallbackScenePainter<'_>),
) {
    let scene = {
        let mut recording = FallbackScenePainter {
            scene: Scene::new(),
            context: painter,
        };
        draw(&mut recording);
        recording.scene
    };
    painter.append_scene(scene, Affine::IDENTITY);
}

impl RenderContext for FallbackScenePainter<'_> {
    fn try_register_custom_resource(
        &mut self,
        resource: Box<dyn Any>,
    ) -> Result<ResourceId, RegisterResourceError> {
        self.context.try_register_custom_resource(resource)
    }

    fn unregister_resource(&mut self, id: ResourceId) {
        self.context.unregister_resource(id);
    }

    fn renderer_specific_context(&self) -> Option<Box<dyn Any>> {
        self.context.renderer_specific_context()
    }
}

impl PaintScene for FallbackScenePainter<'_> {
    fn reset(&mut self) {
        self.scene.reset();
    }

    fn push_layer(
        &mut self,
        fill: Fill,
        blend: impl Into<BlendMode>,
        alpha: f32,
        transform: Affine,
        clip: &impl Shape,
        filter: Option<Arc<Filter>>,
        backdrop_filter: Option<Arc<Filter>>,
    ) {
        self.scene
            .push_layer(fill, blend, alpha, transform, clip, filter, backdrop_filter);
    }

    fn push_clip_layer(&mut self, fill: Fill, transform: Affine, clip: &impl Shape) {
        self.scene.push_clip_layer(fill, transform, clip);
    }

    fn pop_layer(&mut self) {
        self.scene.pop_layer();
    }

    fn stroke<'a>(
        &mut self,
        style: &Stroke,
        transform: Affine,
        brush: impl Into<PaintRef<'a>>,
        brush_transform: Option<Affine>,
        shape: &impl Shape,
    ) {
        self.scene
            .stroke(style, transform, brush, brush_transform, shape);
    }

    fn fill<'a>(
        &mut self,
        style: Fill,
        transform: Affine,
        brush: impl Into<PaintRef<'a>>,
        brush_transform: Option<Affine>,
        shape: &impl Shape,
    ) {
        self.scene
            .fill(style, transform, brush, brush_transform, shape);
    }

    fn draw_glyphs<'a, 's: 'a>(
        &'s mut self,
        font: &'a FontData,
        font_size: f32,
        hint: bool,
        normalized_coords: &'a [NormalizedCoord],
        embolden: Vec2,
        style: impl Into<StyleRef<'a>>,
        brush: impl Into<PaintRef<'a>>,
        brush_alpha: f32,
        transform: Affine,
        glyph_transform: Option<Affine>,
        glyphs: impl Iterator<Item = Glyph> + Clone,
    ) {
        self.scene.draw_glyphs(
            font,
            font_size,
            hint,
            normalized_coords,
            embolden,
            style,
            brush,
            brush_alpha,
            transform,
            glyph_transform,
            glyphs,
        );
    }

    fn draw_box_shadow(
        &mut self,
        transform: Affine,
        rect: Rect,
        brush: Color,
        radius: f64,
        std_dev: f64,
    ) {
        self.scene
            .draw_box_shadow(transform, rect, brush, radius, std_dev);
    }
}
