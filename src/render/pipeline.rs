use std::num::NonZero;
use wgpu::*;
use crate::render::vertex::Vertex;

pub struct Pipeline {
    pub pipeline: RenderPipeline,
    pub bind_group_layout: BindGroupLayout,
    pub uniform_buf: Buffer,
    pub bind_group: BindGroup,
    pub vertex_buf: Buffer,
    pub texture: Texture,
    pub texture_view: TextureView,
    pub sampler: Sampler,
    pub atlas_size: u32,
}

impl Pipeline {
    pub fn new(device: &Device, format: TextureFormat, atlas_size: u32) -> Self {
        let shader = device.create_shader_module(include_wgsl!("shaders/ui.wgsl"));

        let bind_group_layout = device.create_bind_group_layout(&BindGroupLayoutDescriptor {
            label: None,
            entries: &[
                BindGroupLayoutEntry {
                    binding: 0,
                    visibility: ShaderStages::VERTEX,
                    ty: BindingType::Buffer {
                        ty: BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                BindGroupLayoutEntry {
                    binding: 1,
                    visibility: ShaderStages::FRAGMENT,
                    ty: BindingType::Texture {
                        sample_type: TextureSampleType::Float { filterable: true },
                        view_dimension: TextureViewDimension::D2,
                        multisampled: false,
                    },
                    count: None,
                },
                BindGroupLayoutEntry {
                    binding: 2,
                    visibility: ShaderStages::FRAGMENT,
                    ty: BindingType::Sampler(SamplerBindingType::Filtering),
                    count: None,
                },
            ],
        });

        let pipeline_layout = device.create_pipeline_layout(&PipelineLayoutDescriptor {
            label: None,
            // wgpu 29: bind_group_layouts takes &[Option<&BindGroupLayout>]
            bind_group_layouts: &[Some(&bind_group_layout)],
            // wgpu 29: push_constant_ranges is gone; replaced by immediate_size
            immediate_size: 0,
        });

        let pipeline = device.create_render_pipeline(&RenderPipelineDescriptor {
            label: None,
            layout: Some(&pipeline_layout),
            vertex: VertexState {
                module: &shader,
                // wgpu 29: entry_point is Option<&str>
                entry_point: Some("vs_main"),
                buffers: &[Vertex::desc()],
                // wgpu 29: compilation_options required
                compilation_options: PipelineCompilationOptions::default(),
            },
            fragment: Some(FragmentState {
                module: &shader,
                entry_point: Some("fs_main"),
                targets: &[Some(ColorTargetState {
                    format,
                    blend: Some(BlendState::ALPHA_BLENDING),
                    write_mask: ColorWrites::ALL,
                })],
                // wgpu 29: compilation_options required
                compilation_options: PipelineCompilationOptions::default(),
            }),
            primitive: PrimitiveState::default(),
            depth_stencil: None,
            multisample: MultisampleState::default(),
            multiview_mask: NonZero::new(0),
            cache: None,
        });

        let uniform_buf = device.create_buffer(&BufferDescriptor {
            label: None,
            size: 64,
            usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let texture = device.create_texture(&TextureDescriptor {
            label: None,
            size: Extent3d { width: atlas_size, height: atlas_size, depth_or_array_layers: 1 },
            mip_level_count: 1,
            sample_count: 1,
            dimension: TextureDimension::D2,
            format: TextureFormat::Rgba8Unorm,
            usage: TextureUsages::TEXTURE_BINDING | TextureUsages::COPY_DST,
            view_formats: &[],
        });

        let texture_view = texture.create_view(&TextureViewDescriptor::default());

        let sampler = device.create_sampler(&SamplerDescriptor {
            address_mode_u: AddressMode::ClampToEdge,
            address_mode_v: AddressMode::ClampToEdge,
            mag_filter: FilterMode::Linear,
            min_filter: FilterMode::Linear,
            ..Default::default()
        });

        let bind_group = device.create_bind_group(&BindGroupDescriptor {
            label: None,
            layout: &bind_group_layout,
            entries: &[
                BindGroupEntry { binding: 0, resource: uniform_buf.as_entire_binding() },
                BindGroupEntry { binding: 1, resource: BindingResource::TextureView(&texture_view) },
                BindGroupEntry { binding: 2, resource: BindingResource::Sampler(&sampler) },
            ],
        });

        let vertex_buf = device.create_buffer(&BufferDescriptor {
            label: None,
            size: (98304 * std::mem::size_of::<Vertex>()) as u64,
            usage: BufferUsages::VERTEX | BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        Self {
            pipeline,
            bind_group_layout,
            uniform_buf,
            bind_group,
            vertex_buf,
            texture,
            texture_view,
            sampler,
            atlas_size,
        }
    }

    pub fn update_atlas(&self, queue: &Queue, pixels: &[u8]) {
        // wgpu 29: ImageCopyTexture → TexelCopyTextureInfo, ImageDataLayout → TexelCopyBufferLayout
        queue.write_texture(
            TexelCopyTextureInfo {
                texture: &self.texture,
                mip_level: 0,
                origin: Origin3d::ZERO,
                aspect: TextureAspect::All,
            },
            pixels,
            TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(self.atlas_size * 4),
                rows_per_image: None,
            },
            Extent3d { width: self.atlas_size, height: self.atlas_size, depth_or_array_layers: 1 },
        );
    }

    pub fn update_projection(&self, queue: &Queue, width: f32, height: f32) {
        let matrix: [f32; 16] = [
            2.0 / width, 0.0, 0.0, 0.0,
            0.0, -2.0 / height, 0.0, 0.0,
            0.0, 0.0, -1.0, 0.0,
            -1.0, 1.0, 0.0, 1.0,
        ];
        queue.write_buffer(&self.uniform_buf, 0, bytemuck::bytes_of(&matrix));
    }
}