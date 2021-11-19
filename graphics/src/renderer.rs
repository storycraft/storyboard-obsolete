/*
 * Created on Tue Sep 28 2021
 *
 * Copyright (c) storycraft. Licensed under the MIT Licence.
 */

use std::collections::VecDeque;

use smallbox::{smallbox, SmallBox};
use wgpu::{CommandEncoder, RenderPass};

use crate::component::{DrawState, RenderState};

use super::{
    component::Drawable,
    context::{DrawContext, RenderContext},
    pass::StoryboardRenderPass,
};

pub struct StoryboardRenderer<'a> {
    draw_states: VecDeque<(f32, SmallBox<dyn DrawState<'a> + 'a, [u64; 64]>)>,
    render_states: RenderStateQueue<'a>,
}

impl<'a> StoryboardRenderer<'a> {
    pub const DEFAULT_CAPACITY: usize = 32;

    pub fn new() -> Self {
        Self {
            draw_states: VecDeque::with_capacity(Self::DEFAULT_CAPACITY),
            render_states: RenderStateQueue(Vec::new()),
        }
    }

    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            draw_states: VecDeque::with_capacity(capacity),
            render_states: RenderStateQueue(Vec::new()),
        }
    }

    pub fn prepare(&mut self, context: &mut DrawContext, encoder: &mut CommandEncoder) {
        self.render_states.reserve(self.draw_states.len());

        let len = self.draw_states.len() as f32;
        for (i, mut draw_state) in self.draw_states.drain(..).rev() {
            let depth = 1.0 - i / len;

            draw_state.prepare(context, depth, encoder, &mut self.render_states);
        }
    }

    pub fn render(&mut self, context: &RenderContext, pass: RenderPass) {
        {
            let mut pass = StoryboardRenderPass::new(pass);

            for state in &mut self.render_states.0 {
                state.render(context, &mut pass);
            }
        }

        self.render_states.clear();
    }

    pub fn append(&mut self, drawable: Drawable<impl DrawState<'a> + 'a>) {
        let index = self.draw_states.len() as f32;
        let draw_state = drawable.state;

        if drawable.opaque {
            self.draw_states.push_back((index, smallbox!(draw_state)));
        } else {
            self.draw_states.push_front((index, smallbox!(draw_state)));
        }
    }
}

pub type BoxedRenderState<'a> = SmallBox<dyn RenderState + 'a, [u64; 16]>;

pub struct RenderStateQueue<'a>(Vec<BoxedRenderState<'a>>);

impl<'a> RenderStateQueue<'a> {
    #[inline(always)]
    pub fn push(&mut self, state: impl RenderState + 'a) {
        self.0.push(smallbox!(state));
    }

    #[inline]
    fn reserve(&mut self, additional: usize) {
        self.0.reserve(additional)
    }

    #[inline]
    fn clear(&mut self) {
        self.0.clear()
    }
}
