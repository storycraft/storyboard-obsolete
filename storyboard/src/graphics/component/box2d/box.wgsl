// Created on Thu Sep 16 2021
//
// Copyright (c) storycraft. Licensed under the MIT Licence.

struct VertexInput {
    [[location(0)]] position: vec3<f32>;
    [[location(1)]] fill_color: vec4<f32>;
    [[location(2)]] border_color: vec4<f32>;
    [[location(3)]] rect_coord: vec2<f32>;
};

struct InstanceInput {
    [[location(4)]] rect: vec4<f32>;
    [[location(5)]] texture_rect: vec4<f32>;
    [[location(6)]] border_radius: vec4<f32>;
    [[location(7)]] border_thickness: f32;
    [[location(8)]] glow_radius: f32;
    [[location(9)]] glow_color: vec4<f32>;
    [[location(10)]] shadow_offset: vec2<f32>;
    [[location(11)]] shadow_radius: f32;
    [[location(12)]] shadow_color: vec4<f32>;
};

struct VertexOutput {
    [[builtin(position)]] position: vec4<f32>;
    [[location(0)]] fill_color: vec4<f32>;
    [[location(1)]] border_color: vec4<f32>;
    [[location(2)]] glow_color: vec4<f32>;
    [[location(3)]] shadow_color: vec4<f32>;
    [[location(4)]] rect_coord: vec2<f32>;

    [[location(5), interpolate(flat)]] rect: vec4<f32>;
    [[location(6), interpolate(flat)]] texture_rect: vec4<f32>;
    [[location(7), interpolate(flat)]] border_radius: vec4<f32>;
    [[location(8), interpolate(flat)]] border_thickness: f32;
    [[location(9), interpolate(flat)]] glow_radius: f32;
    [[location(10), interpolate(flat)]] shadow_offset: vec2<f32>;
    [[location(11), interpolate(flat)]] shadow_radius: f32;
};

[[stage(vertex)]]
fn vs_main(
    vertex: VertexInput,
    instance: InstanceInput
) -> VertexOutput {
    var out: VertexOutput;

    out.position = vec4<f32>(vertex.position, 1.0);
    out.fill_color = vertex.fill_color;
    out.border_color = vertex.border_color;
    out.rect_coord = vertex.rect_coord;

    out.rect = instance.rect;
    out.texture_rect = instance.texture_rect;
    out.border_radius = instance.border_radius;
    out.border_thickness = instance.border_thickness;
    out.glow_radius = instance.glow_radius;
    out.glow_color = instance.glow_color;
    out.shadow_offset = instance.shadow_offset;
    out.shadow_radius = instance.shadow_radius;
    out.shadow_color = instance.shadow_color;

    return out;
}

[[group(0), binding(0)]]
var texture: texture_2d<f32>;
[[group(0), binding(1)]]
var texture_sampler: sampler;

// Returns vec3(distanceX, distanceY, borderRadius)
fn box2d(rect: vec4<f32>, border_radius: vec4<f32>, coord: vec2<f32>) -> vec3<f32> {
    let half_size = rect.zw / 2.0;
    let center = rect.xy + half_size;

    let radius = border_radius[u32(center.y - coord.y <= 0.0) * u32(2) + u32(center.x - coord.x <= 0.0)];

    let dist = max(abs(center - coord) - half_size + radius, vec2<f32>(0.0, 0.0));

    return vec3<f32>(dist, radius);
}

fn box_distance(box2d: vec3<f32>) -> f32 {
    return max(sqrt(dot(box2d.xy, box2d.xy)) - box2d.z, 0.0);
}

fn blend(source: vec4<f32>, dest: vec4<f32>, alpha: f32) -> vec4<f32> {
    return vec4<f32>(source.xyz * (1.0 - alpha) + dest.xyz * alpha, 1.0);
}

fn wrapped_texture_coord(rect: vec4<f32>, coord: vec2<f32>) -> vec2<f32> {
    return (coord - rect.xy) / rect.zw;
}

[[stage(fragment)]]
fn fs_main(in: VertexOutput) -> [[location(0)]] vec4<f32> {
    let box = box2d(in.rect, in.border_radius, in.rect_coord);
    let box_dist = box_distance(box);

    let shadow_box = box2d(in.rect, in.border_radius, in.rect_coord - in.shadow_offset);
    let shadow_box_dist = box_distance(shadow_box);

    var color = vec4<f32>(0.0, 0.0, 0.0, 1.0);

    let sampled_texture = textureSample(texture, texture_sampler, wrapped_texture_coord(vec4<f32>(in.texture_rect.xy - in.rect.xy, in.texture_rect.zw), in.rect_coord));
    let fill_color = in.fill_color * sampled_texture;

    // Shadow
    if (shadow_box_dist < in.shadow_radius) {
        let t = 1.0 - select(0.0, shadow_box_dist / in.shadow_radius, in.shadow_radius != 0.0);
        color = blend(color, in.shadow_color, in.shadow_color.w * t);
    }

    // Glow
    if (box_dist <= in.border_thickness + in.glow_radius && box_dist > in.border_thickness) {
        let t = 1.0 - select(0.0, (box_dist - in.border_thickness) / in.glow_radius, in.glow_radius != 0.0);
        color = blend(color, in.glow_color, in.glow_color.w * t);
    }

    // Border
    if (max(box.x, box.y) <= box.z + in.border_thickness && box_dist < in.border_thickness + 1.0 && box_dist > 0.0) {
        let t = 1.0 - max(box_dist - in.border_thickness, 0.0);
        color = blend(color, in.border_color, in.border_color.w * t);
    }

    // Fill Color
    if (max(box.x, box.y) <= box.z && box_dist < 1.0) {
        color = blend(color, fill_color, fill_color.w * (1.0 - box_dist));
    }

    return color;
}