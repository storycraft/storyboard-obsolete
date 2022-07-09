use storyboard_core::{
    euclid::Transform3D,
    unit::{LogicalPixelUnit, RenderUnit},
};

use crate::{
    buffer::stream::{BufferStream, StreamBuffer},
    ScreenRect, shared::RenderScope,
};

/// [DrawContext] contains reference to backend, resources store, and stream for component data preparing
#[derive(Debug)]
pub struct DrawContext<'a> {
    pub scope: RenderScope<'a>,

    pub screen: ScreenRect,
    pub screen_matrix: Transform3D<f32, LogicalPixelUnit, RenderUnit>,

    pub vertex_stream: &'a mut BufferStream<'static>,
    pub index_stream: &'a mut BufferStream<'static>,
}

impl<'a> DrawContext<'a> {
    pub fn into_render_context(self) -> RenderContext<'a> {
        let backend = self.scope.backend();
        let vertex_stream = self
            .vertex_stream
            .finish(backend.device(), backend.queue());
        let index_stream = self
            .index_stream
            .finish(backend.device(), backend.queue());

        RenderContext {
            scope: self.scope,
            vertex_stream,
            index_stream,
        }
    }
}

/// [RenderContext] contains gpu device and stream for component rendering
#[derive(Debug)]
pub struct RenderContext<'a> {
    pub scope: RenderScope<'a>,

    pub vertex_stream: StreamBuffer<'a>,
    pub index_stream: StreamBuffer<'a>,
}
