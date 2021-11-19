/*
 * Created on Fri Nov 05 2021
 *
 * Copyright (c) storycraft. Licensed under the MIT Licence.
 */

use winit::{error::OsError, event_loop::{ControlFlow, EventLoop}, window::{Window, WindowBuilder}};

use crate::scene::{SceneState, StoryboardScene};

pub struct StoryboardApp {
    window: Window,
    event_loop: EventLoop<()>
}

impl StoryboardApp {
    pub fn new(builder: WindowBuilder) -> Self {
        let event_loop = EventLoop::new();

        // TODO:: Error checking
        let window = builder.build(&event_loop).unwrap();

        Self {
            window,
            event_loop
        }
    }

    pub fn run(self, first_scene: Box<dyn StoryboardScene>) -> ! {
        let mut current_scene = first_scene;

        self.event_loop.run(move |event, _, control_flow| {
            match event {
                _ => {

                }
            }

            match current_scene.update() {
                SceneState::Keep => {
                    
                },

                SceneState::Exit => {
                    if current_scene.on_exit() {
                        *control_flow = ControlFlow::Exit;
                    }  
                },

                SceneState::Move(scene) => {
                    current_scene = scene;
                },
            }
            
            // current_scene.render(screen, compositor, renderer);
        })
    }
}
