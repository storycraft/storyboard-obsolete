use storyboard_core::store::{Store, StoreResources};
use wgpu::{Device, Queue, TextureFormat};

#[derive(Debug, Default)]
/// Shared backend data container
pub struct BackendShared {
    store: Store,
}

impl BackendShared {
    pub fn new() -> Self {
        Self {
            store: Store::new(),
        }
    }

    #[inline]
    pub const fn scope<'a>(&'a self, context: BackendScopeContext<'a>) -> BackendScope<'a> {
        BackendScope::new(context, self)
    }

    pub fn get<'a, T: StoreResources<BackendScopeContext<'a>>>(
        &self,
        backend: BackendScopeContext<'a>,
    ) -> &T {
        self.store.get(&backend)
    }
}

#[derive(Debug, Clone, Copy)]
pub struct BackendScope<'a> {
    container: &'a BackendShared,
    context: BackendScopeContext<'a>,
}

impl<'a> BackendScope<'a> {
    pub const fn new(context: BackendScopeContext<'a>, container: &'a BackendShared) -> Self {
        Self { context, container }
    }

    #[inline]
    pub const fn context(&self) -> BackendScopeContext {
        self.context
    }

    #[inline]
    pub const fn device(&self) -> &Device {
        self.context.device
    }

    #[inline]
    pub const fn queue(&self) -> &Queue {
        self.context.queue
    }

    #[inline]
    pub const fn container(&self) -> &BackendShared {
        self.container
    }

    pub fn get<T: StoreResources<BackendScopeContext<'a>>>(&self) -> &'a T {
        self.container.store.get(&self.context)
    }

    #[inline]
    pub const fn render_scope(self, container: &'a RenderShared) -> RenderScope {
        RenderScope::new(self, container)
    }
}

#[derive(Debug, Clone, Copy)]
pub struct BackendScopeContext<'a> {
    pub device: &'a Device,
    pub queue: &'a Queue,
}

#[derive(Debug)]
/// Shared render data container
pub struct RenderShared {
    texture_format: TextureFormat,
    store: Store,
}

impl RenderShared {
    pub fn new(texture_format: TextureFormat) -> Self {
        Self {
            texture_format,
            store: Store::new(),
        }
    }

    #[inline]
    pub const fn texture_format(&self) -> TextureFormat {
        self.texture_format
    }

    pub fn get<'a, T: StoreResources<RenderScopeContext<'a>>>(
        &self,
        backend: BackendScope<'a>,
    ) -> &T {
        self.store.get(&RenderScopeContext {
            backend,
            texture_format: self.texture_format,
        })
    }
}

#[derive(Debug, Clone, Copy)]
pub struct RenderScope<'a> {
    backend: BackendScope<'a>,
    container: &'a RenderShared,
}

impl<'a> RenderScope<'a> {
    pub const fn new(backend: BackendScope<'a>, container: &'a RenderShared) -> Self {
        Self { backend, container }
    }

    #[inline]
    pub fn context(&self) -> RenderScopeContext {
        RenderScopeContext {
            backend: self.backend,
            texture_format: self.container.texture_format,
        }
    }

    #[inline]
    pub const fn backend(&self) -> &BackendScope<'a> {
        &self.backend
    }

    #[inline]
    pub const fn container(&self) -> &RenderShared {
        self.container
    }

    #[inline]
    pub const fn texture_format(&self) -> TextureFormat {
        self.container.texture_format
    }

    pub fn is_valid_for(&self, format: TextureFormat) -> bool {
        self.container.texture_format == format
    }

    pub fn get<T: for<'ctx> StoreResources<RenderScopeContext<'ctx>>>(&self) -> &'a T {
        self.container.store.get(&self.context())
    }
}

#[derive(Debug, Clone, Copy)]
pub struct RenderScopeContext<'a> {
    pub backend: BackendScope<'a>,
    pub texture_format: TextureFormat,
}
