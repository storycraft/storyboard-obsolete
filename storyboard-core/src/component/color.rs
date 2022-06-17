use std::{marker::PhantomData, ops::Index};

use palette::rgb::{LinSrgba, Rgb};

pub type Color = LinSrgba;

#[derive(Debug, Clone)]
pub enum ShapeColor<const VERTICES: usize = 1> {
    Single(Color),
    Gradient([Color; VERTICES]),
}

impl<const VERTICES: usize> ShapeColor<VERTICES> {
    pub const WHITE: ShapeColor<VERTICES> = ShapeColor::Single(Color {
        color: Rgb {
            red: 1.0,
            green: 1.0,
            blue: 1.0,
            standard: PhantomData,
        },
        alpha: 1.0,
    });

    pub const RED: ShapeColor<VERTICES> = ShapeColor::Single(Color {
        color: Rgb {
            red: 1.0,
            green: 0.0,
            blue: 0.0,
            standard: PhantomData,
        },
        alpha: 1.0,
    });
    pub const GREEN: ShapeColor<VERTICES> = ShapeColor::Single(Color {
        color: Rgb {
            red: 0.0,
            green: 1.0,
            blue: 0.0,
            standard: PhantomData,
        },
        alpha: 1.0,
    });
    pub const BLUE: ShapeColor<VERTICES> = ShapeColor::Single(Color {
        color: Rgb {
            red: 0.0,
            green: 0.0,
            blue: 1.0,
            standard: PhantomData,
        },
        alpha: 1.0,
    });

    pub const BLACK: ShapeColor<VERTICES> = ShapeColor::Single(Color {
        color: Rgb {
            red: 0.0,
            green: 0.0,
            blue: 0.0,
            standard: PhantomData,
        },
        alpha: 1.0,
    });

    pub const TRANSPARENT: ShapeColor<VERTICES> = ShapeColor::Single(Color {
        color: Rgb {
            red: 0.0,
            green: 0.0,
            blue: 0.0,
            standard: PhantomData,
        },
        alpha: 0.0,
    });

    pub fn opaque(&self) -> bool {
        match self {
            ShapeColor::Single(color) => color.alpha >= 1.0,
            ShapeColor::Gradient(colors) => colors.iter().any(|color| color.alpha >= 1.0),
        }
    }
}

impl<const VERTICES: usize> Default for ShapeColor<VERTICES> {
    fn default() -> Self {
        Self::WHITE
    }
}

impl<const VERTICES: usize> From<Color> for ShapeColor<VERTICES> {
    fn from(color: Color) -> Self {
        Self::Single(color)
    }
}

impl<const VERTICES: usize> From<[Color; VERTICES]> for ShapeColor<VERTICES> {
    fn from(gradient: [Color; VERTICES]) -> Self {
        Self::Gradient(gradient)
    }
}

impl Into<Color> for ShapeColor<1> {
    fn into(self) -> Color {
        self[0]
    }
}

impl<const VERTICES: usize> Index<usize> for ShapeColor<VERTICES> {
    type Output = Color;

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
