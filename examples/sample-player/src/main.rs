#![feature(generic_associated_types)]

pub mod main_menu;
pub mod player;

use main_menu::MainMenu;
use storyboard::{
    app::{StoryboardApp, StoryboardAppProp, StoryboardAppState},
    render::{
        backend::BackendOptions,
        wgpu::{Limits, PowerPreference, PresentMode},
    },
    winit::{
        event_loop::{ControlFlow, EventLoop},
        window::{Window, WindowBuilder},
    },
    Storyboard,
};
use storyboard_state::{StateData, StateSystem, SystemStatus};

#[cfg(not(target_arch = "wasm32"))]
fn main() {
    use futures::executor::block_on;

    let event_loop = EventLoop::new();
    let window = WindowBuilder::new()
        .with_title("Storyboard example player")
        .build(&event_loop)
        .unwrap();

    block_on(main_async(event_loop, window));
}

#[cfg(target_arch = "wasm32")]
fn main() {
    std::panic::set_hook(Box::new(console_error_panic_hook::hook));
    console_log::init().expect("could not initialize logger");

    let event_loop = EventLoop::new();
    let window = WindowBuilder::new()
        .with_title("Storyboard example player")
        .build(&event_loop)
        .unwrap();

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
    )
    .await
    .unwrap();

    storyboard.run(event_loop, App::new());
}

#[derive(Debug)]
pub struct StoryboardStateData;
impl StateData for StoryboardStateData {
    type Prop<'p> = StoryboardAppProp;
    type State<'s> = StoryboardAppState<'s>;
}

#[derive(Debug)]
pub struct App {
    system: Option<StateSystem<StoryboardStateData>>,
}

impl App {
    pub fn new() -> Self {
        Self {
            system: None
        }
    }
}

impl StoryboardApp for App {
    fn load(&mut self, prop: &StoryboardAppProp) {
        self.system = Some(StateSystem::new(Box::new(MainMenu::new()), prop));
    }

    fn unload(&mut self, _: &StoryboardAppProp) {
        self.system.take();
    }

    fn update(&mut self, prop: &StoryboardAppProp, state: &mut StoryboardAppState) {
        let system = self.system.as_mut().unwrap();

        let status = system.run(prop, state);

        *state.control_flow = if system.finished() {
            ControlFlow::Exit
        } else {
            match status {
                SystemStatus::Poll => ControlFlow::Poll,
                SystemStatus::Wait => ControlFlow::Wait,
            }
        };
    }
}
