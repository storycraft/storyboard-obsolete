struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) fill_color: vec4<f32>,
    @location(2) border_color: vec4<f32>,
    @location(3) rect_coord: vec2<f32>,
    @location(4) texture_coord: vec2<f32>,
};

struct InstanceInput {
    @location(5) rect: vec4<f32>,
    @location(6) texture_rect: vec4<f32>,
    @location(7) texture_wrap_mode: vec2<u32>,
    @location(8) border_radius: vec4<f32>,
    @location(9) border_thickness: f32,
    @location(10) glow_radius: f32,
    @location(11) glow_color: vec4<f32>,
    @location(12) shadow_offset: vec2<f32>,
    @location(13) shadow_radius: f32,
    @location(14) shadow_color: vec4<f32>,
};

struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) fill_color: vec4<f32>,
    @location(1) border_color: vec4<f32>,
    @location(2) glow_color: vec4<f32>,
    @location(3) shadow_color: vec4<f32>,
    @location(4) rect_coord: vec2<f32>,
    @location(5) texture_coord: vec2<f32>,

    @location(6) @interpolate(flat) rect: vec4<f32>,
    @location(7) @interpolate(flat) texture_rect: vec4<f32>,
    @location(8) @interpolate(flat) texture_wrap_mode: vec2<u32>,
    @location(9) @interpolate(flat) border_radius: vec4<f32>,
    @location(10) @interpolate(flat) border_thickness: f32,
    @location(11) @interpolate(flat) glow_radius: f32,
    @location(12) @interpolate(flat) shadow_offset: vec2<f32>,
    @location(13) @interpolate(flat) shadow_radius: f32,
};

@vertex
fn vs_main(
    vertex: VertexInput,
    instance: InstanceInput
) -> VertexOutput {
    var out: VertexOutput;

    out.position = vec4<f32>(vertex.position, 1.0);
    out.fill_color = vertex.fill_color;
    out.border_color = vertex.border_color;
    out.rect_coord = vertex.rect_coord;
    out.texture_coord = vertex.texture_coord;

    out.rect = instance.rect;
    out.texture_rect = instance.texture_rect;
    out.texture_wrap_mode = instance.texture_wrap_mode;
    out.border_radius = instance.border_radius;
    out.border_thickness = instance.border_thickness;
    out.glow_radius = instance.glow_radius;
    out.glow_color = instance.glow_color;
    out.shadow_offset = instance.shadow_offset;
    out.shadow_radius = instance.shadow_radius;
    out.shadow_color = instance.shadow_color;

    return out;
}

@group(0) @binding(0)
var texture: texture_2d<f32>;
@group(0) @binding(1)
var texture_sampler: sampler;

// Returns vec3(distanceX, distanceY, borderRadius)
fn box2d(rect: vec4<f32>, border_radius: vec4<f32>, coord: vec2<f32>) -> vec3<f32> {
    let half_size = rect.zw / 2.0;
    let center = rect.xy + half_size;

    let radius = border_radius[u32(center.y - coord.y <= 0.0) * 2u + u32(center.x - coord.x <= 0.0)];

    let dist = max(abs(center - coord) - half_size + radius, vec2<f32>(0.0, 0.0));

    return vec3<f32>(dist, radius);
}

fn box_distance(box2d: vec3<f32>) -> f32 {
    return max(sqrt(dot(box2d.xy, box2d.xy)) - box2d.z, 0.0);
}

fn blend(source: vec4<f32>, dest: vec4<f32>, alpha: f32) -> vec4<f32> {
    return vec4<f32>(source.xyz * (1.0 - alpha) + dest.xyz * alpha, alpha);
}

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
    let box = box2d(in.rect, in.border_radius, in.rect_coord);
    let box_dist = box_distance(box);

    let shadow_box = box2d(in.rect, in.border_radius, in.rect_coord - in.shadow_offset);
    let shadow_box_dist = box_distance(shadow_box);

    var color = vec4<f32>(0.0, 0.0, 0.0, 0.0);

    let fill_color = in.fill_color * mapped_texture_color(texture, texture_sampler, in.texture_wrap_mode, in.texture_rect, in.texture_coord);

    // Shadow
    if (shadow_box_dist <= in.shadow_radius) {
        let t = select(0.0, shadow_box_dist / in.shadow_radius, in.shadow_radius != 0.0);
        color = blend(color, in.shadow_color, 1.0 - t * t);
    }

    // Glow
    if (box_dist <= in.border_thickness + in.glow_radius && box_dist > in.border_thickness) {
        let t = select(0.0, (box_dist - in.border_thickness) / in.glow_radius, in.glow_radius != 0.0);
        color = blend(color, in.glow_color, 1.0 - t * t);
    }

    // Border
    if (box_dist < in.border_thickness + 1.0 && box_dist > 0.0) {
        let t = max(box_dist - in.border_thickness, 0.0);
        color = blend(color, in.border_color, 1.0 - t * t);
    }

    // Fill Color
    if (box_dist < 1.0) {
        color = blend(color, fill_color, 1.0 - box_dist * box_dist);
    }

    return color;
}
