use storyboard_state::{DefaultSystemData, State, StateStatus, StateSystem};

#[test]
fn state_test() {
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