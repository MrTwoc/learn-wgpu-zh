use crate::{hilbert_curve::HilbertCurve, line::Line};
use app_surface::{AppSurface, SurfaceFrame};
use std::{iter, sync::Arc};
use utils::{BufferObj, SceneUniform, WgpuAppAction};
use wgpu::WasmNotSend;
use winit::dpi::PhysicalSize;

pub struct HilbertCurveApp {
    app: AppSurface,
    size: PhysicalSize<u32>,
    size_changed: bool,
    mvp_buffer: BufferObj,
    line: Line,
    // 当前曲线与目标曲线的顶点缓冲区
    vertex_buffers: Vec<wgpu::Buffer>,
    // 当前曲线的顶点总数
    curve_vertex_count: usize,
    // 当前动画帧的索引，用于设置缓冲区的动态偏移
    animate_index: u32,
    // 每一个动画阶段的总帧数
    draw_count: u32,
    // 目标曲线维度
    curve_dimention: u32,
    // 是否为升维动画
    is_animation_up: bool,
}

impl HilbertCurveApp {
    /// 必要的时候调整 surface 大小
    fn resize_surface_if_needed(&mut self) {
        if self.size_changed {
            //  需先 resize surface
            self.app
                .resize_surface_by_size((self.size.width, self.size.height));

            let viewport = glam::Vec2 {
                x: self.size.width as f32,
                y: self.size.height as f32,
            };
            // 更新 uniform
            let (p_matrix, mv_matrix, _) = utils::matrix_helper::perspective_mvp(viewport);
            let resized_uniform = SceneUniform {
                mvp: (p_matrix * mv_matrix).to_cols_array_2d(),
                viewport_pixels: viewport.to_array(),
                padding: [0., 0.],
            };
            self.app.queue.write_buffer(
                &self.mvp_buffer.buffer,
                0,
                bytemuck::bytes_of(&resized_uniform),
            );
            self.size_changed = false;
        }
    }
}

impl WgpuAppAction for HilbertCurveApp {
    fn new(
        window: Arc<winit::window::Window>,
    ) -> impl std::future::Future<Output = Self> + WasmNotSend {
        async move {
            // 创建 wgpu 应用
            let mut app = AppSurface::new(window).await;

            // 兼容 web
            let format = app.config.format.remove_srgb_suffix();
            app.ctx.update_config_format(format);

            let viewport = glam::Vec2 {
                x: app.config.width as f32,
                y: app.config.height as f32,
            };

            // 投影
            let (p_matrix, mv_matrix, _) = utils::matrix_helper::perspective_mvp(viewport);
            let mvp_buffer = BufferObj::create_uniform_buffer(
                &app.device,
                &SceneUniform {
                    mvp: (p_matrix * mv_matrix).to_cols_array_2d(),
                    viewport_pixels: viewport.to_array(),
                    padding: [0., 0.],
                },
                Some("SceneUniform"),
            );
            // 动作总帧总
            let draw_count = 60 * 3;
            let offset_buffer_size = 256;
            let hilbert_buf = BufferObj::create_empty_uniform_buffer(
                &app.device,
                (draw_count * offset_buffer_size) as wgpu::BufferAddress,
                offset_buffer_size,
                true,
                Some("动画的动态偏移缓冲区"),
            );
            // 按动态偏移量填充 uniform 缓冲区
            let mut depth_bias = 1.0;
            for step in 0..draw_count {
                let uniform = crate::HilbertUniform {
                    near_target_ratio: step as f32 / (draw_count - 1) as f32,
                    depth_bias,
                };
                app.queue.write_buffer(
                    &hilbert_buf.buffer,
                    offset_buffer_size * (step),
                    bytemuck::bytes_of(&uniform),
                );
                depth_bias -= 0.01;
            }

            // buffer 大小
            let size = (4 * 4 * 3) * HilbertCurve::new(5).vertices.len() as u64;
            // 创建两个 ping-pong 顶点缓冲区
            let mut vertex_buffers: Vec<wgpu::Buffer> = Vec::with_capacity(2);
            for _ in 0..2 {
                let buf = app.device.create_buffer(&wgpu::BufferDescriptor {
                    size,
                    usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
                    label: Some("vertex buffer"),
                    mapped_at_creation: false,
                });
                vertex_buffers.push(buf);
            }

            let line = Line::new(&app, &mvp_buffer, &hilbert_buf);

            let size = PhysicalSize::new(app.config.width, app.config.height);

            Self {
                app,
                size,
                size_changed: false,
                mvp_buffer,
                line,
                vertex_buffers,
                curve_vertex_count: 0,
                animate_index: 0,
                draw_count: draw_count as u32,
                curve_dimention: 1,
                is_animation_up: true,
            }
        }
    }

    fn set_window_resized(&mut self, new_size: PhysicalSize<u32>) {
        if self.app.config.width == new_size.width && self.app.config.height == new_size.height {
            return;
        }
        self.size = new_size;
        self.size_changed = true;
    }

    fn get_size(&self) -> PhysicalSize<u32> {
        PhysicalSize::new(self.app.config.width, self.app.config.height)
    }

    fn render(&mut self) -> Result<(), wgpu::SurfaceError> {
        self.resize_surface_if_needed();

        // 循环执行动画
        self.animate_index += 1;
        if self.animate_index == self.draw_count {
            // 本阶段动画完成，调整到下一阶段
            self.animate_index = 0;
            if self.is_animation_up {
                if self.curve_dimention == 6 {
                    self.is_animation_up = false;
                    self.curve_dimention -= 1;
                } else {
                    self.curve_dimention += 1;
                }
            } else if self.curve_dimention == 1 {
                self.is_animation_up = true;
                self.curve_dimention += 1;
            } else {
                self.curve_dimention -= 1;
            }

            let mut target = HilbertCurve::new(self.curve_dimention);
            let start = if self.is_animation_up {
                let mut start = HilbertCurve::new(self.curve_dimention - 1);
                // 把顶点数翻 4 倍来对应目标维度曲线
                start.four_times_vertices();
                start
            } else {
                target.four_times_vertices();
                HilbertCurve::new(self.curve_dimention + 1)
            };
            // 更新顶点数
            self.curve_vertex_count = target.vertices.len();
            // 填充顶点 buffer
            for (buf, curve) in self.vertex_buffers.iter().zip([start, target].iter()) {
                self.app
                    .queue
                    .write_buffer(buf, 0, bytemuck::cast_slice(&curve.vertices));
            }
        }
        if self.curve_vertex_count == 0 {
            return Ok(());
        }

        // 动画帧绘制
        let output = self.app.surface.get_current_texture().unwrap();
        let frame_view = output
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());
        let mut encoder = self
            .app
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Render Encoder"),
            });
        {
            let mut rpass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: None,
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &frame_view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(utils::unpack_u32_to_color(0xf2eaddff)),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                ..Default::default()
            });
            rpass.set_pipeline(&self.line.pipeline);
            rpass.set_bind_group(0, &self.line.bg_setting.bind_group, &[]);
            rpass.set_bind_group(
                1,
                &self.line.dy_bg.bind_group,
                &[256 * self.animate_index as wgpu::DynamicOffset],
            );
            let instance_count = self.curve_vertex_count as u32 - 1;

            rpass.set_vertex_buffer(0, self.vertex_buffers[0].slice(..));
            rpass.set_vertex_buffer(1, self.vertex_buffers[0].slice(12..));
            rpass.set_vertex_buffer(2, self.vertex_buffers[1].slice(..));
            rpass.set_vertex_buffer(3, self.vertex_buffers[1].slice(12..));

            rpass.draw(0..6, 0..instance_count);
        }
        self.app.queue.submit(iter::once(encoder.finish()));
        output.present();

        Ok(())
    }
}
