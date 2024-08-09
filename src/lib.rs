use std::{any::TypeId, collections::HashMap, hash::Hasher};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum State {
    Running,
    Stopped,
}

pub type Id = (TypeId, u64);

pub trait GetId {
    fn get_id(&self) -> Id;
}

impl<T: std::hash::Hash + 'static> GetId for T {
    fn get_id(&self) -> Id {
        let type_id = TypeId::of::<T>();
        let mut hasher = std::collections::hash_map::DefaultHasher::new();
        self.hash(&mut hasher);
        let hash = hasher.finish();
        (type_id, hash)
    }
}

pub trait System {
    fn step(&mut self, value: *mut u8) -> State;
}

pub trait Preprocessor<In> {
    fn preprocess(&mut self, value: &In) -> *mut u8;

    /// # Safety
    /// This function is unsafe because it is the responsibility of the implementor to ensure that
    /// the value is correctly cleaned up.
    /// This function should only ever be called on pointers that point to the correct type.
    unsafe fn cleanup(&mut self, value: *mut u8);
}

pub struct SystemPair {
    id: Id,
    system: Box<dyn System>,
}

impl SystemPair {
    pub fn new<S>(id: Id, system: S) -> Self
    where
        S: System + 'static,
    {
        SystemPair {
            id,
            system: Box::new(system),
        }
    }
}

pub struct PreprocessorPair<In> {
    id: Id,
    preprocessor: Box<dyn Preprocessor<In>>,
}

impl<In> PreprocessorPair<In> {
    pub fn new<P>(preprocessor: P) -> Self
    where
        P: Preprocessor<In> + std::hash::Hash + 'static,
    {
        // We do this once on creation to ensure that the id is consistent
        // Otherwise changes in internal state could change the id
        let id = preprocessor.get_id();

        PreprocessorPair {
            id,
            preprocessor: Box::new(preprocessor),
        }
    }

    pub fn id(&self) -> Id {
        self.id
    }
}

pub struct Step<In> {
    preprocessors: Vec<PreprocessorPair<In>>,
    alive: Vec<SystemPair>,
    dead: Vec<SystemPair>,
}

impl<In> Default for Step<In> {
    fn default() -> Self {
        Step {
            preprocessors: Vec::new(),
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

        self.alive
            .into_iter()
            .chain(self.dead)
            .map(|pair| pair.system)
            .collect()
    }

    pub fn add_system(&mut self, pair: SystemPair) {
        self.alive.push(pair);
    }

    pub fn add_preprocessor(&mut self, pair: PreprocessorPair<In>) {
        self.preprocessors.push(pair);
    }

    fn step(&mut self, value: In) -> State {
        let alive = std::mem::take(&mut self.alive);

        let mut preprocessed = HashMap::new();

        for pair in self.preprocessors.iter_mut() {
            let result = pair.preprocessor.preprocess(&value);
            preprocessed.insert(pair.id, result);
        }

        for mut pair in alive.into_iter() {
            let input = preprocessed.get(&pair.id).unwrap();

            if pair.system.step(*input) == State::Stopped {
                self.dead.push(pair);
            } else {
                self.alive.push(pair);
            }
        }

        for ((_, value), pair) in preprocessed.into_iter().zip(self.preprocessors.iter_mut()) {
            unsafe {
                pair.preprocessor.cleanup(value);
            }
        }

        if self.alive.is_empty() {
            State::Stopped
        } else {
            State::Running
        }
    }
}
