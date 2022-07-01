use ab_glyph_rasterizer::{Point, Rasterizer};
use smallvec::SmallVec;
use storyboard_core::euclid::{Box2D, Size2D, Vector2D};
use ttf_parser::OutlineBuilder;

use crate::{rasterizer::GlyphData, FontUnit};

#[derive(Debug)]
pub struct GlyphOutlineBuilder {
    bounding_box: Box2D<f32, FontUnit>,
    commands: SmallVec<[RasterizerCommand; 8]>,
    point: Vector2D<f32, FontUnit>,
    last_move_point: Option<Vector2D<f32, FontUnit>>,
}

impl GlyphOutlineBuilder {
    pub fn new() -> Self {
        Self {
            bounding_box: Box2D::zero(),
            commands: SmallVec::new(),
            point: Vector2D::zero(),
            last_move_point: None,
        }
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        self.commands.is_empty()
    }

    #[inline]
    fn to_point(&self, vec: Vector2D<f32, FontUnit>, scale: f32) -> Point {
        Point {
            x: (vec.x - self.bounding_box.min.x) * scale,
            y: (self.bounding_box.height() - (vec.y - self.bounding_box.min.y)) * scale,
        }
    }

    pub fn rasterize(&self, scale: f32) -> GlyphData {
        let mut rasterizer = Rasterizer::new(
            (self.bounding_box.width() * scale).ceil() as usize,
            (self.bounding_box.height() * scale).ceil() as usize,
        );

        for command in &self.commands {
            match command {
                RasterizerCommand::Line(from, to) => {
                    rasterizer.draw_line(self.to_point(*from, scale), self.to_point(*to, scale))
                }

                RasterizerCommand::QuadCurve(from, ctrl, to) => rasterizer.draw_quad(
                    self.to_point(*from, scale),
                    self.to_point(*ctrl, scale),
                    self.to_point(*to, scale),
                ),

                RasterizerCommand::CubicCurve(from, ctrl1, ctrl2, to) => rasterizer.draw_cubic(
                    self.to_point(*from, scale),
                    self.to_point(*ctrl1, scale),
                    self.to_point(*ctrl2, scale),
                    self.to_point(*to, scale),
                ),
            }
        }

        let mut data: Vec<u8> = vec![0; rasterizer.dimensions().0 * rasterizer.dimensions().1];
        rasterizer.for_each_pixel(|i, alpha| data[i] = (alpha * 255.0) as u8);

        let mut offset = self.bounding_box.min.to_vector() * scale;
        offset.y *= -1.0;

        let size = Size2D::new(
            rasterizer.dimensions().0 as u32,
            rasterizer.dimensions().1 as u32,
        );

        GlyphData {
            offset: offset.cast_unit(),
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

        let command = RasterizerCommand::Line(from, to);

        self.bounding_box = Box2D::from_points([
            from.to_point(),
            to.to_point(),
            self.bounding_box.min,
            self.bounding_box.max,
        ]);
        self.point = to;

        self.commands.push(command);
    }

    fn quad_to(&mut self, x1: f32, y1: f32, x: f32, y: f32) {
        let from = self.point;
        let ctrl = Vector2D::new(x1, y1);
        let to = Vector2D::new(x, y);

        let command = RasterizerCommand::QuadCurve(from, ctrl, to);

        self.bounding_box = Box2D::from_points([
            from.to_point(),
            ctrl.to_point(),
            to.to_point(),
            self.bounding_box.min,
            self.bounding_box.max,
        ]);
        self.point = to;

        self.commands.push(command);
    }

    fn curve_to(&mut self, x1: f32, y1: f32, x2: f32, y2: f32, x: f32, y: f32) {
        let from = self.point;
        let ctrl1 = Vector2D::new(x1, y1);
        let ctrl2 = Vector2D::new(x2, y2);
        let to = Vector2D::new(x, y);

        let command = RasterizerCommand::CubicCurve(from, ctrl1, ctrl2, to);

        self.bounding_box = Box2D::from_points([
            from.to_point(),
            ctrl1.to_point(),
            ctrl2.to_point(),
            to.to_point(),
            self.bounding_box.min,
            self.bounding_box.max,
        ]);
        self.point = to;

        self.commands.push(command);
    }

    fn close(&mut self) {
        if let Some(point) = self.last_move_point {
            self.line_to(point.x, point.y);
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum RasterizerCommand {
    Line(Vector2D<f32, FontUnit>, Vector2D<f32, FontUnit>),
    QuadCurve(
        Vector2D<f32, FontUnit>,
        Vector2D<f32, FontUnit>,
        Vector2D<f32, FontUnit>,
    ),
    CubicCurve(
        Vector2D<f32, FontUnit>,
        Vector2D<f32, FontUnit>,
        Vector2D<f32, FontUnit>,
        Vector2D<f32, FontUnit>,
    ),
}
