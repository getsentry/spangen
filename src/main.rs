use std::io::{StdoutLock, Write};

use argh::FromArgs;
use rand::Rng;
use rand_distr::{Distribution, Normal};

use crate::data::{FakeSpan, SpanInfo};
use crate::types::{SpanId, TraceId};

mod data;
mod types;

const MAX_PROJECTS: u64 = 1000;

/// A load generator for spans in traces.
#[derive(Debug, FromArgs)]
pub struct Config {
    /// the number of spans to generate in total.
    #[argh(option)]
    pub count: usize,

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

    /// the maximum number of spans that will be generated in a single run.
    ///
    /// This is used to simulate a stream of spans that are generated in batches. SDKs do not
    /// typically generate spans one by one, but rather in batches. This parameter controls the
    /// maximum number of spans that will be generated in a single run.
    #[argh(option, default = "100")]
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
    pub concurrent_traces: usize,

    /// the size of the payload in bytes.
    #[argh(option, default = "14400")]
    pub payload_size: usize,

    /// the depth of the span tree within each segment.
    #[argh(option, default = "3")]
    pub tree_depth: usize,

    /// the percentage of segments without an explicit root span (0..100)
    #[argh(option, default = "0")]
    pub segments_without_root: u16,

    /// the number of organizations.
    #[argh(option, default = "1000")]
    pub number_of_orgs: u64,

    /// the number of projects per organization.
    #[argh(option, default = "10")]
    pub number_of_projects: u64,
}

impl Config {
    pub fn validate(&mut self) -> Result<(), &'static str> {
        if self.number_of_orgs == 0 {
            log::error!("invalid number of orgs, using default value of 1");
            self.number_of_orgs = 1;
        }

        if self.number_of_projects == 0 {
            log::error!("invalid number of projects, using default value of 1");
            self.number_of_projects = 1;
        }

        if self.number_of_projects >= MAX_PROJECTS {
            log::error!(
                "number of projects too high, using maximum value of {}",
                MAX_PROJECTS - 1
            );
            self.number_of_projects = MAX_PROJECTS - 1;
        }

        if self.tree_depth == 0 {
            log::error!("invalid tree depth, using default value of 1");
            self.tree_depth = 1;
        }

        if self.segments_without_root > 100 {
            return Err("segments-without-root must be between 0 and 100");
        }

        Ok(())
    }

    pub fn segments_per_trace_dist(&self) -> Normal<f64> {
        rand_distr::Normal::new(
            self.segments_per_trace as f64,
            self.segments_per_trace_stddev,
        )
        .unwrap()
    }

    pub fn spans_per_segment_dist(&self) -> Normal<f64> {
        rand_distr::Normal::new(self.spans_per_segment as f64, self.spans_per_segment_stddev)
            .unwrap()
    }

    pub fn batch_delay_dist(&self) -> Normal<f64> {
        rand_distr::Normal::new(self.batch_delay_ms as f64, self.batch_delay_stddev as f64).unwrap()
    }
}

#[derive(Debug)]
struct Trace {
    trace_id: TraceId,
    organization_id: u64,
}

impl Trace {
    pub fn new(organization_id: u64) -> Self {
        Self {
            trace_id: TraceId::new(),
            organization_id,
        }
    }

    pub fn create_segment(&mut self, project_id: u64) -> Segment {
        // TODO: implement propagated segments
        Segment {
            trace_id: self.trace_id,
            organization_id: self.organization_id,
            project_id,
            root_span_id: None,
            span_ids: Vec::new(),
        }
    }
}

#[derive(Debug)]
struct Segment {
    trace_id: TraceId,
    organization_id: u64,
    project_id: u64,
    root_span_id: Option<SpanId>,
    span_ids: Vec<SpanId>,
}

impl Segment {
    pub fn create_span(&mut self) -> Span {
        let info = SpanInfo {
            trace_id: self.trace_id,
            span_id: SpanId::new(),
            // TODO: implement nesting
            parent_span_id: self.root_span_id,
            // TODO: implement remote spans
            is_remote: false,
            organization_id: self.organization_id,
            project_id: self.project_id,
        };

        self.root_span_id.get_or_insert(info.span_id);
        self.span_ids.push(info.span_id);

        Span { info }
    }
}

struct Span {
    info: SpanInfo,
}

impl Span {
    pub fn fake(&self) -> FakeSpan<'_> {
        FakeSpan::new(&self.info)
    }
}

struct BoundedWriter {
    stdout: StdoutLock<'static>,
    remaining: usize,
}

impl BoundedWriter {
    pub fn new(count: usize) -> Self {
        Self {
            stdout: std::io::stdout().lock(),
            remaining: count,
        }
    }

    pub fn write_span(&mut self, span: &Span) -> bool {
        serde_json::to_writer(&mut self.stdout, &span.fake()).ok();
        writeln!(&mut self.stdout).ok();

        self.remaining -= 1;
        self.remaining > 0
    }
}

fn produce(config: &Config) {
    let mut rng = rand::rng();
    let trace_dist = config.segments_per_trace_dist();
    let segment_dist = config.spans_per_segment_dist();

    let mut writer = BoundedWriter::new(config.count);

    loop {
        let organization_id = rng.random_range(1..config.number_of_orgs + 1);
        let mut trace = Trace::new(organization_id);
        let num_segments = trace_dist.sample(&mut rng).round().max(1.0) as usize;
        log::trace!(
            "creating trace {} with {} segments in org {}",
            trace.trace_id,
            num_segments,
            organization_id,
        );

        for _ in 0..num_segments {
            let project_id =
                rng.random_range(1..config.number_of_projects + 1) + organization_id * MAX_PROJECTS;
            let mut segment = trace.create_segment(project_id);
            let num_spans = segment_dist.sample(&mut rng).round().max(1.0) as usize;
            log::trace!(
                "creating segment {} with {} spans in project {}",
                segment.trace_id,
                num_spans,
                project_id,
            );

            for _ in 0..num_spans {
                let span = segment.create_span();
                if !writer.write_span(&span) {
                    return;
                }
            }
        }
    }
}

fn main() -> Result<(), &'static str> {
    pretty_env_logger::init();

    let mut config: Config = argh::from_env();
    config.validate()?;

    // TODO: concurrent producers
    // let mut producers = Vec::new();
    // for _ in 0..config.concurrent_traces {
    //     producers.push(Producer::new(&config));
    // }

    produce(&config);

    Ok(())
}
