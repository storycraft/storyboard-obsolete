/*
 * Created on Fri Jun 03 2022
 *
 * Copyright (c) storycraft. Licensed under the MIT Licence.
 */

use std::{any::TypeId, fmt::Debug};

use parking_lot::RwLock;
use rustc_hash::FxHashMap;

#[derive(Debug)]
/// Resource store for storing type erased local resource data
pub struct Store {
    map: RwLock<FxHashMap<TypeId, *mut ()>>,
}

// SAFETY: Values in Store is Send
unsafe impl Send for Store {}

// SAFETY: Values in Store is Sync
unsafe impl Sync for Store {}

impl Store {
    pub fn new() -> Self {
        Self {
            map: RwLock::new(FxHashMap::default()),
        }
    }

    pub fn get<'a, T: StoreResources<Context> + Sized + 'static, Context>(
        &'a self,
        ctx: &Context,
    ) -> &'a T {
        if let Some(item) = self.map.read().get(&TypeId::of::<T>()) {
            // SAFETY: Value was created with valid type and was type erased.
            return unsafe { &*(*item as *mut T) };
        }

        let item = Box::new(T::initialize(self, ctx));
        self.map
            .write()
            .insert(TypeId::of::<T>(), Box::into_raw(item) as *mut ());

        self.get(ctx)
    }
}

impl Drop for Store {
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
    fn initialize(store: &Store, ctx: &Context) -> Self;
}

#[cfg(test)]
mod tests {
    use crate::store::{Store, StoreResources};

    #[test]
    fn store_test() {
        let store = Store::new();

        struct ResA {
            pub number: i32,
        }

        impl StoreResources<()> for ResA {
            fn initialize(_: &Store, _: &()) -> Self {
                ResA { number: 32 }
            }
        }

        struct ResB {
            pub string: String,
        }

        impl StoreResources<()> for ResB {
            fn initialize(_: &Store, _: &()) -> Self {
                ResB {
                    string: "test".into(),
                }
            }
        }

        assert_eq!(store.get::<ResA, _>(&()).number, 32);
        assert_eq!(store.get::<ResB, _>(&()).string, "test");
    }
}
