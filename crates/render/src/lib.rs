//! Core rendering abtraction module

// Reexports
pub use wgpu;

pub mod backend;
pub mod buffer;
pub mod component;
pub mod renderer;
pub mod task;
pub mod texture;
pub mod cache;

use storyboard_core::{unit::{PhyiscalPixelUnit, LogicalPixelUnit, RenderUnit}, euclid::{Rect, Size2D, Transform3D}};

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ScreenRect {
    pub rect: Rect<u32, PhyiscalPixelUnit>,
    pub scale_factor: f32,
}

impl ScreenRect {
    pub const fn new(rect: Rect<u32, PhyiscalPixelUnit>, scale_factor: f32) -> Self {
        Self {
            rect,
            scale_factor
        }
    }

    pub fn get_logical_size(&self) -> Size2D<f32, LogicalPixelUnit> {
        (self.rect.size.cast::<f32>() / self.scale_factor)
            .ceil()
            .cast_unit()
    }

    pub fn get_logical_rect(&self) -> Rect<f32, LogicalPixelUnit> {
        Rect::new(self.rect.origin.cast().cast_unit(), self.get_logical_size())
    }

    pub fn get_logical_ortho_matrix(&self) -> Transform3D<f32, LogicalPixelUnit, RenderUnit> {
        Transform3D::ortho(
            self.rect.origin.x as f32,
            self.rect.origin.x as f32 + self.rect.size.width as f32 / self.scale_factor,
            self.rect.origin.y as f32 + self.rect.size.height as f32 / self.scale_factor,
            self.rect.origin.y as f32,
            0.0,
            1.0,
        )
    }
}
