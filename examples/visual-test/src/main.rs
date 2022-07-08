use std::{borrow::Cow, sync::Arc};

use storyboard::{
    app::{StoryboardApp, StoryboardAppProp, StoryboardAppState},
    core::{
        color::ShapeColor,
        euclid::{Point2D, Rect, Size2D, Transform3D, Vector2D},
        unit::LogicalPixelUnit,
    },
    render::{
        backend::BackendOptions,
        wgpu::{Limits, PowerPreference, PresentMode, TextureFormat, TextureUsages},
    },
    texture::{ComponentTexture, TextureLayout, TextureLayoutStyle, TextureWrap},
    winit::{
        event::{Event, WindowEvent},
        event_loop::{ControlFlow, EventLoop},
        window::{Window, WindowBuilder},
    },
    Storyboard,
};
use storyboard_box2d::{Box2D, Box2DStyle};
use storyboard_primitive::Triangle;
use storyboard_text::{cache::GlyphCache, font::Font, Text};

#[cfg(not(target_arch = "wasm32"))]
fn main() {
    use futures::executor::block_on;

    let event_loop = EventLoop::new();
    let window = WindowBuilder::new()
        .with_title("Storyboard visual test")
        .build(&event_loop)
        .unwrap();

    block_on(main_async(event_loop, window));
}

#[cfg(target_arch = "wasm32")]
fn main() {
    let event_loop = EventLoop::new();
    let window = WindowBuilder::new()
        .with_title("Storyboard visual test")
        .build(&event_loop)
        .unwrap();

    std::panic::set_hook(Box::new(console_error_panic_hook::hook));
    console_log::init().expect("could not initialize logger");

    use storyboard::winit::platform::web::WindowExtWebSys;
    // On wasm, append the canvas to the document body
    web_sys::window()
        .and_then(|win| win.document())
        .and_then(|doc| doc.body())
        .and_then(|body| {
            body.append_child(&web_sys::Element::from(window.canvas()))
                .ok()
        })
        .expect("couldn't append canvas to document body");
    wasm_bindgen_futures::spawn_local(main_async(event_loop, window));
}

async fn main_async(event_loop: EventLoop<()>, window: Window) {
    let storyboard = Storyboard::init(
        window,
        &BackendOptions {
            power_preference: PowerPreference::HighPerformance,
            limits: Limits::downlevel_webgl2_defaults(),
            ..Default::default()
        },
        PresentMode::AutoNoVsync,
        Some(std::path::Path::new("traces")),
    )
    .await
    .unwrap();

    storyboard.run(
        event_loop,
        SampleApp::new(Font::new(Cow::Borrowed(FONT), 0).unwrap()),
    );
}

pub static FONT: &[u8] = include_bytes!("./NotoSansCJKkr-Regular.otf");

#[derive(Debug)]
pub struct SampleApp {
    texture: Option<ComponentTexture>,
    cursor: Point2D<f32, LogicalPixelUnit>,
    cache: GlyphCache,
    text: Text,
}

impl SampleApp {
    pub fn new(font: Font) -> Self {
        Self {
            texture: None,
            cursor: Default::default(),
            cache: GlyphCache::new(),
            text: Text::new(
                Point2D::new(100.0, 100.0),
                32,
                Transform3D::identity(),
                font,
                Cow::Borrowed(""),
            ),
        }
    }
}

impl StoryboardApp for SampleApp {
    fn load(&mut self, system_prop: &StoryboardAppProp) {
        let texture = system_prop.create_texture_with_data(
            Some("App texture"),
            Size2D::new(2, 2),
            TextureFormat::Bgra8Unorm,
            TextureUsages::COPY_DST | TextureUsages::TEXTURE_BINDING,
            &[
                0xff, 0x00, 0x00, 0xff, 0x00, 0xff, 0x00, 0xff, 0x00, 0xff, 0x00, 0xff, 0xff, 0x00,
                0x00, 0xff,
            ],
        );

        self.texture = Some(ComponentTexture::new(
            Arc::new(
                system_prop.create_render_texture(
                    texture
                        .create_view_default(None)
                        .slice(Rect::new(Point2D::new(1, 0), Size2D::new(1, 2))),
                    None,
                ),
            ),
            TextureLayout::Absolute(TextureLayoutStyle::Fit),
            (TextureWrap::None, TextureWrap::None),
        ));

        println!("App loaded");
    }

    fn unload(&mut self, _: &StoryboardAppProp) {
        println!("App unloaded");
    }

    fn update(&mut self, prop: &StoryboardAppProp, state: &mut StoryboardAppState) {
        if let Event::RedrawRequested(_) = state.event {
            state.draw(Triangle {
                bounds: Rect::new(Point2D::new(30.0, 30.0), Size2D::new(50.0, 50.0)),
                color: ShapeColor::WHITE,
                texture: self
                    .texture
                    .as_ref()
                    .map(|component| component.inner.clone()),
                texture_coord: [
                    Point2D::new(0.5, 0.0),
                    Point2D::new(0.0, 1.0),
                    Point2D::new(1.0, 1.0),
                ],
                transform: Transform3D::identity(),
            });

            state.draw(Box2D {
                bounds: Rect::new(self.cursor, Size2D::new(50.0, 50.0)),
                fill_color: ShapeColor::WHITE,
                border_color: ShapeColor::RED,
                texture: self.texture.clone(),
                style: Box2DStyle {
                    border_thickness: 5.0,
                    shadow_offset: Vector2D::new(100.0, 100.0),
                    shadow_radius: 2.0,
                    shadow_color: ShapeColor::BLUE.into(),
                    ..Default::default()
                },
                transform: Transform3D::identity(),
            });

            self.text.set_text(Cow::Owned(format!(
                "렌더링 테스트\n{:?}\nElapsed: {} ms\nFps: {}",
                self.cursor * prop.window.scale_factor() as f32,
                prop.elapsed.as_nanos() as f64 / 1_000_000.0,
                state.render_task.frame_rate()
            )));

            self.text.update(
                prop.backend.device(),
                prop.backend.queue(),
                prop.window.scale_factor() as _,
                &prop.texture_data,
                &mut self.cache,
            );

            state.draw(self.text.draw(&ShapeColor::WHITE));

            state.draw(Box2D {
                bounds: self.text.bounding_box().to_rect(),
                fill_color: ShapeColor::TRANSPARENT,
                border_color: ShapeColor::WHITE,
                texture: None,
                style: Box2DStyle {
                    border_thickness: 1.0,
                    ..Default::default()
                },
                transform: Transform3D::identity(),
            });

            state.render();
        } else if let Event::WindowEvent {
            event: WindowEvent::CloseRequested,
            ..
        } = state.event
        {
            *state.control_flow = ControlFlow::Exit;
            return;
        } else if let Event::WindowEvent {
            event: WindowEvent::CursorMoved { position, .. },
            ..
        } = state.event
        {
            self.cursor = Point2D::new(position.x as f32, position.y as f32)
                / prop.window.scale_factor() as f32;

            self.text.position = self.cursor;
        }
        prop.request_redraw();

        *state.control_flow = ControlFlow::Poll;
    }
}
