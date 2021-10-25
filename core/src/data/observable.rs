/*
 * Created on Mon Sep 06 2021
 *
 * Copyright (c) storycraft. Licensed under the MIT Licence.
 */

/// Track changes of inner data
#[derive(Debug, Clone, Default)]
pub struct Observable<T> {
    inner: T,
    valid: bool,
}

impl<T> Observable<T> {
    pub fn new(data: T) -> Self {
        Self {
            inner: data,
            valid: false,
        }
    }

    pub fn new_valid(data: T) -> Self {
        Self {
            inner: data,
            valid: true,
        }
    }

    pub fn valid(&self) -> bool {
        self.valid
    }

    pub fn set(&mut self, value: T) {
        self.inner = value;
        self.mark();
    }

    pub fn inner(self) -> T {
        self.inner
    }

    /// unmark inner data changes.
    /// Return true if changes unmarked.
    pub fn unmark(&mut self) -> bool {
        if !self.valid {
            self.valid = true;

            true
        } else {
            false
        }
    }

    /// Mark data as changed
    pub fn mark(&mut self) {
        if self.valid {
            self.valid = false;
        }
    }
}

impl<T> AsRef<T> for Observable<T> {
    fn as_ref(&self) -> &T {
        &self.inner
    }
}

impl<T> AsMut<T> for Observable<T> {
    fn as_mut(&mut self) -> &mut T {
        self.mark();
        &mut self.inner
    }
}

#[cfg(test)]
#[test]
pub fn observable_test() {
    let mut data = Observable::new_valid(2);

    data.set(2);

    assert!(!data.valid());
}
