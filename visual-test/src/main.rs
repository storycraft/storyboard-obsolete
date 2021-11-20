/*
 * Created on Mon Nov 15 2021
 *
 * Copyright (c) storycraft. Licensed under the MIT Licence.
 */

use futures::executor::block_on;
use storyboard::{
    graphics::{
        backend::BackendOptions,
        default::primitive::{PrimitiveStyle, RectDrawState},
        renderer::StoryboardRenderer,
        PixelUnit,
    },
    math::{Point2D, Rect, Size2D},
    ringbuffer::RingBufferRead,
    state::StateStatus,
    thread::render::{RenderOperation, RenderQueue},
    graphics::wgpu::{Color, LoadOp, Operations, PresentMode},
    window::{
        dpi::PhysicalSize,
        event::{DeviceEvent, Event},
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

    storyboard.run(VisualTestMainState {
        position: Point2D::zero(),
    });
}

struct VisualTestMainState {
    position: Point2D<f32, PixelUnit>,
}

impl StoryboardState for VisualTestMainState {
    fn update(
        &mut self,
        _: &StoryboardSystemProp,
        system_state: &mut StoryboardSystemState,
    ) -> StateStatus<StoryboardSystemProp, StoryboardSystemState> {
        for event in system_state.events.drain() {
            if let Event::DeviceEvent {
                device_id: _,
                event,
            } = event
            {
                if let DeviceEvent::MouseMotion { delta } = event {
                    self.position += Size2D::new(delta.0 as f32, delta.1 as f32);
                }
            }
        }

        let mut render_queue = RenderQueue::new();

        let mut renderer = StoryboardRenderer::new();

        renderer.append(RectDrawState {
            style: PrimitiveStyle::default(),
            draw_box: system_state
                .screen
                .inner_box(Rect::new(self.position, Size2D::new(300.0, 300.0)), None),
        });

        render_queue.set_surface_task(RenderOperation {
            operations: Operations {
                load: LoadOp::Clear(Color::BLACK),
                store: true,
            },
            renderer,
        });

        system_state.submit_render_queue(render_queue);

        // println!("Update: {}, FPS: {}", 1000000.0 / system_state.elapsed.as_micros() as f64, system_state.render_thread().fps());

        StateStatus::Poll
    }

    fn load(&mut self, prop: &StoryboardSystemProp) {
        println!("Loaded!");
        // prop.window.set_cursor_grab(true).unwrap();
        prop.window.set_cursor_visible(false);
    }

    fn unload(&mut self, prop: &StoryboardSystemProp) {
        println!("Unloaded!");
        prop.window.set_cursor_grab(false).unwrap();
        prop.window.set_cursor_visible(true);
    }
}
