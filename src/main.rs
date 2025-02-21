use std::io::{StdoutLock, Write};

use anyhow::Result;
use rand::Rng;
use rand::seq::IndexedRandom;
use serde::Serialize;

use crate::cli::Config;
use crate::data::RandomGenerator;

mod cli;
mod data;
mod types;

struct StdoutProducer {
    stdout: StdoutLock<'static>,
    count: usize,
}

impl StdoutProducer {
    pub fn new() -> Self {
        Self {
            stdout: std::io::stdout().lock(),
            count: 0,
        }
    }

    pub fn produce_json<T: Serialize>(&mut self, value: &T) -> Result<()> {
        serde_json::to_writer(&mut self.stdout, value)?;
        writeln!(&mut self.stdout)?;

        self.count += 1;
        Ok(())
    }

    pub fn count(&self) -> usize {
        self.count
    }
}

fn produce(config: &Config) -> Result<()> {
    let mut generator = RandomGenerator::new(config);
    let mut producer = StdoutProducer::new();

    while producer.count() < config.count {
        let trace = generator.trace();
        let mut remote_parent = None;

        for _ in 0..generator.segment_count() {
            let segment = generator.segment(&trace);
            let span_refs = generator.span_refs();

            for span_ref in &span_refs {
                let mut span = generator.span(&segment, *span_ref);
                if span_ref.parent_id.is_none() {
                    span.parent_span_id = remote_parent;
                    span.is_remote = remote_parent.is_some();
                }

                producer.produce_json(&span)?;
            }

            // in 50% of the cases, pick a random span as the remote parent for the next segment
            if generator.rng().random_ratio(1, 2) {
                remote_parent = span_refs.choose(generator.rng()).map(|sr| sr.span_id);
            }
        }
    }

    Ok(())
}

fn main() -> Result<()> {
    pretty_env_logger::init();

    let mut config: Config = argh::from_env();
    config.validate()?;

    produce(&config)
}
