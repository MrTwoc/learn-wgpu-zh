use std::iter;

use app_surface::{AppSurface, SurfaceFrame};
use winit::window::WindowId;
use winit::{
    dpi::PhysicalSize,
    event::*,
    keyboard::{Key, NamedKey},
};

use utils::framework::{run, Action};

struct State {
    app: AppSurface,
    render_pipeline: wgpu::RenderPipeline,
    challenge_render_pipeline: wgpu::RenderPipeline,
    use_color: bool,
}

impl Action for State {
    fn new(app: AppSurface) -> Self {
        let shader = app
            .device
            .create_shader_module(wgpu::ShaderModuleDescriptor {
                label: Some("Shader"),
                source: wgpu::ShaderSource::Wgsl(include_str!("shader.wgsl").into()),
            });

        let render_pipeline_layout =
            app.device
                .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                    label: Some("Render Pipeline Layout"),
                    bind_group_layouts: &[],
                    push_constant_ranges: &[],
                });

        let render_pipeline = app
            .device
            .create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                label: Some("Render Pipeline"),
                layout: Some(&render_pipeline_layout),
                vertex: wgpu::VertexState {
                    module: &shader,
                    entry_point: "vs_main",
                    compilation_options: Default::default(),
                    buffers: &[],
                },
                fragment: Some(wgpu::FragmentState {
                    module: &shader,
                    entry_point: "fs_main",
                    compilation_options: Default::default(),
                    targets: &[Some(wgpu::ColorTargetState {
                        format: app.config.format.add_srgb_suffix(),
                        blend: Some(wgpu::BlendState {
                            color: wgpu::BlendComponent::REPLACE,
                            alpha: wgpu::BlendComponent::REPLACE,
                        }),
                        write_mask: wgpu::ColorWrites::ALL,
                    })],
                }),
                primitive: wgpu::PrimitiveState {
                    topology: wgpu::PrimitiveTopology::TriangleList,
                    strip_index_format: None,
                    front_face: wgpu::FrontFace::Ccw,
                    cull_mode: Some(wgpu::Face::Back),
                    polygon_mode: wgpu::PolygonMode::Fill,
                    // Requires Features::DEPTH_CLIP_CONTROL
                    unclipped_depth: false,
                    // Requires Features::CONSERVATIVE_RASTERIZATION
                    conservative: false,
                },
                depth_stencil: None,
                multisample: wgpu::MultisampleState {
                    count: 1,
                    mask: !0,
                    alpha_to_coverage_enabled: false,
                },
                // If the pipeline will be used with a multiview render pass, this
                // indicates how many array layers the attachments will have.
                multiview: None,
                cache: None,
            });

        let shader = app
            .device
            .create_shader_module(wgpu::ShaderModuleDescriptor {
                label: Some("Challenge Shader"),
                source: wgpu::ShaderSource::Wgsl(include_str!("challenge.wgsl").into()),
            });

        let challenge_render_pipeline =
            app.device
                .create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                    label: Some("Render Pipeline"),
                    layout: Some(&render_pipeline_layout),
                    vertex: wgpu::VertexState {
                        module: &shader,
                        entry_point: "vs_main",
                        compilation_options: Default::default(),
                        buffers: &[],
                    },
                    fragment: Some(wgpu::FragmentState {
                        module: &shader,
                        entry_point: "fs_main",
                        compilation_options: Default::default(),
                        targets: &[Some(wgpu::ColorTargetState {
                            format: app.config.format.add_srgb_suffix(),
                            blend: Some(wgpu::BlendState::REPLACE),
                            write_mask: wgpu::ColorWrites::ALL,
                        })],
                    }),
                    primitive: wgpu::PrimitiveState {
                        topology: wgpu::PrimitiveTopology::TriangleList,
                        strip_index_format: None,
                        front_face: wgpu::FrontFace::Ccw,
                        cull_mode: Some(wgpu::Face::Back),
                        // Setting this to anything other than Fill requires Features::NON_FILL_POLYGON_MODE
                        polygon_mode: wgpu::PolygonMode::Fill,
                        ..Default::default()
                    },
                    depth_stencil: None,
                    multisample: wgpu::MultisampleState {
                        count: 1,
                        mask: !0,
                        alpha_to_coverage_enabled: false,
                    },
                    // If the pipeline will be used with a multiview render pass, this
                    // indicates how many array layers the attachments will have.
                    multiview: None,
                    cache: None,
                });

        let use_color = true;

        Self {
            app,
            render_pipeline,
            challenge_render_pipeline,
            use_color,
        }
    }

    fn start(&mut self) {
        //  只有在进入事件循环之后，才有可能真正获取到窗口大小。
        let size = self.app.get_view().inner_size();
        self.resize(&size);
    }

    fn get_adapter_info(&self) -> wgpu::AdapterInfo {
        self.app.adapter.get_info()
    }
    fn current_window_id(&self) -> WindowId {
        self.app.get_view().id()
    }

    fn resize(&mut self, size: &PhysicalSize<u32>) {
        if self.app.config.width == size.width && self.app.config.height == size.height {
            return;
        }
        self.app.resize_surface();
    }
    fn request_redraw(&mut self) {
        self.app.get_view().request_redraw();
    }

    fn input(&mut self, event: &WindowEvent) -> bool {
        match event {
            WindowEvent::KeyboardInput {
                event:
                    KeyEvent {
                        state,
                        logical_key: Key::Named(NamedKey::Space),
                        ..
                    },
                ..
            } => {
                self.use_color = *state == ElementState::Released;
                true
            }
            _ => false,
        }
    }

    fn render(&mut self) -> Result<(), wgpu::SurfaceError> {
        let (output, view) = self.app.get_current_frame_view(None);

        let mut encoder = self
            .app
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Render Encoder"),
            });

        let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("Render Pass"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: &view,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(wgpu::Color {
                        r: 0.1,
                        g: 0.2,
                        b: 0.3,
                        a: 1.0,
                    }),
                    store: wgpu::StoreOp::Store,
                },
            })],
            ..Default::default()
        });

        render_pass.set_pipeline(if self.use_color {
            &self.render_pipeline
        } else {
            &self.challenge_render_pipeline
        });
        render_pass.draw(0..3, 0..1);
        drop(render_pass);

        self.app.queue.submit(iter::once(encoder.finish()));
        output.present();

        Ok(())
    }
}

fn main() {
    run::<State>(None, None);
}
