/*
 * Created on Wed May 04 2022
 *
 * Copyright (c) storycraft. Licensed under the MIT Licence.
 */

use std::{error::Error, sync::Arc};

use storyboard::{
    graphics::{
        backend::BackendOptions,
        component::primitive::{Rectangle},
        texture::RenderTexture2D,
    },
    state::{
        StoryboardStateData, StoryboardStateStatus, StoryboardSystemProp, StoryboardSystemState,
    },
    winit::{
        event::{Event, WindowEvent},
        window::WindowBuilder,
    },
    Storyboard,
};
use storyboard_core::{
    component::color::ShapeColor,
    euclid::{Point2D, Rect, Size2D},
    state::State,
    wgpu::{PowerPreference, TextureFormat, TextureUsages}, unit::PixelUnit,
};

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
}

#[derive(Debug)]
pub struct SampleApp {
    texture: Option<Arc<RenderTexture2D>>,
    cursor: Point2D<f32, PixelUnit>
}

impl SampleApp {
    pub fn new() -> Self {
        Self { texture: None, cursor: Default::default() }
    }
}

impl State<StoryboardStateData> for SampleApp {
    fn load(&mut self, system_prop: &StoryboardSystemProp) {
        let texture = system_prop.create_texture_with_data(
            Some("App texture"),
            Size2D::new(2, 2),
            TextureFormat::Bgra8Unorm,
            TextureUsages::COPY_DST | TextureUsages::TEXTURE_BINDING,
            &[
                0xff, 0x00, 0x00, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0x00,
                0x00, 0xff,
            ]
        );

        self.texture = Some(Arc::new(
            system_prop.create_render_texture(
                texture
                    .create_view_default(None)
                    .slice(Rect::new(Point2D::new(0, 1), Size2D::new(2, 1)))
                    .into(),
                None,
            ),
        ));

        println!("App loaded");
    }

    fn unload(&mut self, system_prop: &StoryboardSystemProp) {
        self.texture.take();

        println!("App unloaded");
    }

    fn update(
        &mut self,
        system_prop: &StoryboardSystemProp,
        system_state: &mut StoryboardSystemState,
    ) -> StoryboardStateStatus {
        if let Event::RedrawRequested(_) = system_state.event {
            for _ in 0..200 {
                system_prop.draw(Rectangle {
                    bounds: Rect::new(self.cursor, Size2D::new(100.0, 100.0)),
                    color: ShapeColor::white(),
                    texture: self.texture.clone(),
                    texture_rect: Rect::new(Point2D::new(0.0, 0.0), Size2D::new(100.0, 100.0)),
                });
            }
        } else if let Event::WindowEvent {
            event: WindowEvent::CloseRequested,
            ..
        } = system_state.event
        {
            return StoryboardStateStatus::PopState;
        } else if let Event::WindowEvent { event: WindowEvent::CursorMoved { position, .. }, .. } = system_state.event {
            self.cursor = Point2D::new(position.x as f32, position.y as f32);
            system_prop.request_redraw();
        }

        StoryboardStateStatus::Wait
    }
}
