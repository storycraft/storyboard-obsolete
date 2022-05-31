/*
 * Created on Mon Sep 20 2021
 *
 * Copyright (c) storycraft. Licensed under the MIT Licence.
 */

use std::ops::Index;

use palette::{named, rgb::LinSrgba};

#[derive(Debug, Clone)]
pub enum ShapeColor<const VERTICES: usize> {
    Single(LinSrgba),
    Gradient([LinSrgba; VERTICES]),
}

impl<const VERTICES: usize> ShapeColor<VERTICES> {

    #[inline]
    pub fn white() -> ShapeColor<VERTICES> {
        ShapeColor::Single(named::WHITE.into_format().into_linear().into())
    }

    #[inline]
    pub fn black() -> ShapeColor<VERTICES> {
        ShapeColor::Single(named::BLACK.into_format().into_linear().into())
    }

    #[inline]
    pub fn transparent() -> ShapeColor<VERTICES> {
        ShapeColor::Single(LinSrgba::new(0.0, 0.0, 0.0, 0.0))
    }
}

impl<const VERTICES: usize> Default for ShapeColor<VERTICES> {
    fn default() -> Self {
        Self::white()
    }
}

impl<const VERTICES: usize> ShapeColor<VERTICES> {
    pub fn partial_transparent(&self) -> bool {
        match self {
            ShapeColor::Single(color) => color.alpha < 1.0,
            ShapeColor::Gradient(colors) => colors.iter().any(|color| color.alpha < 1.0),
        }
    }
}

impl<const VERTICES: usize> From<LinSrgba> for ShapeColor<VERTICES> {
    fn from(color: LinSrgba) -> Self {
        Self::Single(color)
    }
}

impl<const VERTICES: usize> From<[LinSrgba; VERTICES]> for ShapeColor<VERTICES> {
    fn from(gradient: [LinSrgba; VERTICES]) -> Self {
        Self::Gradient(gradient)
    }
}

impl<const VERTICES: usize> Index<usize> for ShapeColor<VERTICES> {
    type Output = LinSrgba;

    fn index(&self, index: usize) -> &Self::Output {
        match self {
            ShapeColor::Single(color) => {
                if index >= VERTICES {
                    panic!("Index out of size. size = {}", VERTICES);
                }

                color
            }

            ShapeColor::Gradient(gradient) => &gradient[index],
        }
    }
}
