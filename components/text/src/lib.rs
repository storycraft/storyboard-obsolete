pub mod cache;
pub mod component;
pub mod font;
pub mod layout;

use font::Font;

use ab_glyph_rasterizer::{Point, Rasterizer};
use storyboard_core::{
    euclid::{Point2D, Vector2D},
    unit::LogicalPixelUnit,
};
use ttf_parser::OutlineBuilder;

#[derive(Debug)]
pub struct GlyphOutlineBuilder {
    rasterizer: Rasterizer,
    origin: Vector2D<f32, LogicalPixelUnit>,
    scale: f32,
    point: Point2D<f32, LogicalPixelUnit>,
    last_move_point: Option<Point2D<f32, LogicalPixelUnit>>,
}

impl GlyphOutlineBuilder {
    pub fn new(font: &Font, bounding_box: ttf_parser::Rect, size_px: u32) -> Self {
        let scale = size_px as f32 / (font.units_per_em() as f32);

        Self {
            rasterizer: Rasterizer::new(
                (bounding_box.width() as f32 * scale).ceil() as usize,
                (bounding_box.height() as f32 * scale).ceil() as usize,
            ),
            origin: Vector2D::new(bounding_box.x_min as f32, bounding_box.y_min as f32),
            scale,
            point: Point2D::new(0.0, 0.0),
            last_move_point: None,
        }
    }

    #[inline]
    fn to_pixel_point(&self, x: f32, y: f32) -> Point {
        Point {
            x: (x - self.origin.x) * self.scale,
            y: self.rasterizer.dimensions().1 as f32 - (y - self.origin.y) * self.scale,
        }
    }

    pub fn into_rasterizer(self) -> Rasterizer {
        self.rasterizer
    }
}

impl OutlineBuilder for GlyphOutlineBuilder {
    fn move_to(&mut self, x: f32, y: f32) {
        self.point = Point2D::new(x, y);
        self.last_move_point = Some(self.point);
    }

    fn line_to(&mut self, x: f32, y: f32) {
        self.rasterizer.draw_line(
            self.to_pixel_point(self.point.x, self.point.y),
            self.to_pixel_point(x, y),
        );

        self.point = Point2D::new(x, y);
    }

    fn quad_to(&mut self, x1: f32, y1: f32, x: f32, y: f32) {
        self.rasterizer.draw_quad(
            self.to_pixel_point(self.point.x, self.point.y),
            self.to_pixel_point(x1, y1),
            self.to_pixel_point(x, y),
        );

        self.point = Point2D::new(x, y);
    }

    fn curve_to(&mut self, x1: f32, y1: f32, x2: f32, y2: f32, x: f32, y: f32) {
        self.rasterizer.draw_cubic(
            self.to_pixel_point(self.point.x, self.point.y),
            self.to_pixel_point(x1, y1),
            self.to_pixel_point(x2, y2),
            self.to_pixel_point(x, y),
        );

        self.point = Point2D::new(x, y);
    }

    fn close(&mut self) {
        if let Some(point) = self.last_move_point {
            self.line_to(point.x, point.y);
        }
    }
}
