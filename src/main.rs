use std::io::{StdoutLock, Write};
use std::thread;
use std::time::{Duration, Instant};

use anyhow::Result;
use rand::Rng;
use rand::seq::IndexedRandom;
use serde::Serialize;

use crate::cli::Config;
use crate::data::RandomGenerator;

mod cli;
mod data;
mod types;

const MIN_SLEEP: Duration = Duration::from_millis(1);

struct Throttle {
    throughput: Option<u32>,
    accepted: u32,
    last_sleep: Instant,
}

impl Throttle {
    pub fn new(throughput: Option<u32>) -> Self {
        Self {
            throughput,
            accepted: 0,
            last_sleep: Instant::now(),
        }
    }

    pub fn accept(&mut self) {
        if self.throughput.is_some() {
            self.accepted += 1;
        }
    }

    pub fn wait(&mut self) {
        let Some(throughput) = self.throughput else {
            return;
        };

        let now = Instant::now();
        let elapsed = now - self.last_sleep;

        let expected_duration = (Duration::from_secs(1) * self.accepted) / throughput;
        let sleep_duration = expected_duration.saturating_sub(elapsed);
        if sleep_duration >= MIN_SLEEP {
            thread::sleep(sleep_duration);
            self.last_sleep = now + sleep_duration;
            self.accepted = 0;
        }
    }
}

struct StdoutProducer {
    stdout: StdoutLock<'static>,
}

impl StdoutProducer {
    pub fn new() -> Self {
        Self {
            stdout: std::io::stdout().lock(),
        }
    }

    pub fn produce_json<T: Serialize>(&mut self, value: &T) -> Result<()> {
        serde_json::to_writer(&mut self.stdout, value)?;
        writeln!(&mut self.stdout)?;
        Ok(())
    }
}

fn produce(config: &Config) -> Result<()> {
    let start = Instant::now();
    let mut generator = RandomGenerator::new(config);
    let mut producer = StdoutProducer::new();
    let mut throttle = Throttle::new(config.throughput);

    while generator.stats().spans < config.count {
        let trace = generator.trace();
        let mut remote_parent = None;

        for _ in 0..generator.segment_count() {
            let segment = generator.segment(&trace);
            let span_refs = generator.span_refs(&segment);

            throttle.wait();

            for span_ref in &span_refs {
                let mut span = generator.span(&segment, *span_ref);
                if span_ref.parent_id.is_none() {
                    debug_assert!(span_ref.span_id == segment.span_id);
                    span.parent_span_id = remote_parent;
                    span.is_remote = remote_parent.is_some();
                }

                producer.produce_json(&span)?;
                throttle.accept();
            }

            // in 50% of the cases, pick a random span as the remote parent for the next segment
            if generator.rng().random_ratio(1, 2) {
                remote_parent = span_refs.choose(generator.rng()).map(|sr| sr.span_id);
            }
        }
    }

    let stats = generator.stats();
    log::info!("Finished in {:?}", start.elapsed());
    log::info!("  traces:   {}", stats.traces);
    log::info!("  segments: {}", stats.segments);
    log::info!("  spans:    {}", stats.spans);

    Ok(())
}

fn main() -> Result<()> {
    pretty_env_logger::init();

    let mut config: Config = argh::from_env();
    config.validate()?;

    produce(&config)
}
