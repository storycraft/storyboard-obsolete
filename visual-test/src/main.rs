/*
 * Created on Mon Nov 15 2021
 *
 * Copyright (c) storycraft. Licensed under the MIT Licence.
 */

use std::sync::Arc;

use futures::executor::block_on;
use storyboard::{
    component::layout::texture::{ComponentTexture, TextureLayout},
    graphics::{
        backend::BackendOptions,
        default::primitive::{PrimitiveStyle, RectDrawState},
        renderer::StoryboardRenderer,
        PixelUnit,
    },
    graphics::{
        texture::Texture2D,
        wgpu::{Color, LoadOp, Operations, PresentMode, TextureFormat},
    },
    math::{Point2D, Rect, Size2D},
    ringbuffer::RingBufferRead,
    state::StateStatus,
    thread::render::RenderOperation,
    window::{
        dpi::PhysicalSize,
        event::{Event, WindowEvent},
        window::WindowBuilder,
    },
    Storyboard, StoryboardState, StoryboardSystemProp, StoryboardSystemState,
};

fn main() {
    // simple_logger::SimpleLogger::new().init().unwrap();

    let win_builder = WindowBuilder::new()
        .with_inner_size(PhysicalSize::new(800, 800))
        .with_title("Visual test");

    let mut storyboard = block_on(Storyboard::init(win_builder, &BackendOptions::default()));

    storyboard.render_present_mode = PresentMode::Immediate;

    storyboard.run(VisualTestMainState::new());
}

struct VisualTestMainState {
    cursor_position: Point2D<f32, PixelUnit>,
    cursor_image: Option<Arc<Texture2D>>,
}

impl VisualTestMainState {
    pub fn new() -> Self {
        Self {
            cursor_position: Point2D::zero(),
            cursor_image: None,
        }
    }
}

impl StoryboardState for VisualTestMainState {
    fn update(
        &mut self,
        _: &StoryboardSystemProp,
        system_state: &mut StoryboardSystemState,
    ) -> StateStatus<StoryboardSystemProp, StoryboardSystemState> {
        for event in system_state.events.drain() {
            if let Event::WindowEvent {
                window_id: _,
                event,
            } = event
            {
                if let WindowEvent::CursorMoved {
                    device_id: _,
                    position,
                    modifiers: _,
                } = event
                {
                    self.cursor_position = Point2D::new(position.x as f32, position.y as f32)
                } else if let WindowEvent::CloseRequested = event {
                    return StateStatus::PopState;
                }
            }
        }

        let mut renderer = StoryboardRenderer::new();

        renderer.append(RectDrawState {
            style: PrimitiveStyle {
                texture: self
                    .cursor_image
                    .as_ref()
                    .map(|cursor_image| ComponentTexture {
                        texture: cursor_image.clone(),
                        layout: TextureLayout::Stretch,
                    }),
                ..Default::default()
            },
            draw_box: system_state.screen.inner_box(
                Rect::new(self.cursor_position, Size2D::new(32.0, 32.0)),
                None,
            ),
        });

        system_state.submit_render(RenderOperation {
            operations: Operations {
                load: LoadOp::Clear(Color::BLACK),
                store: true,
            },
            renderer,
        });

        // println!("Update: {}, FPS: {}", 1000000.0 / system_state.elapsed.as_micros() as f64, system_state.render_thread().fps());

        StateStatus::Poll
    }

    fn load(&mut self, prop: &StoryboardSystemProp) {
        println!("Loaded!");
        prop.window.set_cursor_visible(false);
        self.cursor_image = Some(Arc::new(prop.graphics.texture_data.create_texture_data(
            TextureFormat::Rgba8Unorm,
            Size2D::new(2, 2),
            None,
            &[
                0xff, 0xff, 0xff, 0xff, 0xff, 0x00, 0x00, 0xff, 0xff, 0x00, 0x00, 0xff, 0xff, 0xff,
                0xff, 0xff,
            ],
        )));
    }

    fn unload(&mut self, prop: &StoryboardSystemProp) {
        println!("Unloaded!");
        prop.window.set_cursor_visible(true);
        self.cursor_image.take();
    }
}
