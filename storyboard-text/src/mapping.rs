/*
 * Created on Sun Dec 05 2021
 *
 * Copyright (c) storycraft. Licensed under the MIT Licence.
 */

use std::{collections::HashMap, fmt::Debug};

use font_kit::canvas::{Canvas, Format};
use pathfinder_geometry::vector::Vector2I;
use rect_packer::DensePacker;
use rustc_hash::FxHashMap;
use storyboard::{
    graphics::PixelUnit,
    math::{Point2D, Rect, Size2D},
};

use crate::font::DrawFont;

#[derive(Debug)]
pub struct GlyphMappingData {
    canvas: Canvas,
    map: GlyphMapping,
}

impl GlyphMappingData {
    fn new(format: Format, size: Size2D<u32, PixelUnit>, map: GlyphMapping) -> Self {
        Self {
            canvas: Canvas::new(Vector2I::new(size.width as i32, size.height as i32), format),
            map,
        }
    }

    pub fn new_atlas(format: Format, size: Size2D<u32, PixelUnit>) -> Self {
        Self::new(
            format,
            size,
            GlyphMapping::Atlas {
                packer: DensePacker::new(size.width as i32, size.height as i32),
                map: HashMap::new(),
            },
        )
    }

    pub fn new_single(format: Format, size: Size2D<u32, PixelUnit>) -> Self {
        Self::new(
            format,
            size,
            GlyphMapping::Single {
                occupied: None,
                size,
            },
        )
    }

    pub fn canvas(&self) -> &Canvas {
        &self.canvas
    }

    pub fn get_size(&self) -> Size2D<u32, PixelUnit> {
        Size2D::new(self.canvas.size.x() as u32, self.canvas.size.y() as u32)
    }

    pub fn get(&self, font: &DrawFont, key: &GlyphKey) -> Option<Rect<u32, PixelUnit>> {
        let postscript_name = font.postscript_name();
        self.map.get(&postscript_name, &key)
    }

    pub fn cache_rasterized(
        &mut self,
        font: &DrawFont,
        key: GlyphKey,
    ) -> Option<Rect<u32, PixelUnit>> {
        if self.map.can_reserve() {
            let postscript_name = font.postscript_name();
            let bound_rect = font.raster_bounds(key.id, key.size as f32).ok()?;

            let rect = self.map.reserve(&postscript_name, key, bound_rect.size.cast())?;

            font.rasterize(
                &mut self.canvas,
                (rect.origin.cast::<i32>() - bound_rect.origin).to_point(),
                key.id,
                key.size as f32,
            )
            .ok()
            .unwrap();

            Some(rect)
        } else {
            None
        }
    }
}

enum GlyphMapping {
    Single {
        occupied: Option<(String, GlyphKey)>,
        size: Size2D<u32, PixelUnit>,
    },
    Atlas {
        packer: DensePacker,
        map: HashMap<String, FxHashMap<GlyphKey, Rect<u32, PixelUnit>>>,
    },
}

impl GlyphMapping {
    pub fn can_reserve(&self) -> bool {
        match self {
            Self::Single { occupied, size: _ } => occupied.is_none(),
            Self::Atlas { packer, map: _ } => packer.can_pack(1, 1, false),
        }
    }

    pub fn get(&self, postscript_name: &str, key: &GlyphKey) -> Option<Rect<u32, PixelUnit>> {
        match self {
            Self::Single { occupied, size } => {
                if let Some((font_name, stored_key)) = occupied.as_ref() {
                    if postscript_name.eq(font_name) && key.eq(stored_key) {
                        Some(Rect::new(Point2D::new(0, 0), *size))
                    } else {
                        None
                    }
                } else {
                    None
                }
            }

            Self::Atlas { packer: _, map } => {
                let font_map = map.get(postscript_name)?;

                font_map.get(key).copied()
            }
        }
    }

    pub fn reserve(
        &mut self,
        postscript_name: &str,
        info: GlyphKey,
        glyph_bounds: Size2D<u32, PixelUnit>,
    ) -> Option<Rect<u32, PixelUnit>> {
        match self {
            Self::Single { occupied, size } => {
                if occupied.is_none()
                    && glyph_bounds.width <= size.width
                    && glyph_bounds.height <= size.height
                {
                    *occupied = Some((postscript_name.to_string(), info));
                    Some(Rect::new(Point2D::new(0, 0), glyph_bounds))
                } else {
                    None
                }
            }

            Self::Atlas { packer, map } => {
                let font_map = {
                    if let Some(font_map) = map.get_mut(postscript_name) {
                        font_map
                    } else {
                        map.insert(postscript_name.to_string(), FxHashMap::default());

                        map.get_mut(postscript_name).unwrap()
                    }
                };

                let rect =
                    packer.pack(glyph_bounds.width as i32, glyph_bounds.height as i32, false)?;

                let rect = Rect::new(
                    Point2D::new(rect.x as u32, rect.y as u32),
                    Size2D::new(rect.width as u32, rect.height as u32),
                );

                font_map.insert(info, rect);

                Some(rect)
            }
        }
    }
}

impl Debug for GlyphMapping {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Single { occupied, size } => f
                .debug_struct("Single")
                .field("occupied", occupied)
                .field("size", size)
                .finish(),
            Self::Atlas { packer: _, map } => f.debug_struct("Atlas").field("map", map).finish(),
        }
    }
}

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
pub struct GlyphKey {
    pub id: u16,
    pub size: u32,
}
