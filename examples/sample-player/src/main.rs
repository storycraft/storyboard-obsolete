pub mod main_menu;
pub mod player;

use main_menu::AppMain;
use storyboard::{
    render::{
        backend::BackendOptions,
        wgpu::{PowerPreference, PresentMode},
    },
    winit::{
        event_loop::EventLoop,
        window::{Window, WindowBuilder},
    },
    Storyboard,
};

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
            ..Default::default()
        },
        PresentMode::Mailbox,
    )
    .await
    .unwrap();

    storyboard.run(event_loop, AppMain::new());
}
