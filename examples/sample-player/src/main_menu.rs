use std::{borrow::Cow, fs::File, io::BufReader};

use rodio::{Decoder, OutputStream, OutputStreamHandle, Sink};
use storyboard::{
    core::{
        component::color::ShapeColor,
        euclid::Point2D,
        state::{State, StateStatus},
    },
    state::{StoryboardStateData, StoryboardSystemProp, StoryboardSystemState},
    winit::event::{Event, WindowEvent},
};
use storyboard_text::{cache::GlyphCache, component::text::Text, font::Font};

use crate::player::Player;

pub static FONT: &'static [u8] = include_bytes!("./NotoSansCJKkr-Regular.otf");

pub struct AppMain {
    text: Option<Text>,
    cache: GlyphCache,
    _stream: OutputStream,
    handle: OutputStreamHandle,
}

impl AppMain {
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

impl State<StoryboardStateData> for AppMain {
    fn load(&mut self, system_prop: &StoryboardSystemProp) {
        self.text = Some(Text::new(
            Point2D::new(10.0, 10.0),
            32,
            Font::new(Cow::Borrowed(FONT), 0).unwrap(),
            Cow::Borrowed("Drag drop audio file to play"),
        ));
        system_prop.request_redraw();
    }

    fn unload(&mut self, _: &StoryboardSystemProp) {
        self.text.take();
    }

    fn update(
        &mut self,
        system_prop: &StoryboardSystemProp,
        system_state: &mut StoryboardSystemState,
    ) -> StateStatus<StoryboardStateData> {
        match &system_state.event {
            Event::RedrawRequested(_) => {
                system_state.draw(self.text.as_mut().unwrap().draw(
                    system_prop.backend.device(),
                    system_prop.backend.queue(),
                    &ShapeColor::WHITE,
                    system_prop.window.scale_factor() as _,
                    &system_prop.texture_data,
                    &mut self.cache,
                ));
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
