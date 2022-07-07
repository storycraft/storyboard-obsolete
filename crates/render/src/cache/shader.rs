use std::{collections::HashMap, ops::Deref};

use parking_lot::{RwLock, RwLockReadGuard};
use storyboard_core::store::{StoreResources, Store};
use wgpu::ShaderModule;

#[derive(Debug, Default)]
pub struct ShaderCache {
    cache: RwLock<HashMap<String, ShaderModule>>,
}

impl ShaderCache {
    pub fn new() -> Self {
        Self {
            cache: RwLock::new(HashMap::new()),
        }
    }

    pub fn get_or_create(
        &self,
        name: &str,
        func: impl Fn() -> ShaderModule,
    ) -> impl Deref<Target = ShaderModule> + '_ {
        if self.cache.read().contains_key(name) {
            RwLockReadGuard::map(self.cache.read(), |map| map.get(name).unwrap())
        } else {
            let shader_module = func();
            self.cache.write().insert(name.to_string(), shader_module);

            self.get_or_create(name, func)
        }
    }
}

impl<T> StoreResources<T> for ShaderCache {
    fn initialize(_: &Store, _: &T) -> Self {
        Self::new()
    }
}
