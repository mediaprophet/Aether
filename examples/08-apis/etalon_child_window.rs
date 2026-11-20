//! Demonstrate how to use aether as a child window for use in alternative renderers like etalon.
//!
//! The code here is borrowed from wry's example:
//! https://github.com/tauri-apps/wry/blob/dev/examples/wgpu.rs
//!
//! To use this feature set `with_as_child_window()` on your desktop config which will then let you

use aether::prelude::*;
use aether::{
    desktop::tao::{event::Event as WryEvent, window::Window},
    desktop::{Config, tao::window::WindowBuilder, use_wry_event_handler, window},
};
use std::sync::Arc;
use etalon::CurrentSurfaceTexture;

fn main() {
    let config = Config::new()
        .with_window(WindowBuilder::new().with_transparent(true))
        .with_on_window(|window, dom| {
            let resources = Arc::new(pollster::block_on(async {
                let resource = GraphicsContextAsyncBuilder {
                    desktop: window,
                    resources_builder: |ctx| Box::pin(GraphicsResources::new(ctx.clone())),
                }
                .build()
                .await;

                resource.with_resources(|resources| resources.render());

                resource
            }));

            dom.provide_root_context(resources);
        })
        .with_as_child_window();

    aether::LaunchBuilder::desktop()
        .with_cfg(config)
        .launch(app);
}

fn app() -> Element {
    let graphics_resources = consume_context::<Arc<GraphicsContext>>();

    // on first render request a redraw
    use_effect(|| {
        window().window.request_redraw();
    });

    use_wry_event_handler(move |event, _| {
        use aether::desktop::tao::event::WindowEvent;

        if let WryEvent::WindowEvent {
            event: WindowEvent::Resized(new_size),
            ..
        } = event
        {
            graphics_resources.with_resources(|srcs| {
                let mut cfg = srcs.config.clone();
                cfg.width = new_size.width;
                cfg.height = new_size.height;
                srcs.surface.configure(&srcs.device, &cfg);
            });

            window().window.request_redraw();
        }
    });

    rsx! {
        div {
            color: "blue",
            width: "100vw",
            height: "100vh",
            display: "flex",
            justify_content: "center",
            align_items: "center",
            font_size: "20px",
            div { "text overlaid on a etalon surface!" }
        }
    }
}

/// This borrows from the `window` which is contained within an `Arc` so we need to wrap it in a self-borrowing struct
/// to be able to borrow the window for the etalon::Surface
#[ouroboros::self_referencing]
struct GraphicsContext {
    desktop: Arc<Window>,
    #[borrows(desktop)]
    #[not_covariant]
    resources: GraphicsResources<'this>,
}

struct GraphicsResources<'a> {
    surface: etalon::Surface<'a>,
    device: etalon::Device,
    pipeline: etalon::RenderPipeline,
    queue: etalon::Queue,
    config: etalon::SurfaceConfiguration,
}

impl<'a> GraphicsResources<'a> {
    async fn new(window: Arc<Window>) -> Self {
        let size = window.inner_size();

        let instance = etalon::Instance::default();

        let surface: etalon::Surface<'a> = instance.create_surface(window).unwrap();
        let adapter = instance
            .request_adapter(&etalon::RequestAdapterOptions {
                power_preference: etalon::PowerPreference::default(),
                force_fallback_adapter: false,
                // Request an adapter which can render to our surface
                compatible_surface: Some(&surface),
                apply_limit_buckets: false,
            })
            .await
            .expect("Failed to find an appropriate adapter");

        // Create the logical device and command queue
        let (device, queue) = adapter
            .request_device(&etalon::DeviceDescriptor {
                label: None,
                required_features: etalon::Features::empty(),
                // Make sure we use the texture resolution limits from the adapter, so we can support images the size of the swapchain.
                required_limits: etalon::Limits::downlevel_webgl2_defaults()
                    .using_resolution(adapter.limits()),
                memory_hints: etalon::MemoryHints::default(),
                ..Default::default()
            })
            .await
            .expect("Failed to create device");

        // Load the shaders from disk
        let shader = device.create_shader_module(etalon::ShaderModuleDescriptor {
            label: None,
            source: etalon::ShaderSource::Wgsl(
                r#"
@vertex
fn vs_main(@builtin(vertex_index) in_vertex_index: u32) -> @builtin(position) vec4<f32> {
    let x = f32(i32(in_vertex_index) - 1);
    let y = f32(i32(in_vertex_index & 1u) * 2 - 1);
    return vec4<f32>(x, y, 0.0, 1.0);
}

@fragment
fn fs_main() -> @location(0) vec4<f32> {
    return vec4<f32>(1.0, 0.0, 0.0, 1.0);
}
"#
                .into(),
            ),
        });

        let pipeline_layout = device.create_pipeline_layout(&etalon::PipelineLayoutDescriptor {
            label: None,
            bind_group_layouts: &[],
            immediate_size: 0,
        });

        let swapchain_capabilities = surface.get_capabilities(&adapter);
        let swapchain_format = swapchain_capabilities.formats[0];

        let pipeline = device.create_render_pipeline(&etalon::RenderPipelineDescriptor {
            label: None,
            layout: Some(&pipeline_layout),
            vertex: etalon::VertexState {
                module: &shader,
                entry_point: Some("vs_main"),
                buffers: &[],
                compilation_options: etalon::PipelineCompilationOptions::default(),
            },
            fragment: Some(etalon::FragmentState {
                module: &shader,
                entry_point: Some("fs_main"),
                targets: &[Some(swapchain_format.into())],
                compilation_options: etalon::PipelineCompilationOptions::default(),
            }),
            primitive: etalon::PrimitiveState::default(),
            depth_stencil: None,
            multisample: etalon::MultisampleState::default(),
            multiview_mask: None,
            cache: None,
        });

        let config = etalon::SurfaceConfiguration {
            usage: etalon::TextureUsages::RENDER_ATTACHMENT,
            format: swapchain_format,
            width: size.width,
            height: size.height,
            present_mode: etalon::PresentMode::Fifo,
            desired_maximum_frame_latency: 2,
            alpha_mode: etalon::CompositeAlphaMode::PostMultiplied,
            view_formats: vec![],
            color_space: etalon::SurfaceColorSpace::Auto,
        };

        surface.configure(&device, &config);

        GraphicsResources {
            surface,
            device,
            pipeline,
            queue,
            config,
        }
    }

    fn render(&self) {
        let GraphicsResources {
            surface,
            device,
            pipeline,
            queue,
            config,
        } = self;

        let frame = match surface.get_current_texture() {
            CurrentSurfaceTexture::Success(surface_texture) => surface_texture,
            CurrentSurfaceTexture::Lost
            | CurrentSurfaceTexture::Outdated
            | CurrentSurfaceTexture::Suboptimal(_) => {
                surface.configure(device, config);
                return;
            }
            CurrentSurfaceTexture::Occluded | CurrentSurfaceTexture::Timeout => {
                return;
            }
            CurrentSurfaceTexture::Validation => panic!("Current surface texture is invalid"),
        };

        let view = frame
            .texture
            .create_view(&etalon::TextureViewDescriptor::default());

        let mut encoder =
            device.create_command_encoder(&etalon::CommandEncoderDescriptor { label: None });

        {
            let mut rpass = encoder.begin_render_pass(&etalon::RenderPassDescriptor {
                label: None,
                color_attachments: &[Some(etalon::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: etalon::Operations {
                        load: etalon::LoadOp::Clear(etalon::Color::TRANSPARENT),
                        store: etalon::StoreOp::Store,
                    },
                    depth_slice: None,
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
                multiview_mask: None,
            });
            rpass.set_pipeline(pipeline);
            rpass.draw(0..3, 0..1);
        }

        queue.submit(Some(encoder.finish()));
        queue.present(frame);
    }
}
