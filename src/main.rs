use std::{fmt::Debug, rc::Rc};

use lock_step::{State, Step, System};

trait Metric {
    fn record(&mut self, value: usize);
}

struct MetricRecorder<M: Metric> {
    metric: M,
}

impl<M: Metric + Debug> std::fmt::Debug for MetricRecorder<M> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("MetricRecorder")
            .field("metric", &self.metric)
            .finish()
    }
}

impl<M: Metric + Debug> System<usize> for MetricRecorder<M> {
    fn step(&mut self, value: Rc<usize>) -> State {
        self.metric.record(*value);
        State::Running
    }
}

impl<M: Metric> MetricRecorder<M> {
    fn new(metric: M) -> Self {
        MetricRecorder { metric }
    }
}

#[derive(Debug)]
struct Counter {
    count: usize,
}

impl Metric for Counter {
    fn record(&mut self, value: usize) {
        self.count += value;
    }
}

struct Average {
    sum: usize,
    count: usize,
}

impl std::fmt::Debug for Average {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Average")
            .field("avg", &(self.sum / self.count))
            .field("sum", &self.sum)
            .field("count", &self.count)
            .finish()
    }
}

impl Metric for Average {
    fn record(&mut self, value: usize) {
        self.sum += value;
        self.count += 1;
    }
}

#[derive(Debug)]
struct Maximum {
    max: usize,
}

impl Metric for Maximum {
    fn record(&mut self, value: usize) {
        if value > self.max {
            self.max = value;
        }
    }
}

#[derive(Debug)]
struct Minimum {
    min: usize,
}

impl Metric for Minimum {
    fn record(&mut self, value: usize) {
        if value < self.min {
            self.min = value;
        }
    }
}

fn main() {
    let workers = Step::from_stream(1..=1000)
        .add_system(MetricRecorder::new(Counter { count: 0 }))
        .add_system(MetricRecorder::new(Average { sum: 0, count: 0 }))
        .add_system(MetricRecorder::new(Maximum { max: 0 }))
        .add_system(MetricRecorder::new(Minimum { min: usize::MAX }))
        .run();

    for worker in workers {
        println!("{:?}", worker)
    }
}
