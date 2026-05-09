use bytemuck::{Pod, Zeroable};

#[repr(C)]
#[derive(Copy, Clone, Pod, Zeroable)]
pub struct Vertex {
    pub position: [f32; 2],
    pub uv: [f32; 2],
    pub local: [f32; 2],
    pub shape: [f32; 4],
    pub color: [u8; 4],
}

impl Vertex {
    pub const ATTRIBS: [wgpu::VertexAttribute; 5] = wgpu::vertex_attr_array![
        0 => Float32x2,
        1 => Float32x2,
        2 => Float32x2,
        3 => Float32x4,
        4 => Unorm8x4,
    ];

    pub fn desc() -> wgpu::VertexBufferLayout<'static> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<Vertex>() as u64,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &Self::ATTRIBS,
        }
    }
}

pub struct Batch {
    vertices: Vec<Vertex>,
}

impl Batch {
    pub fn new() -> Self {
        Self { vertices: Vec::with_capacity(16384) }
    }

    pub fn clear(&mut self) {
        self.vertices.clear();
    }

    pub fn len(&self) -> usize {
        self.vertices.len()
    }

    pub fn data(&self) -> &[Vertex] {
        &self.vertices
    }

    pub fn push_quad(
        &mut self,
        x: f32, y: f32, w: f32, h: f32,
        u0: f32, v0: f32, u1: f32, v1: f32,
        half_w: f32, half_h: f32,
        radius: f32, mode: f32,
        color: [u8; 4],
    ) {
        let shape = [half_w - radius, half_h - radius, radius, mode];
        let verts = [
            Vertex { position: [x,     y    ], uv: [u0, v0], local: [-half_w, -half_h], shape, color },
            Vertex { position: [x + w, y    ], uv: [u1, v0], local: [ half_w, -half_h], shape, color },
            Vertex { position: [x + w, y + h], uv: [u1, v1], local: [ half_w,  half_h], shape, color },
            Vertex { position: [x,     y + h], uv: [u0, v1], local: [-half_w,  half_h], shape, color },
        ];
        self.vertices.push(verts[0]);
        self.vertices.push(verts[1]);
        self.vertices.push(verts[2]);
        self.vertices.push(verts[2]);
        self.vertices.push(verts[3]);
        self.vertices.push(verts[0]);
    }
}