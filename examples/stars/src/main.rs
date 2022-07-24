#![feature(generic_associated_types)]

use std::time::{Duration, Instant};

use rand::Rng;
use storyboard::{
    app::{StoryboardApp, StoryboardAppProp, StoryboardAppState},
    core::{
        color::{Color, ShapeColor},
        euclid::{rect, Angle, Rect, Transform3D},
        unit::LogicalPixelUnit,
    },
    render::{
        backend::BackendOptions,
        task::RenderTask,
        wgpu::{Limits, PowerPreference, PresentMode},
    },
    winit::{
        event::Event,
        event_loop::{EventLoop, ControlFlow},
        window::{Window, WindowBuilder},
    },
    Storyboard,
};
use storyboard_box2d::{Box2D, Box2DStyle};
use storyboard_frame::{FrameComponent, FrameContainer};
use storyboard_state::{State, StateData, StateStatus, StateSystem, SystemStatus};

fn main() {
    use futures::executor::block_on;

    let event_loop = EventLoop::new();
    let window = WindowBuilder::new()
        .with_title("Storyboard example player")
        .build(&event_loop)
        .unwrap();

    block_on(main_async(event_loop, window));
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
        None,
    )
    .await
    .unwrap();

    storyboard.run(
        event_loop,
        App::new(SimpleAnimApp::new(Duration::from_millis(20))),
    );
}

#[derive(Debug)]
pub struct App {
    inital_state: Option<Box<dyn State<AppStateData>>>,
    state: Option<StateSystem<AppStateData>>,
    container: FrameContainer,
}

impl App {
    pub fn new(inital_state: impl State<AppStateData> + 'static) -> Self {
        Self {
            inital_state: Some(Box::new(inital_state)),
            state: None,
            container: FrameContainer::new(),
        }
    }
}

impl StoryboardApp for App {
    fn load(&mut self, prop: &StoryboardAppProp) {
        if let Some(inital_state) = self.inital_state.take() {
            self.state = Some(StateSystem::new(inital_state, prop));
        }
    }

    fn unload(&mut self, _: &StoryboardAppProp) {}

    fn update(&mut self, app_prop: &StoryboardAppProp, app_state: &mut StoryboardAppState) {
        match app_state.event {
            Event::RedrawRequested(_) => {
                self.container.update();
                for components in self.container.values() {
                    components.draw(app_state.render_task);
                }
                app_state.render();
            }

            Event::MainEventsCleared => {
                if let Some(state) = &mut self.state {
                    *app_state.control_flow = match state.run(app_prop, &mut self.container) {
                        SystemStatus::Poll => ControlFlow::Poll,
                        SystemStatus::Wait => ControlFlow::Wait,
                    }
                }

                if self.container.invalidated() {
                    app_prop.window.request_redraw();
                }
            }

            _ => {}
        }
    }
}

#[derive(Debug)]
pub struct AppStateData;
impl StateData for AppStateData {
    type Prop<'p> = StoryboardAppProp;
    type State<'s> = FrameContainer;
}

#[derive(Debug)]
pub struct SimpleAnimApp {
    elapsed: Duration,
    interval: Duration,
}

impl SimpleAnimApp {
    pub fn new(interval: Duration) -> Self {
        Self {
            elapsed: interval,
            interval,
        }
    }
}

impl State<AppStateData> for SimpleAnimApp {
    fn load<'p>(&mut self, _: &<AppStateData as StateData>::Prop<'p>) {}

    fn unload<'p>(&mut self, _: &<AppStateData as StateData>::Prop<'p>) {}

    fn update<'p, 's>(
        &mut self,
        system_prop: &<AppStateData as StateData>::Prop<'p>,
        system_state: &mut <AppStateData as StateData>::State<'s>,
    ) -> StateStatus<AppStateData> {
        self.elapsed += system_prop.elapsed;

        if self.elapsed >= self.interval {
            self.elapsed = Duration::ZERO;

            let mut rng = rand::thread_rng();

            system_state.add_component(FadingStar::new(
                Instant::now(),
                Duration::from_millis(300 + (rng.gen::<f32>() * 2000.0) as u64),
                rect(
                    rng.gen::<f32>() * system_prop.window.inner_size().width as f32,
                    rng.gen::<f32>() * system_prop.window.inner_size().height as f32,
                    48.0,
                    48.0,
                ),
                (1.0, 1.0, 1.0, 1.0).into(),
            ));
        }

        StateStatus::Poll
    }
}

#[derive(Debug)]
pub struct FadingStar {
    start: Instant,
    duration: Duration,

    bounds: Rect<f32, LogicalPixelUnit>,
    color: Color,

    alpha: f32,
}

impl FadingStar {
    pub fn new(
        start: Instant,
        duration: Duration,
        bounds: Rect<f32, LogicalPixelUnit>,
        color: Color,
    ) -> Self {
        Self {
            start,
            duration,

            bounds,
            color,

            alpha: 1.0,
        }
    }
}

impl FrameComponent for FadingStar {
    fn expired(&self) -> bool {
        self.start.elapsed() > self.duration
    }

    fn update(&mut self) -> bool {
        self.alpha =
            1.0 - self.start.elapsed().as_millis() as f32 / self.duration.as_millis() as f32;
        true
    }

    fn draw(&self, task: &mut RenderTask) {
        task.push(Box2D {
            bounds: self.bounds,
            texture: None,
            fill_color: ShapeColor::Single(
                (
                    self.color.red,
                    self.color.green,
                    self.color.blue,
                    self.color.alpha * self.alpha,
                )
                    .into(),
            ),
            border_color: ShapeColor::TRANSPARENT,
            style: Box2DStyle::default(),
            transform: Transform3D::translation(
                -self.bounds.origin.x - self.bounds.size.width / 2.0,
                -self.bounds.origin.y - self.bounds.size.height / 2.0,
                0.0,
            )
            .then_rotate(0.0, 0.0, 1.0, Angle::degrees(self.alpha * 360.0))
            .then_translate(
                (
                    self.bounds.origin.x + self.bounds.size.width / 2.0,
                    self.bounds.origin.y + self.bounds.size.height / 2.0,
                    0.0,
                )
                    .into(),
            ),
        })
    }
}
