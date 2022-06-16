/*
 * Created on Thu Jun 16 2022
 *
 * Copyright (c) storycraft. Licensed under the MIT Licence.
 */

use std::{
    iter,
    num::NonZeroUsize,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
    thread::{self, JoinHandle},
};

use instant::Instant;
use parking_lot::Mutex;
use ring_channel::{ring_channel, RingSender};
use storyboard_core::{
    graphics::{
        component::Drawable,
        renderer::surface::{StoryboardSurfaceRenderer, SurfaceConfiguration},
    },
    trait_stack::TraitStack,
};

#[derive(Debug)]
pub struct RenderThread {
    handle: JoinHandle<StoryboardSurfaceRenderer>,

    tx: Option<RingSender<TraitStack<dyn Drawable + 'static>>>,

    configuration: Arc<(Mutex<SurfaceConfiguration>, AtomicBool)>,
}

impl RenderThread {
    pub fn run(surface_renderer: StoryboardSurfaceRenderer) -> Self {
        let configuration = Arc::new((
            Mutex::new(surface_renderer.configuration()),
            AtomicBool::new(false),
        ));

        let (tx, rx) = ring_channel(NonZeroUsize::new(2).unwrap());

        let handle = {
            let mut surface_renderer = surface_renderer;

            let mut rx = rx;

            let configuration = configuration.clone();

            thread::spawn(move || {
                while let Ok(drawables) = rx.recv() {
                    let instant = Instant::now();

                    if configuration.1.load(Ordering::Relaxed) {
                        configuration.1.store(false, Ordering::Relaxed);

                        surface_renderer.set_configuration(*configuration.0.lock());
                    }

                    if let Some(res) = surface_renderer.render(&drawables) {
                        surface_renderer
                            .backend()
                            .queue()
                            .submit(iter::once(res.command_buffer));
                        res.surface_texture.present();
                    }
                }

                surface_renderer
            })
        };

        Self {
            handle,
            tx: Some(tx),
            configuration,
        }
    }

    pub fn interrupted(&self) -> bool {
        self.tx.is_none()
    }

    pub fn configuration(&self) -> SurfaceConfiguration {
        *self.configuration.0.lock()
    }

    pub fn set_configuration(&self, configuration: SurfaceConfiguration) {
        *self.configuration.0.lock() = configuration;
        self.configuration.1.store(true, Ordering::Relaxed);
    }

    pub fn interrupt(&mut self) -> bool {
        self.tx.take().is_some()
    }

    pub fn submit(&mut self, drawables: TraitStack<dyn Drawable + 'static>) {
        if let Some(tx) = &mut self.tx {
            tx.send(drawables).ok();
        }
    }

    pub fn join(self) -> thread::Result<StoryboardSurfaceRenderer> {
        self.handle.join()
    }
}
