// Created on Thu Sep 16 2021
//
// Copyright (c) storycraft. Licensed under the MIT Licence.

struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) color: vec4<f32>,
    @location(1) texture_coord: vec2<f32>,

    @location(2) @interpolate(flat) texture_rect: vec4<f32>,
};

@vertex
fn vs_main(
    @location(0) position: vec3<f32>,
    @location(1) color: vec4<f32>,
    @location(2) texture_coord: vec2<f32>,

    @location(3) texture_rect: vec4<f32>
) -> VertexOutput {
    var out: VertexOutput;

    out.position = vec4<f32>(position, 1.0);
    out.color = color;
    out.texture_coord = texture_coord;

    out.texture_rect = texture_rect;

    return out;
}

@group(0) @binding(0) var texture: texture_2d<f32>;
@group(0) @binding(1) var texture_sampler: sampler;

fn wrapped_texture_coord(rect: vec4<f32>, coord: vec2<f32>) -> vec2<f32> {
    return (coord - rect.xy) / rect.zw;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let color = in.color * textureSample(texture, texture_sampler, wrapped_texture_coord(in.texture_rect, in.texture_coord));

    return color;
}
