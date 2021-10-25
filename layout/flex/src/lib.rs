/*
 * Created on Thu Sep 23 2021
 *
 * Copyright (c) storycraft. Licensed under the MIT Licence.
 */

use storyboard::math::{Rect, Transform3D};
use stretch::{geometry::Size, node::Node, number::Number, style::Style, Stretch};

use storyboard::{
    component::transform::DrawTransform, data::observable::Observable, unit::PixelUnit,
};

use storyboard::component::{DrawBox, DrawSpace};

pub extern crate stretch;

#[derive(Debug, Clone)]
pub struct FlexLayoutNode {
    node: Node,

    style: Observable<Style>,
    transform: Observable<DrawTransform>,

    pub z_index: i32,

    draw_rect: Observable<Rect<f32, PixelUnit>>,
    draw_matrix: Transform3D<f32, PixelUnit, PixelUnit>,
}

impl FlexLayoutNode {
    pub fn new(stretch: &mut Stretch) -> Self {
        let style = Style::default();

        FlexLayoutNode {
            node: stretch.new_node(style, Vec::new()).unwrap(),

            style: Observable::new(style),
            transform: Observable::new(DrawTransform::default()),

            z_index: 0,

            draw_rect: Observable::new(Rect::default()),
            draw_matrix: Transform3D::identity(),
        }
    }

    pub fn transform(&self) -> &DrawTransform {
        self.transform.as_ref()
    }

    pub fn transform_mut(&mut self) -> &mut DrawTransform {
        self.transform.as_mut()
    }

    pub fn set_transform(&mut self, transform: DrawTransform) {
        self.transform.set(transform);
    }

    pub fn style(&self) -> &Style {
        self.style.as_ref()
    }

    pub fn style_mut(&mut self) -> &mut Style {
        self.style.as_mut()
    }

    pub fn set_style(&mut self, style: Style) {
        self.style.set(style);
    }

    fn update_transform(&mut self, space: &DrawSpace) {
        if !self.transform.valid() || !self.draw_rect.valid() {
            self.draw_matrix = self
                .transform
                .as_ref()
                .calc_matrix(space, self.draw_rect.as_ref());

            self.transform.unmark();
            self.draw_rect.unmark();
        }
    }

    pub fn get_draw_box(&self, space: &DrawSpace) -> DrawBox {
        space.inner_box(
            self.draw_rect.as_ref().clone(),
            Some(&self.draw_matrix),
        )
    }

    pub fn update_node(&mut self, stretch: &mut Stretch, space: &DrawSpace) {
        if self.style.unmark() {
            stretch
                .set_style(self.node, self.style.as_ref().clone())
                .unwrap();
        }

        let size = space.parent.size;

        stretch
            .compute_layout(
                self.node,
                Size {
                    width: Number::Defined(size.width),
                    height: Number::Defined(size.height),
                },
            )
            .unwrap();
        let layout = stretch.layout(self.node).unwrap();
        self.draw_rect.set(Rect {
            origin: (layout.location.x, layout.location.y).into(),
            size: (layout.size.width, layout.size.height).into(),
        });

        self.update_transform(space);
    }
}
