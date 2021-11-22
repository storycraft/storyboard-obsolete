/*
 * Created on Mon Nov 22 2021
 *
 * Copyright (c) storycraft. Licensed under the MIT Licence.
 */

use dynstack::{DynStack, dyn_push};

use super::pass::StoryboardComputePass;

pub trait ComputeState: Send + Sync {
    fn compute<'r>(&'r mut self, pass: &mut StoryboardComputePass<'r>);
}

pub struct StoryboardComputer<'a> {
    compute_state_queue: DynStack<dyn ComputeState + 'a>
}

impl<'a> StoryboardComputer<'a> {
    pub fn new() -> Self {
        Self {
            compute_state_queue: DynStack::new()
        }
    }

    pub fn append(&mut self, compute_state: impl ComputeState + 'a) {
        dyn_push!(self.compute_state_queue, compute_state);
    }

    pub fn compute<'cpass>(
        &'cpass mut self,
        pass: &mut StoryboardComputePass<'cpass>,
    ) {
        for compute_state in self.compute_state_queue.iter_mut() {
            compute_state.compute(pass);
        }
    }
}

#[derive(Debug)]
pub struct ComputeData {

}

impl ComputeData {

}
