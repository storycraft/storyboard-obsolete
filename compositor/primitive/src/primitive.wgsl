// Created on Thu Sep 16 2021
//
// Copyright (c) storycraft. Licensed under the MIT Licence.

struct VertexOutput {
    [[builtin(position)]] position: vec4<f32>;
    [[location(0)]] color: vec4<f32>;
    [[location(1)]] tex_coord: vec2<f32>;
};

[[stage(vertex)]]
fn vs_main(
    [[location(0)]] position: vec3<f32>,
    [[location(1)]] color: vec4<f32>,
    [[location(2)]] tex_coord: vec2<f32>
) -> VertexOutput {
    var out: VertexOutput;

    out.position = vec4<f32>(position, 1.0);
    out.color = color;
    out.tex_coord = tex_coord;
    return out;
}

[[group(0), binding(0)]]
var texture: texture_2d<f32>;
[[group(0), binding(1)]]
var texture_sampler: sampler;

[[stage(fragment)]]
fn fs_main(in: VertexOutput) -> [[location(0)]] vec4<f32> {
    let color = in.color * select(
        vec4<f32>(1.0, 1.0, 1.0, 1.0),
        textureSample(texture, texture_sampler, in.tex_coord),
        in.tex_coord.x >= 0.0 && in.tex_coord.y >= 0.0 && in.tex_coord.x <= 1.0 && in.tex_coord.y <= 1.0
    );

    return color;
}
