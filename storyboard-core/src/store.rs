/*
 * Created on Fri Jun 03 2022
 *
 * Copyright (c) storycraft. Licensed under the MIT Licence.
 */

use std::{any::TypeId, fmt::Debug, marker::PhantomData};

use parking_lot::RwLock;
use rustc_hash::FxHashMap;

#[derive(Debug)]
/// Concurrent resource store for storing type erased local resource data
pub struct Store<Context> {
    map: RwLock<FxHashMap<TypeId, *mut ()>>,
    phantom: PhantomData<Context>,
}

// SAFETY: Values in Store is Send
unsafe impl<Context> Send for Store<Context> {}

// SAFETY: Values in Store is Sync
unsafe impl<Context> Sync for Store<Context> {}

pub type DefaultStore = Store<()>;

impl<Context> Store<Context> {
    pub fn new() -> Self {
        Self {
            map: RwLock::new(FxHashMap::default()),
            phantom: PhantomData,
        }
    }

    pub fn get<'a, T: StoreResources<Context> + Sized + 'static>(&'a self, ctx: &Context) -> &'a T {
        if let Some(item) = self.map.read().get(&TypeId::of::<T>()).cloned() {
            // SAFETY: Value was created with valid type and was type erased.
            return unsafe { &*(item as *mut T) };
        }

        let item = Box::new(T::initialize(self, ctx));
        self.map
            .write()
            .insert(TypeId::of::<T>(), Box::into_raw(item) as *mut ());

        self.get(ctx)
    }
}

impl<T> Drop for Store<T> {
    fn drop(&mut self) {
        // SAFETY: pointer created with [Box::into_raw]
        unsafe {
            for (_, value) in self.map.write().drain() {
                Box::from_raw(value);
            }
        }
    }
}

pub trait StoreResources<Context>: Send + Sync {
    fn initialize(store: &Store<Context>, ctx: &Context) -> Self;
}

#[cfg(test)]
mod tests {
    use crate::store::StoreResources;

    use super::DefaultStore;

    #[test]
    fn store_test() {
        let store = DefaultStore::new();

        struct ResA {
            pub number: i32,
        }

        impl StoreResources<()> for ResA {
            fn initialize(_: &DefaultStore, _: &()) -> Self {
                ResA { number: 32 }
            }
        }

        struct ResB {
            pub string: String,
        }

        impl StoreResources<()> for ResB {
            fn initialize(_: &DefaultStore, _: &()) -> Self {
                ResB {
                    string: "test".into(),
                }
            }
        }

        assert_eq!(store.get::<ResA>(&()).number, 32);
        assert_eq!(store.get::<ResB>(&()).string, "test");
    }
}
