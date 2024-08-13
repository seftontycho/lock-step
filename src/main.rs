use lock_step::{Preprocessor, State, Step, System};

#[derive(Hash)]
struct AddOne;

impl Preprocessor<i32> for AddOne {
    fn preprocess(&mut self, value: &i32) -> Vec<u8> {
        let value = value + 1;
        value.to_be_bytes().to_vec()
    }
}

#[derive(Hash)]
struct ToString;

impl Preprocessor<i32> for ToString {
    fn preprocess(&mut self, value: &i32) -> Vec<u8> {
        value.to_string().into_bytes()
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
    fn step(&mut self, value: &[&[u8]]) -> State {
        for v in value {
            let v = i32::from_be_bytes(v.to_vec().try_into().unwrap());
            println!("{}", v);
        }

        self.count += 1;

        if self.count >= self.max {
            State::Stopped
        } else {
            State::Running
        }
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
    fn step(&mut self, value: &[&[u8]]) -> State {
        for v in value {
            let v = String::from_utf8(v.to_vec()).unwrap();
            println!("String: {}", v);
        }

        self.count += 1;

        if self.count >= self.max {
            State::Stopped
        } else {
            State::Running
        }
    }
}

fn main() {
    // TODO: tidy up creation with macros
    let adder = AddOne;
    let stringer = ToString;

    let stream: Vec<i32> = (0..50).collect();

    let mut step = Step::new();

    let adder_key = step.add_preprocessor(adder);
    let stringer_key = step.add_preprocessor(stringer);

    let int_printer = IntPrinter::new(10);
    let string_printer = StringPrinter::new(5);

    step.add_system(int_printer, &[adder_key]);
    step.add_system(string_printer, &[stringer_key]);

    let _ = step.run(stream);
}
