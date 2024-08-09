use lock_step::{Preprocessor, PreprocessorPair, Step, System, SystemPair};

#[derive(Hash)]
struct AddOne;

impl Preprocessor<i32> for AddOne {
    fn preprocess(&mut self, value: &i32) -> *mut u8 {
        Box::into_raw(Box::new(value + 1)) as *mut u8
    }

    unsafe fn cleanup(&mut self, value: *mut u8) {
        let _ = Box::from_raw(value as *mut i32);
    }
}

#[derive(Hash)]
struct ToString;

impl Preprocessor<i32> for ToString {
    fn preprocess(&mut self, value: &i32) -> *mut u8 {
        Box::into_raw(Box::new(value.to_string())) as *mut u8
    }

    unsafe fn cleanup(&mut self, value: *mut u8) {
        let _ = Box::from_raw(value as *mut String);
    }
}

struct IntPrinter {
    max: i32,
    count: i32,
}

impl IntPrinter {
    fn new(max: i32) -> Self {
        IntPrinter { max, count: 0 }
    }
}

impl System for IntPrinter {
    fn step(&mut self, value: *mut u8) -> lock_step::State {
        // Is this casting safe if we are sure that the value is an i32?
        let value = unsafe { Box::from_raw(value as *mut i32) };
        println!("Int: {}", value);

        // Need to leak here to avoid freeing before other systems try and use it
        let _ = Box::into_raw(value);

        self.count += 1;

        if self.count == self.max {
            return lock_step::State::Stopped;
        }

        lock_step::State::Running
    }
}

struct StringPrinter {
    max: i32,
    count: i32,
}

impl StringPrinter {
    fn new(max: i32) -> Self {
        StringPrinter { max, count: 0 }
    }
}

impl System for StringPrinter {
    fn step(&mut self, value: *mut u8) -> lock_step::State {
        // Is this casting safe if we are sure that the value is a String?
        let value = unsafe { Box::from_raw(value as *mut String) };
        println!("String: {}", value);

        // Need to leak here to avoid freeing before other systems try and use it
        let _ = Box::into_raw(value);

        self.count += 1;

        if self.count == self.max {
            return lock_step::State::Stopped;
        }

        lock_step::State::Running
    }
}

fn main() {
    // TODO: tidy up creation with macros
    let adder = AddOne;
    let stringer = ToString;

    let adder_pair = PreprocessorPair::new(adder);
    let stringer_pair = PreprocessorPair::new(stringer);

    let int_printer = IntPrinter::new(5);
    let str_printer = StringPrinter::new(10);

    let adder_system = SystemPair::new(adder_pair.id(), int_printer);
    let stringer_system = SystemPair::new(stringer_pair.id(), str_printer);

    let mut step = Step::new();

    step.add_preprocessor(adder_pair);
    step.add_preprocessor(stringer_pair);

    step.add_system(adder_system);
    step.add_system(stringer_system);

    let stream: Vec<i32> = (0..50).collect();

    let _ = step.run(stream);
}
