//! El widget de video: pinta el buffer de mpv como una textura propia.
//!
//! Tres tipos, tres vidas:
//!
//! - [`Video`] es el `Program` del widget. Se crea en `view`, uno por frame.
//! - [`Frame`] es el dato de ese frame: píxeles y tamaño.
//! - [`Gpu`] es el estado de GPU (pipeline, textura, bind group). Se crea una
//!   vez por proceso y no depende de cuántos frames pasen.

use std::fmt;
use std::sync::Arc;

use iced::Rectangle;
use iced::wgpu;
use iced::widget::shader;

pub struct Video {
    pub pixels: Arc<Vec<u8>>,
    pub width: u32,
    pub height: u32,
}

impl<Message> shader::Program<Message> for Video {
    type State = ();
    type Primitive = Frame;

    fn draw(&self, _state: &(), _cursor: iced::mouse::Cursor, _bounds: Rectangle) -> Frame {
        Frame {
            pixels: Arc::clone(&self.pixels),
            width: self.width,
            height: self.height,
        }
    }
}

pub struct Frame {
    pixels: Arc<Vec<u8>>,
    width: u32,
    height: u32,
}

impl shader::Primitive for Frame {
    type Pipeline = Gpu;

    fn prepare(
        &self,
        gpu: &mut Gpu,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        bounds: &Rectangle,
        _viewport: &shader::Viewport,
    ) {
        gpu.ensure_texture(device, self.width, self.height);

        // Letterbox: el widget puede tener otra forma que el video. Se encoge el
        // quad en el eje que sobra en vez de deformarlo.
        let (escala_x, escala_y) = if bounds.width > 0.0 && bounds.height > 0.0 {
            let aspecto_widget = bounds.width / bounds.height;
            let aspecto_video = self.width as f32 / self.height as f32;

            if aspecto_video > aspecto_widget {
                (1.0, aspecto_widget / aspecto_video)
            } else {
                (aspecto_video / aspecto_widget, 1.0)
            }
        } else {
            (1.0, 1.0)
        };

        let mut escala = [0u8; 16];
        escala[0..4].copy_from_slice(&escala_x.to_le_bytes());
        escala[4..8].copy_from_slice(&escala_y.to_le_bytes());
        queue.write_buffer(&gpu.uniforms, 0, &escala);

        let Some(texture) = gpu.texture.as_ref() else {
            return;
        };

        queue.write_texture(
            wgpu::TexelCopyTextureInfo {
                texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            &self.pixels,
            wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(self.width * 4),
                rows_per_image: Some(self.height),
            },
            wgpu::Extent3d {
                width: self.width,
                height: self.height,
                depth_or_array_layers: 1,
            },
        );
    }

    fn draw(&self, gpu: &Gpu, render_pass: &mut wgpu::RenderPass<'_>) -> bool {
        let Some(bind_group) = gpu.bind_group.as_ref() else {
            return false;
        };

        render_pass.set_pipeline(&gpu.pipeline);
        render_pass.set_bind_group(0, bind_group, &[]);
        render_pass.draw(0..4, 0..1);

        true
    }
}

impl fmt::Debug for Frame {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Frame")
            .field("width", &self.width)
            .field("height", &self.height)
            .field("bytes", &self.pixels.len())
            .finish()
    }
}

pub struct Gpu {
    pipeline: wgpu::RenderPipeline,
    layout: wgpu::BindGroupLayout,
    sampler: wgpu::Sampler,
    uniforms: wgpu::Buffer,
    texture_format: wgpu::TextureFormat,
    texture: Option<wgpu::Texture>,
    bind_group: Option<wgpu::BindGroup>,
    size: (u32, u32),
}

impl shader::Pipeline for Gpu {
    fn new(device: &wgpu::Device, _queue: &wgpu::Queue, format: wgpu::TextureFormat) -> Self {
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("rod-app frame shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("frame.wgsl").into()),
        });

        let layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("rod-app frame layout"),
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        view_dimension: wgpu::TextureViewDimension::D2,
                        multisampled: false,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 2,
                    visibility: wgpu::ShaderStages::VERTEX,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
            ],
        });

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("rod-app frame pipeline layout"),
            bind_group_layouts: &[&layout],
            push_constant_ranges: &[],
        });

        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("rod-app frame pipeline"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs_main"),
                buffers: &[],
                compilation_options: Default::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: Some("fs_main"),
                targets: &[Some(wgpu::ColorTargetState {
                    format,
                    // El frame llega opaco: sin blending, se escribe tal cual.
                    blend: None,
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: Default::default(),
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleStrip,
                ..Default::default()
            },
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
            multiview: None,
            cache: None,
        });

        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("rod-app frame sampler"),
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            ..Default::default()
        });

        let uniforms = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("rod-app frame escala"),
            size: 16,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        // Si el target del frame es sRGB, la textura tiene que serlo también:
        // así la decodificación al muestrear y la codificación al escribir se
        // cancelan y los colores salen como los dio mpv.
        let texture_format = if format.is_srgb() {
            wgpu::TextureFormat::Rgba8UnormSrgb
        } else {
            wgpu::TextureFormat::Rgba8Unorm
        };

        Self {
            pipeline,
            layout,
            sampler,
            uniforms,
            texture_format,
            texture: None,
            bind_group: None,
            size: (0, 0),
        }
    }
}

impl Gpu {
    /// Crea la textura y su bind group la primera vez, y los rehace solo si el
    /// tamaño del frame cambia. `write_texture` reescribe el contenido.
    fn ensure_texture(&mut self, device: &wgpu::Device, width: u32, height: u32) {
        if self.size == (width, height) && self.texture.is_some() {
            return;
        }

        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("rod-app frame texture"),
            size: wgpu::Extent3d {
                width,
                height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: self.texture_format,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            view_formats: &[],
        });

        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());

        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("rod-app frame bind group"),
            layout: &self.layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&self.sampler),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: wgpu::BindingResource::Buffer(
                        self.uniforms.as_entire_buffer_binding(),
                    ),
                },
            ],
        });

        self.texture = Some(texture);
        self.bind_group = Some(bind_group);
        self.size = (width, height);
    }
}
