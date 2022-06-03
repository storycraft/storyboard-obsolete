// Created on Thu Sep 16 2021
//
// Copyright (c) storycraft. Licensed under the MIT Licence.

struct VertexOutput {
    [[builtin(position)]] position: vec4<f32>;
    [[location(0)]] color: vec4<f32>;
};

[[stage(vertex)]]
struct PathInstance {
    [[location(2)]] matrix_0: vec4<f32>;
    [[location(3)]] matrix_1: vec4<f32>;
    [[location(4)]] matrix_2: vec4<f32>;
    [[location(5)]] matrix_3: vec4<f32>;
};

[[stage(vertex)]]
fn vs_main(
    [[location(0)]] position: vec3<f32>,
    [[location(1)]] color: vec4<f32>,
    instance: PathInstance
) -> VertexOutput {
    var out: VertexOutput;

    var pos = mat4x4<f32>(
        instance.matrix_0,
        instance.matrix_1,
        instance.matrix_2,
        instance.matrix_3
    ) * vec4<f32>(position, 1.0);

    pos.z = position.z;
    pos.w = 1.0;

    out.position = pos;
    out.color = color;

    return out;
}

[[stage(fragment)]]
fn fs_main(in: VertexOutput) -> [[location(0)]] vec4<f32> {
    return in.color;
}
