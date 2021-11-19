/*
 * Created on Mon Nov 15 2021
 *
 * Copyright (c) storycraft. Licensed under the MIT Licence.
 */

use std::{hash::Hash};

use rustc_hash::FxHashMap;

use crate::id_gen::{IdGenerator, NodeId};

#[derive(Debug)]
pub struct Store<I, V> {
    id_gen: IdGenerator,
    map: FxHashMap<I, V>,
}

impl<I: From<NodeId> + Hash + Eq + Copy, V> Store<I, V> {
    pub fn new() -> Self {
        Self {
            id_gen: IdGenerator::new(),
            map: FxHashMap::default(),
        }
    }

    pub fn store(&mut self, value: V) -> I {
        let id = self.id_gen.gen();

        let identifier = I::from(id);

        self.map.insert(identifier, value);

        identifier
    }

    pub fn get(&self, identifier: &I) -> Option<&V> {
        self.map.get(identifier)
    }

    pub fn get_mut(&mut self, identifier: &I) -> Option<&mut V> {
        self.map.get_mut(identifier)
    }

    pub fn remove(&mut self, identifier: &I) -> Option<V> {
        self.map.remove(identifier)
    }
}
