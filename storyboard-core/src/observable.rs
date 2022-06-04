/*
 * Created on Mon Sep 06 2021
 *
 * Copyright (c) storycraft. Licensed under the MIT Licence.
 */

use std::ops::{Deref, DerefMut};

/// Track changes of inner data
#[derive(Debug, Clone, Default)]
pub struct Observable<T> {
    inner: T,
    changed: bool,
}

impl<T> Observable<T> {
    /// Create new [Observable] with changed state.
    /// It is default constructor for any conversion implementation.
    pub const fn new(data: T) -> Self {
        Self {
            inner: data,
            changed: true,
        }
    }

    /// Create new [Observable] with unchanged state
    pub const fn new_unchanged(data: T) -> Self {
        Self {
            inner: data,
            changed: false,
        }
    }

    pub fn changed(this: &Self) -> bool {
        this.changed
    }

    pub fn into_inner(this: Self) -> T {
        this.inner
    }

    /// Invalidate inner data change flag.
    /// Return true if changes unmarked.
    pub fn invalidate(this: &mut Self) -> bool {
        if this.changed {
            this.changed = false;

            true
        } else {
            false
        }
    }

    /// Mark data changed.
    pub fn mark(this: &mut Self) {
        if !this.changed {
            this.changed = true;
        }
    }
}

impl<T> Deref for Observable<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl<T> DerefMut for Observable<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        Self::mark(self);
        &mut self.inner
    }
}

impl<T> From<T> for Observable<T> {
    fn from(value: T) -> Self {
        Observable::new(value)
    }
}

#[cfg(test)]
#[test]
pub fn observable_test() {
    let mut data = Observable::new_unchanged(2);

    assert!(Observable::changed(&data));
    
    data = 2.into();

    assert!(!Observable::changed(&data));
}
