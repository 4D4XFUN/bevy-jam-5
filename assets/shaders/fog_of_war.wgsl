#import bevy_sprite::mesh2d_vertex_output::VertexOutput

@group(2) @binding(0)
var<uniform> color: vec4<f32>;
@group(2) @binding(1)
var fog_texture: texture_2d<f32>;
@group(2) @binding(2)
var fog_sampler: sampler;

@fragment
fn fragment(in: VertexOutput) -> @location(0) vec4<f32> {
    let fog_value = textureSample(fog_texture, fog_sampler, in.uv).r;
    return vec4<f32>(color.rgb, color.a * fog_value);
}