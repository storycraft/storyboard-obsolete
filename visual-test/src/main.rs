/*
 * Created on Mon Nov 15 2021
 *
 * Copyright (c) storycraft. Licensed under the MIT Licence.
 */

use std::sync::Arc;

use futures::executor::block_on;
use storyboard::{
    component::{
        color::ShapeColor,
        extent::ExtentUnit,
        layout::texture::{ComponentTexture, TextureLayout},
    },
    graphics::{backend::BackendOptions, renderer::StoryboardRenderer, PixelUnit},
    graphics::{
        default::box2d::{Box2DDrawState, BoxStyle},
        texture::Texture2D,
        wgpu::{Color, LoadOp, Operations, PresentMode},
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
use storyboard_text::{
    font::DrawFont, font_kit::source::SystemSource, mapping::GlyphKey, store::GlyphStore,
    TextDrawState,
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
    test_font: DrawFont,
    glyph_store: GlyphStore,

    cursor_position: Point2D<f32, PixelUnit>,
    cursor_image: Option<Arc<Texture2D>>,
}

impl VisualTestMainState {
    pub fn new() -> Self {
        Self {
            test_font: DrawFont::new(
                SystemSource::new()
                    .select_by_postscript_name("NotoSansCJKkr-Regular")
                    .unwrap()
                    .load()
                    .unwrap(),
            ),
            glyph_store: GlyphStore::new(),
            cursor_position: Point2D::zero(),
            cursor_image: None,
        }
    }
}

impl StoryboardState for VisualTestMainState {
    fn update(
        &mut self,
        system_prop: &StoryboardSystemProp,
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

        /*renderer.append(Box2DDrawState {
            style: BoxStyle {
                border_radius: ExtentUnit::Percent(0.0),
                border_thickness: 1.0,
                border_color: ShapeColor::white(),
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
                Rect::new(self.cursor_position, Size2D::new(256.0, 256.0)),
                None,
            ),
        });*/

        let glyph = self.glyph_store
        .get_glyph(
            &self.test_font,
            GlyphKey {
                id: self.test_font.char_to_glyph('가').unwrap(),
                size: 64,
            },
            &system_prop.graphics.texture_data
        )
        .unwrap();

        renderer.append(TextDrawState {
            color: ShapeColor::white(),
            glyphs: vec![
                (
                    system_state.screen.inner_box(
                        Rect::new(self.cursor_position, glyph.rect.size.cast()),
                        None,
                    ),
                    glyph
                );
                1
            ],
            textures: self.glyph_store.textures().clone(),
        });

        self.glyph_store.prepare(&system_prop.graphics.texture_data);

        system_state.submit_render(RenderOperation {
            operations: Operations {
                load: LoadOp::Clear(Color::BLACK),
                store: true,
            },
            renderer,
        });

        self.glyph_store.finish();

        // println!("Update: {}, FPS: {}", 1000000.0 / system_state.elapsed.as_micros() as f64, system_state.render_thread().fps());

        StateStatus::Poll
    }

    fn load(&mut self, prop: &StoryboardSystemProp) {
        prop.window.set_cursor_visible(false);

        for ch in "Hello world!".chars() {
            self.glyph_store.get_glyph(
                &self.test_font,
                GlyphKey {
                    id: self.test_font.char_to_glyph(ch).unwrap(),
                    size: 256,
                },
                &prop.graphics.texture_data,
            );
        }

        self.glyph_store.prepare(&prop.graphics.texture_data);

        self.cursor_image = Some(self.glyph_store.textures()[0].clone());

        self.glyph_store.finish();
    }

    fn unload(&mut self, prop: &StoryboardSystemProp) {
        prop.window.set_cursor_visible(true);
        self.cursor_image.take();

        println!("Unloaded!");
    }
}
