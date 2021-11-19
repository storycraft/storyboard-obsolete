/*
 * Created on Fri Nov 12 2021
 *
 * Copyright (c) storycraft. Licensed under the MIT Licence.
 */

use std::sync::atomic::{AtomicUsize, Ordering};

pub type NodeId = usize;

#[derive(Debug)]
pub struct IdGenerator {
    next: AtomicUsize
}

impl IdGenerator {
    pub const fn new() -> Self {
        Self {
            next: AtomicUsize::new(0)
        }
    }

    pub fn gen(&self) -> NodeId {
        let id = self.next.load(Ordering::Relaxed);

        self.next.fetch_add(1, Ordering::Relaxed);

        id
    }
}

#[cfg(test)]
mod tests {
    use super::IdGenerator;

    #[test]
    pub fn id_gen_test() {
        let generator = IdGenerator::new();

        assert_eq!(generator.gen(), 0);
        assert_eq!(generator.gen(), 1);
        assert_eq!(generator.gen(), 2);
    }
}
