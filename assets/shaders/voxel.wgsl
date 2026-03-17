#import bevy_pbr::{
    mesh_functions,
    view_transformations::position_world_to_clip
}

struct Vertex {
    @builtin(instance_index) instance_index: u32,
    @location(0) position: vec3<f32>,
    @location(1) normal: vec3<f32>,
    @location(2) uv: vec2<f32>,
    @location(5) light: vec2<f32>,
};

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) uv: vec2<f32>,
    @location(1) light: vec2<f32>,
    @location(2) world_pos: vec3<f32>,
};

@group(3) @binding(0) var voxel_texture: texture_2d<f32>;
@group(3) @binding(1) var voxel_sampler: sampler;
@group(3) @binding(2) var<uniform> sky_color: vec4<f32>;
@group(3) @binding(3) var<uniform> fog_color: vec4<f32>;
@group(3) @binding(4) var<uniform> fog_params: vec4<f32>;
@group(3) @binding(5) var<uniform> sun_params: vec4<f32>;

@vertex
fn vertex(v: Vertex) -> VertexOutput {
    var out: VertexOutput;

    let world_from_local = mesh_functions::get_world_from_local(v.instance_index);
    let world_pos4 =
        mesh_functions::mesh_position_local_to_world(world_from_local, vec4<f32>(v.position, 1.0));

    out.clip_position = position_world_to_clip(world_pos4.xyz);
    out.uv = v.uv;
    out.light = v.light;
    out.world_pos = world_pos4.xyz;

    return out;
}

@fragment
fn fragment(in: VertexOutput) -> @location(0) vec4<f32> {
    let tex = textureSample(voxel_texture, voxel_sampler, in.uv);

    let sunlight = pow(in.light.x, 1.5);
    let blocklight = pow(in.light.y, 1.3);
    
    let sun_strength = sun_params.x;

    let torch_color = vec3<f32>(1.0, 0.80, 0.55);
    let sky = sky_color.xyz;

    let lit = sunlight * sky * sun_strength + blocklight * torch_color;
    let ambient = vec3<f32>(0.01, 0.01, 0.012);
    let final_light = clamp(lit + ambient, vec3<f32>(0.0), vec3<f32>(1.0));

    let lit_rgb = tex.rgb * final_light;

    return vec4<f32>(lit_rgb, tex.a);
}