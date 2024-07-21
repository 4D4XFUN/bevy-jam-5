#import bevy_sprite::mesh2d_view_bindings
#import bevy_pbr::utils

@group(1) @binding(0) var<uniform> material: FovMaterial;

struct FovMaterial {
    color: vec4<f32>,
    arc_params: vec4<f32>,
};

@fragment
fn fragment(
    #import bevy_sprite::mesh2d_fragment_input
) -> @location(0) vec4<f32> {
    let uv = (in.uv - 0.5) * 2.0;
    let dist = length(uv);
    let angle = atan2(uv.y, uv.x);

    let start_angle = material.arc_params.x;
    let end_angle = material.arc_params.y;
    let inner_radius = material.arc_params.z;
    let outer_radius = material.arc_params.w;

    if (angle >= start_angle && angle <= end_angle && dist >= inner_radius && dist <= outer_radius) {
        return material.color;
    } else {
        return vec4<f32>(0.0, 0.0, 0.0, 0.0);
    }
}