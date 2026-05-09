// In src/render/shaders/ui.wgsl:

struct Uniforms {
    projection: mat4x4<f32>,
}

@group(0) @binding(0) var<uniform> uniforms: Uniforms;
@group(0) @binding(1) var atlas: texture_2d<f32>;
@group(0) @binding(2) var samp: sampler;

struct VertexInput {
    @location(0) position: vec2<f32>,
    @location(1) uv: vec2<f32>,
    @location(2) local: vec2<f32>,
    @location(3) shape: vec4<f32>,
    @location(4) color: vec4<f32>,
}

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) uv: vec2<f32>,
    @location(1) local: vec2<f32>,
    @location(2) shape: vec4<f32>,
    @location(3) color: vec4<f32>,
}

@vertex
fn vs_main(in: VertexInput) -> VertexOutput {
    var out: VertexOutput;
    out.clip_position = uniforms.projection * vec4<f32>(in.position, 0.0, 1.0);
    out.uv = in.uv;
    out.local = in.local;
    out.shape = in.shape;
    out.color = in.color;
    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let base = in.color * textureSample(atlas, samp, in.uv);
    let mode = in.shape.w;

    if mode == 0.0 {
        return base;
    }

    let q = abs(in.local) - in.shape.xy;
    let field = length(max(q, vec2<f32>(0.0))) + min(max(q.x, q.y), 0.0) - in.shape.z;

    if mode == 1.0 {
        let sm = fwidth(field) * 1.5;
        let alpha = 1.0 - smoothstep(-sm, sm, field);
        return vec4<f32>(base.rgb, base.a * alpha);
    }

    if mode == 3.0 {
        let border = abs(field + 0.25) - 0.25;
        let sm = fwidth(field) * 1.5;
        let alpha = 1.0 - smoothstep(-sm, sm, border);
        return vec4<f32>(base.rgb, base.a * alpha);
    }

    if mode == 2.0 {
        let norm = in.local / in.shape.xy;
        let ef = length(norm) - 1.0;
        let sm = fwidth(ef) * 1.5;
        let alpha = 1.0 - smoothstep(-sm, sm, ef);
        return vec4<f32>(base.rgb, base.a * alpha);
    }

    return base;
}