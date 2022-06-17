struct VertexOutput {
    [[builtin(position)]] position: vec4<f32>;
    [[location(0)]] color: vec4<f32>;
    [[location(1)]] tex_coord: vec2<f32>;
    [[location(2)]] mask_coord: vec2<f32>;
};

[[stage(vertex)]]
fn vs_main(
    [[location(0)]] position: vec3<f32>,
    [[location(1)]] color: vec4<f32>,
    [[location(2)]] tex_coord: vec2<f32>,
    [[location(3)]] mask_coord: vec2<f32>
) -> VertexOutput {
    var out: VertexOutput;

    out.position = vec4<f32>(position, 1.0);
    out.color = color;
    out.tex_coord = tex_coord;
    out.mask_coord = mask_coord;

    return out;
}

[[group(0), binding(0)]]
var texture: texture_2d<f32>;
[[group(0), binding(1)]]
var texture_sampler: sampler;

[[group(1), binding(0)]]
var mask: texture_2d<f32>;
[[group(1), binding(1)]]
var mask_sampler: sampler;

[[stage(fragment)]]
fn fs_main(in: VertexOutput) -> [[location(0)]] vec4<f32> {
    var color = in.color * textureSample(texture, texture_sampler, in.tex_coord);
    let mask_sample = textureSample(mask, mask_sampler, in.mask_coord);

    color.a = color.a * mask_sample.r;

    return color;
}
