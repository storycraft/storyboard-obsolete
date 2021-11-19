/*
 * Created on Fri Nov 05 2021
 *
 * Copyright (c) storycraft. Licensed under the MIT Licence.
 */

use storyboard_graphics::{
    math::Size2D, surface::StoryboardSurface, unit::PixelUnit, wgpu::Instance,
};
use winit::{dpi::PhysicalSize, window::Window};

#[derive(Debug)]
pub struct StoryboardWindow {
    window: Window,
    surface: StoryboardSurface,
}

impl StoryboardWindow {
    pub fn init(instance: &Instance, window: Window) -> Self {
        let mut surface = StoryboardSurface::init(instance, &window).unwrap();

        let inner_size = window.inner_size();
        surface.update_view(Size2D::new(inner_size.width, inner_size.height));

        Self { window, surface }
    }

    pub fn surface(&self) -> &StoryboardSurface {
        &self.surface
    }

    pub fn surface_mut(&mut self) -> &mut StoryboardSurface {
        &mut self.surface
    }

    pub fn inner_size(&self) -> Size2D<u32, PixelUnit> {
        let inner_size = self.window.inner_size();

        Size2D::new(inner_size.width, inner_size.height)
    }

    pub fn update_inner_size(&mut self, size: Size2D<u32, PixelUnit>) {
        self.window
            .set_inner_size(PhysicalSize::new(size.width, size.height));
        self.surface.update_view(size);
    }

    pub fn update_view(&mut self, size: Size2D<u32, PixelUnit>) {
        self.surface.update_view(size);
    }

    #[inline]
    pub fn request_redraw(&self) {
        self.window.request_redraw();
    }
}
