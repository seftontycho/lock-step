use std::rc::Rc;

use lock_step::{State, Step, System};

struct Recorder<const LENGTH: usize> {
    values: Vec<usize>,
}

impl<const LENGTH: usize> Default for Recorder<LENGTH> {
    fn default() -> Self {
        Recorder { values: Vec::new() }
    }
}

impl<const LENGTH: usize> System<usize, usize> for Recorder<LENGTH> {
    fn step(&mut self, value: Rc<usize>) -> State {
        self.values.push(*value.clone());

        if self.values.len() >= LENGTH {
            State::Stopped
        } else {
            State::Running
        }
    }

    fn report(&self) -> usize {
        self.values.iter().sum()
    }
}

fn main() {
    let reports = Step::from_stream(1..=1000)
        .add_system::<Recorder<10>>()
        .add_system::<Recorder<20>>()
        .add_system::<Recorder<100>>()
        .add_system::<Recorder<10000>>()
        .run();

    for report in reports {
        println!("{}", report);
    }
}
