use std::{
    any::TypeId,
    collections::HashMap,
    hash::{Hash, Hasher},
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum State {
    Running,
    Stopped,
}

pub trait System {
    fn step(&mut self, value: &[&[u8]]) -> State;
}

pub trait Preprocessor<In> {
    fn preprocess(&mut self, value: &In) -> Vec<u8>;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct RegistrationKey {
    type_id: TypeId,
    hash: u64,
}

impl RegistrationKey {
    fn new<T: Hash + 'static>(value: &T) -> Self {
        let type_id = TypeId::of::<T>();
        let mut hasher = std::collections::hash_map::DefaultHasher::new();
        value.hash(&mut hasher);
        let hash = hasher.finish();

        RegistrationKey { type_id, hash }
    }
}

pub struct Step<In> {
    preprocessors: HashMap<RegistrationKey, Box<dyn Preprocessor<In>>>,
    alive: Vec<(Box<dyn System>, Vec<RegistrationKey>)>,
    dead: Vec<Box<dyn System>>,
}

impl<In> Default for Step<In> {
    fn default() -> Self {
        Step {
            preprocessors: HashMap::new(),
            alive: Vec::new(),
            dead: Vec::new(),
        }
    }
}

impl<In> Step<In> {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn run<Stream>(mut self, stream: Stream) -> Vec<Box<dyn System>>
    where
        Stream: IntoIterator<Item = In>,
    {
        for value in stream {
            if self.step(value) == State::Stopped {
                break;
            }
        }

        self.alive.into_iter().map(|(system, _)| system).collect()
    }

    pub fn add_system<S: System + 'static>(&mut self, system: S, keys: &[RegistrationKey]) {
        self.alive.push((Box::new(system), keys.to_vec()));
    }

    pub fn add_preprocessor<P: Preprocessor<In> + Hash + 'static>(
        &mut self,
        preprocessor: P,
    ) -> RegistrationKey {
        let key = RegistrationKey::new(&preprocessor);
        self.preprocessors.insert(key, Box::new(preprocessor));
        key
    }

    fn step(&mut self, value: In) -> State {
        let alive = std::mem::take(&mut self.alive);

        let mut preprocessed = HashMap::new();

        for (key, preprocessor) in self.preprocessors.iter_mut() {
            let result = preprocessor.preprocess(&value);
            preprocessed.insert(key, result);
        }

        for (mut system, keys) in alive.into_iter() {
            let mut inputs = Vec::new();

            for key in keys.iter() {
                inputs.push(preprocessed.get(key).unwrap().as_slice());
            }

            match system.step(inputs.as_slice()) {
                State::Running => self.alive.push((system, keys)),
                State::Stopped => self.dead.push(system),
            }
        }

        if self.alive.is_empty() {
            State::Stopped
        } else {
            State::Running
        }
    }
}
