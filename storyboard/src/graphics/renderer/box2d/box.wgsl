// Created on Thu Sep 16 2021
//
// Copyright (c) storycraft. Licensed under the MIT Licence.

struct VertexOutput {
    [[builtin(position)]] position: vec4<f32>;
    [[location(0)]] color: vec4<f32>;
    [[location(1)]] border_color: vec4<f32>;
    [[location(2)]] tex_coord: vec2<f32>;
    [[location(3)]] rect_coord: vec2<f32>;

    [[location(4), interpolate(flat)]] rect_size: vec2<f32>;
    [[location(5), interpolate(flat)]] border_radius: f32;
    [[location(6), interpolate(flat)]] border_thickness: f32;
};

[[stage(vertex)]]
fn vs_main(
    [[location(0)]] position: vec3<f32>,
    [[location(1)]] color: vec4<f32>,
    [[location(2)]] border_color: vec4<f32>,
    [[location(3)]] tex_coord: vec2<f32>,
    [[location(4)]] rect_coord: vec2<f32>,

    [[location(5)]] rect_size: vec2<f32>,
    [[location(6)]] border_radius: f32,
    [[location(7)]] border_thickness: f32
) -> VertexOutput {
    var out: VertexOutput;

    out.position = vec4<f32>(position, 1.0);
    out.color = color;
    out.border_color = border_color;
    out.tex_coord = tex_coord;
    out.rect_coord = rect_coord;

    out.rect_size = rect_size;
    out.border_radius = border_radius;
    out.border_thickness = border_thickness;
    return out;
}

[[group(0), binding(0)]]
var texture: texture_2d<f32>;
[[group(0), binding(1)]]
var texture_sampler: sampler;

[[stage(fragment)]]
fn fs_main(in: VertexOutput) -> [[location(0)]] vec4<f32> {
    let rect_center = in.rect_size.xy * vec2<f32>(0.5, 0.5);
    let radius_padding = vec2<f32>(in.border_radius, in.border_radius);

    let rect_distance = max(abs(rect_center - in.rect_coord) - rect_center + radius_padding, vec2<f32>(0.0, 0.0));
    let distance = pow(rect_distance.x * rect_distance.x + rect_distance.y * rect_distance.y, 0.5);

    let border_all = in.border_radius + in.border_thickness;
    let border_smoothing_start = max(border_all - 1.0, 0.0);

    let smoothing_alpha = select(
        1.0,
        border_all - distance,
        distance > border_smoothing_start && distance <= border_all
    );

    var color = select(
        select(
            vec4<f32>(0.0, 0.0, 0.0, 0.0),
            in.color * select(
                vec4<f32>(1.0, 1.0, 1.0, 1.0),
                textureSample(texture, texture_sampler, in.tex_coord),
                in.tex_coord.x >= 0.0 && in.tex_coord.y >= 0.0 && in.tex_coord.x <= 1.0 && in.tex_coord.y <= 1.0
            ),
            distance <= in.border_radius
        ),
        in.border_color,
        distance > in.border_radius && distance <= border_all
    );

    color.w = color.w * smoothing_alpha;

    return color;
}
