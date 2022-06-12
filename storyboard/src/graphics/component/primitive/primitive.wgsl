// Created on Thu Sep 16 2021
//
// Copyright (c) storycraft. Licensed under the MIT Licence.

struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) color: vec4<f32>,
    @location(1) texture_coord: vec2<f32>,

    @location(2) @interpolate(flat) texture_rect: vec4<f32>,
    @location(3) @interpolate(flat) texture_wrap_mode: vec2<u32>,
};

@vertex
fn vs_main(
    @location(0) position: vec3<f32>,
    @location(1) color: vec4<f32>,
    @location(2) texture_coord: vec2<f32>,
    @location(3) texture_rect: vec4<f32>,
    @location(4) texture_wrap_mode: vec2<u32>
) -> VertexOutput {
    var out: VertexOutput;

    out.position = vec4<f32>(position, 1.0);
    out.color = color;
    out.texture_coord = texture_coord;

    out.texture_rect = texture_rect;
    out.texture_wrap_mode = texture_wrap_mode;

    return out;
}

@group(0) @binding(0) var texture: texture_2d<f32>;
@group(0) @binding(1) var texture_sampler: sampler;

fn wrap_texture_coord(coord: f32, wrap_mode: u32) -> f32 {
    if (wrap_mode == 1u) {
        return clamp(coord, 0.0, 1.0);
    } else if (wrap_mode == 2u) {
        return fract(coord);
    } else {
        return coord;
    }
}

fn mapped_texture_color(tex: texture_2d<f32>, tex_sampler: sampler, wrap_mode: vec2<u32>, tex_sub_rect: vec4<f32>, tex_coord: vec2<f32>) -> vec4<f32> {
    let coord = tex_sub_rect.xy + vec2<f32>(wrap_texture_coord(tex_coord.x, wrap_mode.x), wrap_texture_coord(tex_coord.y, wrap_mode.y)) * tex_sub_rect.zw;
    let tex_color = textureSample(tex, tex_sampler, coord);

    return select(
        vec4<f32>(1.0, 1.0, 1.0, 1.0),
        tex_color,
        coord.x >= tex_sub_rect.x && coord.y >= tex_sub_rect.y && coord.x <= tex_sub_rect.x + tex_sub_rect.z && coord.y <= tex_sub_rect.y + tex_sub_rect.w
    );
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let color = in.color * mapped_texture_color(texture, texture_sampler, in.texture_wrap_mode, in.texture_rect, in.texture_coord);
    return color;
}
