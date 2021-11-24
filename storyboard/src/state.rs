/*
 * Created on Sat Nov 13 2021
 *
 * Copyright (c) storycraft. Licensed under the MIT Licence.
 */

//! Module providing general purpose state system

pub type DefaultPropData = ();
pub type DefaultStateData = ();

/// A state holds mutable data as a part of system
pub trait State<P = DefaultPropData, S = DefaultStateData> {
    
    /// Invoked before loaded by [StateSystem]
    fn load(&mut self, system_prop: &P);

    /// Invoked after unloaded by [StateSystem]
    fn unload(&mut self, system_prop: &P);

    /// Invoked when [StateSystem::run] called. Update State and return next status for System.
    fn update(&mut self, system_prop: &P, system_state: &mut S) -> StateStatus<P, S>;
}

/// Next status for [StateSystem] from [State]
pub enum StateStatus<P = DefaultPropData, S = DefaultStateData> {
    /// Keep current [State] and request to invoke [State::update] immediately when possible
    Poll,
    
    /// Keep current [State] but caller [StateSystem] can determine next update
    Wait,

    /// Push next [State] to caller [StateSystem]
    PushState(Box<dyn State<P, S>>),

    /// Push next State and pop current [State] from caller [StateSystem]
    TransitionState(Box<dyn State<P, S>>),

    /// Pop Current [State]
    PopState,

    /// Finish caller [StateSystem]
    Exit,
}

/// [StateSystem] status after single run
#[derive(Debug, Clone, Copy)]
pub enum SystemStatus {
    /// request to run [StateSystem::run] immediately when possible
    Poll,

    /// request to run [StateSystem::run] after waiting any event or task.
    /// The user can determine to wait or not.
    Wait
}

/// A system holds a stack of [State] which can work as state machine.
/// 
/// System have one initial [State] after initalization.
/// The system is finished and cannot be run more when the stack becomes empty.
pub struct StateSystem<'a, P = DefaultPropData, S = DefaultStateData> {
    stack: Vec<Box<dyn State<P, S> + 'a>>
}

impl<'a, P, S> StateSystem<'a, P, S> {

    /// Initialize new [StateSystem] and load initial [State] 
    pub fn new(mut initial_state: Box<dyn State<P, S> + 'a>, system_prop: &P) -> Self {
        let mut stack = Vec::with_capacity(1);

        initial_state.load(&system_prop);
        stack.push(initial_state);

        Self { stack }
    }

    /// Update latest [State] on stack of system and return [SystemStatus] for next run.
    pub fn run(&mut self, system_prop: &P, system_state: &mut S) -> SystemStatus {
        if let Some(state) = self.stack.last_mut() {
            match state.update(system_prop, system_state) {
                StateStatus::Poll => {
                    SystemStatus::Poll
                }

                StateStatus::Wait => {
                    SystemStatus::Wait
                }

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
