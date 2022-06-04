/*
 * Created on Sat Nov 13 2021
 *
 * Copyright (c) storycraft. Licensed under the MIT Licence.
 */

//! Module providing general purpose state system.

use std::fmt::Debug;

use smallvec::{SmallVec, smallvec};

pub type DefaultPropData = ();
pub type DefaultStateData = ();

/// StateData for [StateSystem].
/// 
/// Current implementation uses separate trait due to lifetime mangement problem.
pub trait StateData {
    type Prop<'p>;
    type State<'s>;
}

#[derive(Debug)]
pub struct DefaultSystemData;

impl StateData for DefaultSystemData {
    type Prop<'p> = ();
    type State<'s> = ();
}

/// A state holds mutable data as a part of system
pub trait State<Data: StateData> {
    /// Invoked before loaded by [StateSystem]
    fn load<'p>(&mut self, system_prop: &Data::Prop<'p>);

    /// Invoked after unloaded by [StateSystem]
    fn unload<'p>(&mut self, system_prop: &Data::Prop<'p>);

    /// Invoked when [StateSystem::run] called. Update State and return next status for System.
    fn update<'p, 's>(
        &mut self,
        system_prop: &Data::Prop<'p>,
        system_state: &mut Data::State<'s>,
    ) -> StateStatus<Data>;
}

impl<Data> Debug for dyn State<Data> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("State").finish_non_exhaustive()
    }
}

/// Next status for [StateSystem] from [State]
pub enum StateStatus<Data> {
    /// Keep current [State] and request to invoke [State::update] immediately when possible
    Poll,

    /// Keep current [State] but caller [StateSystem] can determine next update
    Wait,

    /// Push next [State] to caller [StateSystem]
    PushState(Box<dyn State<Data>>),

    /// Push next State and pop current [State] from caller [StateSystem]
    TransitionState(Box<dyn State<Data>>),

    /// Pop Current [State]
    PopState,

    /// Finish caller [StateSystem]
    Exit,
}

/// [StateSystem] status after single run
#[derive(Debug, Clone, Copy)]
pub enum SystemStatus {
    /// Request to run immediately when possible
    Poll,

    /// Run if any outer event happens.
    Wait,
}

/// A system holds a stack of [State] which can work as state machine.
///
/// System have one initial [State] after initalization.
/// The system is finished and cannot be run more when the stack becomes empty.
#[derive(Debug)]
pub struct StateSystem<Data> {
    stack: SmallVec<[Box<dyn State<Data>>; 4]>,
}

impl<Data: StateData> StateSystem<Data> {
    /// Initialize new [StateSystem] and load initial [State]
    pub fn new<'p>(
        mut initial_state: Box<dyn State<Data> + 'static>,
        system_prop: &Data::Prop<'p>,
    ) -> Self {
        initial_state.load(&system_prop);

        let stack = smallvec![initial_state];
        Self { stack }
    }

    /// Update latest [State] on stack of system and return [SystemStatus] for next run.
    pub fn run<'p, 's>(
        &mut self,
        system_prop: &Data::Prop<'p>,
        system_state: &mut Data::State<'s>,
    ) -> SystemStatus {
        if let Some(state) = self.stack.last_mut() {
            match state.update(system_prop, system_state) {
                StateStatus::Poll => SystemStatus::Poll,

                StateStatus::Wait => SystemStatus::Wait,

                StateStatus::PushState(mut next_state) => {
                    next_state.load(system_prop);
                    self.stack.push(next_state);

                    SystemStatus::Poll
                }

                StateStatus::TransitionState(mut next_state) => {
                    self.stack.pop().unwrap().unload(system_prop);
                    next_state.load(system_prop);
                    self.stack.push(next_state);

                    SystemStatus::Poll
                }

                StateStatus::PopState => {
                    self.stack.pop().unwrap().unload(system_prop);

                    SystemStatus::Poll
                }

                StateStatus::Exit => {
                    for mut state in self.stack.drain(..) {
                        state.unload(system_prop);
                    }

                    SystemStatus::Poll
                }
            }
        } else {
            SystemStatus::Poll
        }
    }

    /// Check if system is finished
    pub fn finished(&self) -> bool {
        self.stack.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use crate::state::{State, StateStatus, DefaultSystemData};

    use super::StateSystem;

    #[test]
    pub fn state_test() {
        struct SampleA {
            num: i32,
        }

        impl State<DefaultSystemData> for SampleA {
            fn load(&mut self, _: &()) {}
            fn unload(&mut self, _: &()) {}

            fn update(&mut self, _: &(), _: &mut ()) -> StateStatus<DefaultSystemData> {
                println!("SampleA: {}", self.num);

                StateStatus::PushState(Box::new(SampleB {
                    text: "asdf".to_string(),
                }))
            }
        }

        struct SampleB {
            text: String,
        }

        impl State<DefaultSystemData> for SampleB {
            fn load(&mut self, _: &()) {}
            fn unload(&mut self, _: &()) {}

            fn update(&mut self, _: &(), _: &mut ()) -> StateStatus<DefaultSystemData> {
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
