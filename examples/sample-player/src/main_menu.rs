use std::{borrow::Cow, fs::File, io::BufReader};

use rodio::{Decoder, OutputStream, OutputStreamHandle, Sink};
use storyboard::{
    app::{StoryboardAppProp, StoryboardAppState},
    core::{
        color::ShapeColor,
        euclid::{Point2D, Transform3D},
    },
    winit::event::{Event, WindowEvent},
};
use storyboard_state::{State, StateStatus};
use storyboard_text::{cache::GlyphCache, font::Font, Text};

use crate::{player::Player, StoryboardStateData};

pub static FONT: &[u8] = include_bytes!("./NotoSansCJKkr-Regular.otf");

pub struct MainMenu {
    text: Option<Text>,
    cache: GlyphCache,
    _stream: OutputStream,
    handle: OutputStreamHandle,
}

impl MainMenu {
    pub fn new() -> Self {
        let (stream, handle) = OutputStream::try_default().unwrap();

        Self {
            text: None,
            cache: GlyphCache::new(),
            _stream: stream,
            handle,
        }
    }
}

impl Default for MainMenu {
    fn default() -> Self {
        Self::new()
    }
}

impl State<StoryboardStateData> for MainMenu {
    fn load(&mut self, system_prop: &StoryboardAppProp) {
        self.text = Some(Text::new(
            Point2D::new(10.0, 10.0),
            32,
            Transform3D::identity(),
            Font::new(Cow::Borrowed(FONT), 0).unwrap(),
            Cow::Borrowed("Drag drop audio file to play"),
        ));
        system_prop.request_redraw();
    }

    fn unload(&mut self, _: &StoryboardAppProp) {
        self.text.take();
    }

    fn update(
        &mut self,
        system_prop: &StoryboardAppProp,
        system_state: &mut StoryboardAppState,
    ) -> StateStatus<StoryboardStateData> {
        match &system_state.event {
            Event::RedrawRequested(_) => {
                self.text.as_mut().unwrap().update(
                    system_prop.backend.device(),
                    system_prop.backend.queue(),
                    system_prop.window.scale_factor() as _,
                    &system_prop.texture_data,
                    &mut self.cache,
                );

                system_state.draw(self.text.as_mut().unwrap().draw(&ShapeColor::WHITE));

                system_state.render();
            }

            Event::WindowEvent {
                window_id: _,
                event: WindowEvent::DroppedFile(path),
            } => {
                if let Ok(file) = File::open(path) {
                    if let Ok(decoder) = Decoder::new(BufReader::new(file)) {
                        return StateStatus::PushState(Box::new(Player::new(
                            Sink::try_new(&self.handle).unwrap(),
                            decoder,
                        )));
                    } else {
                        eprintln!("File is not valid audio file");
                    }
                } else {
                    eprintln!("Cannot read file");
                }
            }

            Event::WindowEvent {
                window_id: _,
                event: WindowEvent::CloseRequested,
            } => {
                return StateStatus::PopState;
            }

            _ => {}
        };

        StateStatus::Wait
    }
}
