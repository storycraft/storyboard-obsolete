use downcast::{downcast, Any};
use rustc_hash::{FxHashSet, FxHasher};
use std::{marker::PhantomData, hash::BuildHasherDefault, fmt::Debug};
use storyboard_render::task::RenderTask;

use indexmap::IndexMap;

#[derive(Debug)]
pub struct FrameContainer {
    state_map: IndexMap<usize, Box<dyn FrameComponent>, BuildHasherDefault<FxHasher>>,
    dirty_list: FxHashSet<usize>,
}

impl FrameContainer {
    pub fn new() -> Self {
        Self {
            state_map: IndexMap::default(),
            dirty_list: FxHashSet::default(),
        }
    }

    pub fn invalidated(&self) -> bool {
        !self.dirty_list.is_empty()
    }

    pub fn values(&self) -> impl Iterator<Item = &Box<dyn FrameComponent>> {
        self.state_map.values()
    }

    pub fn contains<T: FrameComponent>(&self, key: &FrameComponentKey<T>) -> bool {
        self.state_map.contains_key(&key.0)
    }

    pub fn get<T: FrameComponent>(&self, key: &FrameComponentKey<T>) -> Option<&T> {
        let boxed = self.state_map.get(&key.0)?;

        boxed.downcast_ref().ok()
    }

    pub fn get_mut<T: FrameComponent>(&mut self, key: &FrameComponentKey<T>) -> Option<&mut T> {
        let boxed = self.state_map.get_mut(&key.0)?;
        self.dirty_list.insert(key.0);

        boxed.downcast_mut().ok()
    }

    pub fn add_component<T: FrameComponent>(&mut self, state: T) -> FrameComponentKey<T> {
        let boxed = Box::new(state);
        let key = FrameComponentKey((&*boxed as *const _) as usize, PhantomData);

        self.state_map.insert(key.0, boxed);
        self.dirty_list.insert(key.0);

        key
    }

    pub fn remove<T: FrameComponent>(&mut self, key: FrameComponentKey<T>) -> Option<T> {
        let boxed = self.state_map.remove(&key.0)?.downcast::<T>().ok()?;
        Some(*boxed)
    }

    pub fn update(&mut self) {
        self.dirty_list.retain(|id| {
            if let Some(state) = self.state_map.get_mut(id) {
                if state.expired() {
                    self.state_map.remove(id);
                    false
                } else {
                    state.update()
                }
            } else {
                false
            }
        });
    }
}

pub trait FrameComponent: Any {
    fn expired(&self) -> bool;

    fn update(&mut self) -> bool;

    fn draw(&self, task: &mut RenderTask);
}

downcast!(dyn FrameComponent);

impl Debug for dyn FrameComponent {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ScreenObject").finish_non_exhaustive()
    }
}

#[derive(Debug)]
pub struct FrameComponentKey<T>(usize, PhantomData<T>);

impl<T> Clone for FrameComponentKey<T> {
    fn clone(&self) -> Self {
        Self(self.0, PhantomData)
    }
}

impl<T> Copy for FrameComponentKey<T> {}
