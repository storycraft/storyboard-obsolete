/*
 * Created on Wed May 04 2022
 *
 * Copyright (c) storycraft. Licensed under the MIT Licence.
 */

use std::error::Error;

use storyboard::{
    graphics::{
        backend::BackendOptions,
        compositor::primitive::{Primitive, Quad},
    },
    state::{
        StoryboardStateData, StoryboardStateStatus, StoryboardSystemProp, StoryboardSystemState,
    },
    winit::{event::Event, window::WindowBuilder},
    Storyboard,
};
use storyboard_core::{component::color::ShapeColor, state::State, wgpu::PowerPreference};

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let builder = WindowBuilder::new().with_title("Storyboard visual test");

    let storyboard = Storyboard::init(
        builder,
        &BackendOptions {
            power_preference: PowerPreference::HighPerformance,
            ..Default::default()
        },
    )
    .await?;

    storyboard.run(SampleApp::new());

    Ok(())
}

#[derive(Debug)]
pub struct SampleApp {}

impl SampleApp {
    pub fn new() -> Self {
        Self {}
    }
}

impl State<StoryboardStateData> for SampleApp {
    fn load(&mut self, system_prop: &StoryboardSystemProp) {
        println!("App loaded");
    }

    fn unload(&mut self, system_prop: &StoryboardSystemProp) {}

    fn update<'s>(
        &mut self,
        system_prop: &StoryboardSystemProp,
        system_state: &mut StoryboardSystemState<'s>,
    ) -> StoryboardStateStatus {
        if let Event::MainEventsCleared = system_state.event {
            system_state.render_component(Primitive::Quad(Quad {
                points: [
                    (-0.5, -0.5).into(),
                    (-0.5, 0.5).into(),
                    (0.5, 0.5).into(),
                    (0.5, -0.5).into(),
                ],
                color: ShapeColor::white(),
                texture: None,
                texture_coords: [
                    (0.0, 0.0).into(),
                    (0.0, 1.0).into(),
                    (1.0, 1.0).into(),
                    (1.0, 0.0).into(),
                ],
            }));
        }

        println!(
            "Elapsed: {} ms, means {} fps",
            system_prop.elapsed.as_millis(),
            1000_000_f32 / system_prop.elapsed.as_micros() as f32
        );

        StoryboardStateStatus::Poll
    }
}
