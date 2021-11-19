/*
 * Created on Fri Nov 05 2021
 *
 * Copyright (c) storycraft. Licensed under the MIT Licence.
 */

use storyboard_graphics::{component::DrawSpace, renderer::StoryboardRenderer};

use crate::compositor::StoryboardCompositor;

pub trait StoryboardScene {
    fn on_load(&mut self);
    fn on_unload(&mut self);

    fn on_exit(&mut self) -> bool;

    fn update(&mut self) -> SceneState;

    fn render<'a>(
        &mut self,
        screen: DrawSpace,
        compositor: &'a StoryboardCompositor,
        renderer: &mut StoryboardRenderer<'a>,
    );
}

pub enum SceneState {
    Keep,
    Exit,
    Move(Box<dyn StoryboardScene>)
}
