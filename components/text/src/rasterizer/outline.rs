use ab_glyph_rasterizer::{Point, Rasterizer};
use storyboard_core::{
    euclid::{Rect, Size2D, Vector2D},
    unit::PhyiscalPixelUnit,
};
use ttf_parser::OutlineBuilder;

use crate::{rasterizer::GlyphData, FontUnit};

#[derive(Debug)]
pub struct GlyphOutlineBuilder {
    bounds: Rect<f32, PhyiscalPixelUnit>,
    rasterizer: Rasterizer,
    scale: f32,
    point: Vector2D<f32, FontUnit>,
    last_move_point: Option<Vector2D<f32, FontUnit>>,
}

impl GlyphOutlineBuilder {
    pub fn new(bounds: Rect<f32, FontUnit>, scale: f32) -> Self {
        let mut bounds = bounds.scale(scale, scale).cast_unit();
        bounds.size.width = bounds.size.width.ceil();
        bounds.size.height = bounds.size.height.ceil();

        Self {
            bounds,
            rasterizer: Rasterizer::new(bounds.size.width as usize, bounds.size.height as usize),
            scale,
            point: Vector2D::zero(),
            last_move_point: None,
        }
    }

    #[inline]
    fn to_point(&self, vec: Vector2D<f32, FontUnit>) -> Point {
        Point {
            x: vec.x * self.scale - self.bounds.origin.x,
            y: self.bounds.size.height - vec.y * self.scale + self.bounds.origin.y,
        }
    }

    pub fn get_glyph_data(&self) -> GlyphData {
        let mut data: Vec<u8> =
            vec![0; self.rasterizer.dimensions().0 * self.rasterizer.dimensions().1];
        self.rasterizer
            .for_each_pixel(|i, alpha| data[i] = (alpha * 255.0) as u8);

        let mut origin = self.bounds.origin.to_vector();
        origin.y *= -1.0;

        let size = Size2D::new(
            self.rasterizer.dimensions().0 as u32,
            self.rasterizer.dimensions().1 as u32,
        );

        GlyphData {
            origin: origin.cast_unit(),
            size,
            data,
        }
    }
}

impl OutlineBuilder for GlyphOutlineBuilder {
    fn move_to(&mut self, x: f32, y: f32) {
        self.point = Vector2D::new(x, y);
        self.last_move_point = Some(self.point);
    }

    fn line_to(&mut self, x: f32, y: f32) {
        let from = self.point;
        let to = Vector2D::new(x, y);

        self.rasterizer
            .draw_line(self.to_point(from), self.to_point(to));
            
        self.point = to;
    }

    fn quad_to(&mut self, x1: f32, y1: f32, x: f32, y: f32) {
        let from = self.point;
        let ctrl = Vector2D::new(x1, y1);
        let to = Vector2D::new(x, y);

        self.rasterizer
            .draw_quad(self.to_point(from), self.to_point(ctrl), self.to_point(to));
            
        self.point = to;
    }

    fn curve_to(&mut self, x1: f32, y1: f32, x2: f32, y2: f32, x: f32, y: f32) {
        let from = self.point;
        let ctrl1 = Vector2D::new(x1, y1);
        let ctrl2 = Vector2D::new(x2, y2);
        let to = Vector2D::new(x, y);

        self.rasterizer.draw_cubic(
            self.to_point(from),
            self.to_point(ctrl1),
            self.to_point(ctrl2),
            self.to_point(to),
        );
        
        self.point = to;
    }

    fn close(&mut self) {
        if let Some(point) = self.last_move_point {
            self.line_to(point.x, point.y);
        }
    }
}
