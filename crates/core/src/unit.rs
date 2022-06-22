#[derive(Debug, Clone, Copy)]
/// top-left origin 1:n matching logical Pixel unit.
pub struct LogicalPixelUnit;

#[derive(Debug, Clone, Copy)]
/// top-left origin 1:1 matching physical Pixel unit.
pub struct PhyiscalPixelUnit;

#[derive(Debug, Clone, Copy)]
/// Unit projected with ortho projection. [-1.0, 1.0]
pub struct RenderUnit;

#[derive(Debug, Clone, Copy)]
/// Unit on wgpu texture. top-left origin UV unit. [0.0, 1.0]
pub struct TextureUnit;
