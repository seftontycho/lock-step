use std::rc::Rc;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum State {
    Running,
    Stopped,
}

pub trait System<In, Out> {
    fn step(&mut self, value: Rc<In>) -> State;
    fn report(&self) -> Out;
}

pub struct Step<Stream, In, Out>
where
    Stream: IntoIterator<Item = In>,
{
    stream: Option<Stream>,
    alive: Vec<Box<dyn System<In, Out>>>,
    dead: Vec<Box<dyn System<In, Out>>>,
}

impl<Stream, In, Out> Step<Stream, In, Out>
where
    Stream: IntoIterator<Item = In>,
{
    pub fn from_stream(stream: Stream) -> Self {
        Step {
            stream: Some(stream),
            alive: Vec::new(),
            dead: Vec::new(),
        }
    }

    pub fn run(mut self) -> Vec<Out> {
        let stream = self.stream.take().unwrap();

        for value in stream {
            if self.step(value) == State::Stopped {
                break;
            }
        }

        self.alive
            .into_iter()
            .chain(self.dead)
            .map(|system| system.report())
            .collect()
    }

    pub fn add_system<S: Default + System<In, Out> + 'static>(mut self) -> Self {
        self.alive.push(Box::new(S::default()));
        self
    }

    fn step(&mut self, value: In) -> State {
        let alive = std::mem::take(&mut self.alive);

        let value = Rc::new(value);

        for mut system in alive.into_iter() {
            if system.step(value.clone()) == State::Stopped {
                self.dead.push(system);
            } else {
                self.alive.push(system);
            }
        }

        if self.alive.is_empty() {
            State::Stopped
        } else {
            State::Running
        }
    }
}
