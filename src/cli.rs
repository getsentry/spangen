use std::str::FromStr;

use anyhow::Result;
use argh::FromArgs;

pub const MAX_PROJECTS: u64 = 1000;

/// A load generator for spans in traces.
#[derive(Debug, FromArgs)]
pub struct Config {
    /// the number of spans to generate in total.
    ///
    /// spangen will stop generating new traces after this number has been reached, but it will
    /// finish started traces and segments. The actual number of spans generated may therefore be
    /// higher than this option.
    #[argh(option)]
    pub count: usize,

    /// the throughput of spans per second (defaults to no throttling).
    #[argh(option)]
    pub throughput: Option<u32>,

    /// the average number of spans per segment (randomized).
    #[argh(option, default = "17")]
    pub spans_per_segment: usize,

    /// the standard deviation for randomizing the number of spans per segment.
    #[argh(option, default = "17.0")]
    pub spans_per_segment_stddev: f64,

    /// the average number of segments per trace (randomized).
    #[argh(option, default = "1")]
    pub segments_per_trace: usize,

    /// the standard deviation for randomizing the number of segments per trace.
    #[argh(option, default = "1.0")]
    pub segments_per_trace_stddev: f64,

    /// the order in which spans are written in a segment.
    #[argh(option, default = "SpanOrder::Post")]
    pub order: SpanOrder,

    /// the maximum number of spans that will be generated in a single run.
    ///
    /// This is used to simulate a stream of spans that are generated in batches. SDKs do not
    /// typically generate spans one by one, but rather in batches. This parameter controls the
    /// maximum number of spans that will be generated in a single run.
    #[argh(option, default = "100")]
    #[allow(
        dead_code,
        reason = "TODO: Support batched emission with delays and receive time"
    )]
    pub batch_size: usize,

    /// the delay in milliseconds between consecutive batches of a segment.
    ///
    /// This is used to simulate an operating SDK that collects spans over time and flushes them in
    /// batches with a given delay.
    #[argh(option, default = "2000")]
    pub batch_delay_ms: u64,

    /// the standard deviation for the batch delay in milliseconds.
    #[argh(option, default = "500")]
    pub batch_delay_stddev: u64,

    /// the number of concurrent traces that interleave on the stream.
    #[argh(option, default = "1000")]
    #[allow(dead_code, reason = "TODO: Implement concurrent traces")]
    pub concurrent_traces: usize,

    /// the size of the payload in bytes.
    #[argh(option, default = "14400")]
    #[allow(dead_code, reason = "TODO: Support configuring data size")]
    pub payload_size: usize,

    /// the depth of the span tree within each segment.
    #[argh(option, default = "3")]
    pub tree_depth: usize,

    /// the percentage of segments without an explicit root span (0..100)
    #[argh(option, default = "0")]
    #[allow(dead_code, reason = "TODO: Support configuring data size")]
    pub segments_without_root: u16,

    /// the number of organizations.
    #[argh(option, default = "1000")]
    pub orgs: u64,

    /// the number of projects per organization.
    #[argh(option, default = "10")]
    pub projects: u64,
}

impl Config {
    pub fn validate(&mut self) -> Result<()> {
        if self.orgs == 0 {
            log::error!("invalid number of orgs, using default value of 1");
            self.orgs = 1;
        }

        if self.projects == 0 {
            log::error!("invalid number of projects, using default value of 1");
            self.projects = 1;
        }

        if self.projects >= MAX_PROJECTS {
            log::error!(
                "number of projects too high, using maximum value of {}",
                MAX_PROJECTS - 1
            );
            self.projects = MAX_PROJECTS - 1;
        }

        if self.tree_depth == 0 {
            log::error!("invalid tree depth, using default value of 1");
            self.tree_depth = 1;
        }

        if self.segments_without_root > 100 {
            anyhow::bail!("segments-without-root must be between 0 and 100");
        }

        Ok(())
    }
}

/// The order in which spans are written in a segment.
#[derive(Clone, Copy, Debug)]
pub enum SpanOrder {
    /// Parents are written after their children.
    Post,
    /// Parents are written before their children.
    Pre,
    /// Spans are written in a random order.
    Random,
}

impl FromStr for SpanOrder {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self> {
        match s.to_lowercase().as_str() {
            "post" => Ok(SpanOrder::Post),
            "pre" => Ok(SpanOrder::Pre),
            "random" => Ok(SpanOrder::Random),
            _ => anyhow::bail!("invalid span order: {}", s),
        }
    }
}
