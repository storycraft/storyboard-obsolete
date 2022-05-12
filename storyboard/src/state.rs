/*
 * Created on Mon May 02 2022
 *
 * Copyright (c) storycraft. Licensed under the MIT Licence.
 */

//! Prop and States implemention for storyboard app.

use std::{marker::PhantomData, sync::Arc, time::Duration};

use storyboard_core::{
    euclid::Size2D,
    graphics::texture::SizedTexture2D,
    state::{StateData, StateStatus},
    unit::PixelUnit,
    wgpu::{TextureFormat, TextureUsages},
};
use winit::{event::Event, window::Window};

use crate::{
    graphics::{backend::StoryboardBackend, compositor::ComponentCompositor, texture::TextureData},
    DefaultCompositor,
};

/// System properties for [StoryboardState].
///
/// Contains [winit::window::Window], [GraphicsData] of app
#[derive(Debug)]
pub struct StoryboardSystemProp {
    pub backend: Arc<StoryboardBackend>,
    pub texture_data: Arc<TextureData>,
    pub window: Window,
    pub elapsed: Duration,
}

impl StoryboardSystemProp {
    /// Create [SizedTexture2D] from descriptor
    pub fn init_texture(
        &self,
        label: Option<&str>,
        size: Size2D<u32, PixelUnit>,
        format: TextureFormat,
        usage: TextureUsages,
    ) -> SizedTexture2D {
        SizedTexture2D::init(self.backend.device(), label, size, format, usage)
    }
}

/// Mutable system state for [StoryboardState].
///
/// Contains event.
#[derive(Debug)]
pub struct StoryboardSystemState<'a, Compositor: ComponentCompositor = DefaultCompositor> {
    pub event: Event<'a, ()>,

    pub(crate) components: Vec<Compositor::Component>,
}

impl<'a, Compositor: ComponentCompositor> StoryboardSystemState<'a, Compositor> {
    pub fn render_component(&mut self, component: impl Into<Compositor::Component>) {
        self.components.push(component.into());
    }
}

pub struct StoryboardStateData<T = DefaultCompositor> {
    phantom: PhantomData<T>,
}

impl<Compositor: ComponentCompositor> StateData for StoryboardStateData<Compositor> {
    type Prop<'p> = StoryboardSystemProp;
    type State<'s> = StoryboardSystemState<'s, Compositor>;
}

pub type StoryboardStateStatus<Compositor = DefaultCompositor> =
    StateStatus<StoryboardStateData<Compositor>>;
