/*
 * Created on Sat Nov 13 2021
 *
 * Copyright (c) storycraft. Licensed under the MIT Licence.
 */

pub type DefaultPropData = ();
pub type DefaultStateData = ();

pub trait State<P = DefaultPropData, S = DefaultStateData> {
    fn load(&mut self, system_prop: &P);
    fn unload(&mut self, system_prop: &P);

    fn update(&mut self, system_prop: &P, system_state: &mut S) -> StateStatus<P, S>;
}

pub enum StateStatus<P = DefaultPropData, S = DefaultStateData> {
    Poll,
    PushState(Box<dyn State<P, S>>),
    TransitionState(Box<dyn State<P, S>>),
    PopState,
    Exit,
}

pub struct StateSystem<'a, P = DefaultPropData, S = DefaultStateData> {
    stack: Vec<Box<dyn State<P, S> + 'a>>
}

impl<'a, P, S> StateSystem<'a, P, S> {
    #[inline]
    pub fn new(inital_state: Box<dyn State<P, S> + 'a>, system_prop: &P) -> Self {
        Self::with_capacity(1, inital_state, system_prop)
    }

    pub fn with_capacity(
        capacity: usize,
        mut inital_state: Box<dyn State<P, S> + 'a>,
        system_prop: &P
    ) -> Self {
        let mut stack = Vec::with_capacity(capacity);

        inital_state.load(&system_prop);
        stack.push(inital_state);

        Self { stack }
    }

    pub fn run(&mut self, system_prop: &P, system_state: &mut S) {
        if let Some(state) = self.stack.last_mut() {
            match state.update(system_prop, system_state) {
                StateStatus::Poll => {}

                StateStatus::PushState(mut next_state) => {
                    next_state.load(system_prop);
                    self.stack.push(next_state);
                }

                StateStatus::TransitionState(mut next_state) => {
                    self.stack.pop().unwrap().unload(system_prop);
                    next_state.load(system_prop);
                    self.stack.push(next_state);
                }

                StateStatus::PopState => {
                    self.stack.pop().unwrap().unload(system_prop);
                }

                StateStatus::Exit => {
                    for mut state in self.stack.drain(..) {
                        state.unload(system_prop);
                    }
                }
            }
        }
    }

    pub fn finished(&self) -> bool {
        self.stack.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use crate::state::{State, StateStatus};

    use super::StateSystem;

    #[test]
    pub fn state_test() {
        struct SampleA {
            num: i32,
        }

        impl State for SampleA {
            fn load(&mut self, _: &()) {}
            fn unload(&mut self, _: &()) {}

            fn update(&mut self, _: &(), _: &mut ()) -> StateStatus {
                println!("SampleA: {}", self.num);

                StateStatus::PushState(Box::new(SampleB {
                    text: "asdf".to_string(),
                }))
            }
        }

        struct SampleB {
            text: String,
        }

        impl State for SampleB {
            fn load(&mut self, _: &()) {}
            fn unload(&mut self, _: &()) {}

            fn update(&mut self, _: &(), _: &mut ()) -> StateStatus {
                println!("SampleB: {}", self.text);

                StateStatus::Exit
            }
        }

        let mut system = StateSystem::new(Box::new(SampleA { num: 1 }), &());

        let mut counter = 0;
        while !system.finished() {
            system.run(&(), &mut ());
            counter += 1;
        }

        assert_eq!(counter, 2);
    }
}
