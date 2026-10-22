use bytemuck::{Pod, Zeroable};
use wgpu::{
    BlendState, ColorTargetState, ColorWrites, FragmentState, MultisampleState,
    PipelineCompilationOptions, PipelineLayoutDescriptor, PrimitiveState, RenderPipeline,
    RenderPipelineDescriptor, ShaderModuleDescriptor, ShaderSource, VertexBufferLayout,
    VertexState,
};

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
pub struct Vertex {
    position: [f32; 2],
    color: [f32; 4],
}

#[derive(Clone, Copy)]
pub struct Rect {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
    pub color: [f32; 4],
}

pub struct RectRenderer {
    pipeline: RenderPipeline,
    buffer: wgpu::Buffer,
    capacity: usize,
}

impl RectRenderer {
    pub fn new(device: &wgpu::Device, format: wgpu::TextureFormat) -> Self {
        let shader = device.create_shader_module(ShaderModuleDescriptor {
            label: Some("ancleto-rect-shader"),
            source: ShaderSource::Wgsl(SHADER.into()),
        });

        let layout = device.create_pipeline_layout(&PipelineLayoutDescriptor {
            label: Some("ancleto-rect-layout"),
            bind_group_layouts: &[],
            immediate_size: 0,
        });

        let pipeline = device.create_render_pipeline(&RenderPipelineDescriptor {
            label: Some("ancleto-rect-pipeline"),
            layout: Some(&layout),
            vertex: VertexState {
                module: &shader,
                entry_point: Some("vs"),
                compilation_options: PipelineCompilationOptions::default(),
                buffers: &[Some(VertexBufferLayout {
                    array_stride: std::mem::size_of::<Vertex>() as wgpu::BufferAddress,
                    step_mode: wgpu::VertexStepMode::Vertex,
                    attributes: &[
                        wgpu::VertexAttribute {
                            format: wgpu::VertexFormat::Float32x2,
                            offset: 0,
                            shader_location: 0,
                        },
                        wgpu::VertexAttribute {
                            format: wgpu::VertexFormat::Float32x4,
                            offset: 8,
                            shader_location: 1,
                        },
                    ],
                })],
            },
            fragment: Some(FragmentState {
                module: &shader,
                entry_point: Some("fs"),
                compilation_options: PipelineCompilationOptions::default(),
                targets: &[Some(ColorTargetState {
                    format,
                    blend: Some(BlendState::ALPHA_BLENDING),
                    write_mask: ColorWrites::ALL,
                })],
            }),
            primitive: PrimitiveState::default(),
            depth_stencil: None,
            multisample: MultisampleState::default(),
            multiview_mask: None,
            cache: None,
        });

        let capacity = 1024;
        let buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("ancleto-rect-vertices"),
            size: (capacity * 6 * std::mem::size_of::<Vertex>()) as u64,
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        Self {
            pipeline,
            buffer,
            capacity,
        }
    }

    pub fn prepare(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        rects: &[Rect],
        width: f32,
        height: f32,
    ) -> u32 {
        let mut vertices = Vec::with_capacity(rects.len() * 6);
        for rect in rects {
            if rect.width <= 0.0 || rect.height <= 0.0 {
                continue;
            }
            let x0 = rect.x / width * 2.0 - 1.0;
            let y0 = 1.0 - rect.y / height * 2.0;
            let x1 = (rect.x + rect.width) / width * 2.0 - 1.0;
            let y1 = 1.0 - (rect.y + rect.height) / height * 2.0;
            let corners = [
                ([x0, y0], rect.color),
                ([x1, y0], rect.color),
                ([x1, y1], rect.color),
                ([x0, y0], rect.color),
                ([x1, y1], rect.color),
                ([x0, y1], rect.color),
            ];
            for (position, color) in corners {
                vertices.push(Vertex { position, color });
            }
        }

        if vertices.len() > self.capacity * 6 {
            self.capacity = vertices.len() / 6 + 256;
            self.buffer = device.create_buffer(&wgpu::BufferDescriptor {
                label: Some("ancleto-rect-vertices"),
                size: (self.capacity * 6 * std::mem::size_of::<Vertex>()) as u64,
                usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
                mapped_at_creation: false,
            });
        }

        if !vertices.is_empty() {
            queue.write_buffer(&self.buffer, 0, bytemuck::cast_slice(&vertices));
        }

        vertices.len() as u32
    }

    pub fn render(&self, pass: &mut wgpu::RenderPass<'_>, count: u32) {
        if count == 0 {
            return;
        }
        pass.set_pipeline(&self.pipeline);
        pass.set_vertex_buffer(0, self.buffer.slice(..));
        pass.draw(0..count, 0..1);
    }
}

const SHADER: &str = r#"
struct VertexOut {
  @builtin(position) position: vec4<f32>,
  @location(0) color: vec4<f32>,
};

@vertex
fn vs(@location(0) position: vec2<f32>, @location(1) color: vec4<f32>) -> VertexOut {
  return VertexOut(vec4<f32>(position, 0.0, 1.0), color);
}

@fragment
fn fs(in: VertexOut) -> @location(0) vec4<f32> {
  return in.color;
}
"#;

pub fn rgba((r, g, b): (u8, u8, u8), alpha: f32) -> [f32; 4] {
    [r as f32 / 255.0, g as f32 / 255.0, b as f32 / 255.0, alpha]
}
