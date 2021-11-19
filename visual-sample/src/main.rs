/*
 * Created on Fri Nov 05 2021
 *
 * Copyright (c) storycraft. Licensed under the MIT Licence.
 */

use futures::executor::block_on;
use storyboard::{
    app::StoryboardApp,
    compositor::StoryboardCompositor,
    graphics::{
        backend::BackendOptions, component::DrawSpace, math::Rect, renderer::StoryboardRenderer,
    },
    primitive::PrimitiveStyle,
    scene::{SceneState, StoryboardScene},
    winit::{
        dpi::PhysicalSize,
        event_loop::{ControlFlow, EventLoop},
        window::WindowBuilder,
    },
    Storyboard,
};

pub fn main() {
    let app = StoryboardApp::new(
        WindowBuilder::new()
            .with_inner_size(PhysicalSize::new(800, 800))
            .with_title("Storyboard visual sample"),
    );

    let mut storyboard = block_on(Storyboard::init(window, BackendOptions::default()));

    app.run(Box::new(TestScene::new("Sample".into())));
}

pub struct TestScene {
    title: String
}

impl TestScene {
    pub fn new(title: String) -> Self {
        Self {
            title
        }
    }
}

impl StoryboardScene for TestScene {
    fn on_load(&mut self) {
        
    }

    fn on_unload(&mut self) {
        
    }

    fn on_exit(&mut self) -> bool {
        true
    }

    fn update(&mut self) -> SceneState {
        SceneState::Keep
    }

    fn render<'a>(
        &mut self,
        screen: DrawSpace,
        compositor: &'a StoryboardCompositor,
        renderer: &mut StoryboardRenderer<'a>,
    ) {
        println!("Rendering");
        renderer.append(compositor.primitive().rect(
            &PrimitiveStyle::default(),
            &screen.inner_box(
                Rect {
                    origin: (150.0, 150.0).into(),
                    size: (150.0, 150.0).into(),
                },
                None,
            ),
        ));
    }
}
