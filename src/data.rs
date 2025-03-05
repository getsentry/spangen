use std::collections::BTreeMap;
use std::time::Duration;

use fake::faker::filesystem::en::DirPath;
use fake::faker::lorem::en::Sentence;
use fake::faker::time::en::DateTimeBetween;
use fake::{Fake, Faker};
use rand::Rng;
use rand::rngs::ThreadRng;
use rand_distr::{Distribution, Normal};
use serde::Serialize;
use time::OffsetDateTime;

use crate::cli::{Config, MAX_PROJECTS};
use crate::types::{SpanId, TraceId};

pub struct RandomGenerator<'a> {
    config: &'a Config,
    rng: ThreadRng,
    segment_dist: Normal<f64>,
    span_dist: Normal<f64>,
    #[allow(dead_code, reason = "TODO: support custom receive time")]
    batch_delay_dist: Normal<f64>,
}

impl<'a> RandomGenerator<'a> {
    pub fn new(config: &'a Config) -> Self {
        let segment_dist = Normal::new(
            config.segments_per_trace as f64,
            config.segments_per_trace_stddev,
        )
        .unwrap();

        let span_dist = Normal::new(
            config.spans_per_segment as f64,
            config.spans_per_segment_stddev,
        )
        .unwrap();

        let batch_delay_dist = Normal::new(
            config.batch_delay_ms as f64,
            config.batch_delay_stddev as f64,
        )
        .unwrap();

        Self {
            config,
            rng: rand::rng(),
            segment_dist,
            span_dist,
            batch_delay_dist,
        }
    }

    pub fn rng(&mut self) -> &mut ThreadRng {
        &mut self.rng
    }

    pub fn organization_id(&mut self) -> u64 {
        self.rng.random_range(1..self.config.number_of_orgs + 1)
    }

    pub fn project_id(&mut self, organization_id: u64) -> u64 {
        self.rng.random_range(1..self.config.number_of_projects + 1)
            + organization_id * MAX_PROJECTS
    }

    pub fn segment_count(&mut self) -> usize {
        self.segment_dist.sample(&mut self.rng).round().max(1.0) as usize
    }

    pub fn span_count(&mut self) -> usize {
        self.span_dist.sample(&mut self.rng).round().max(1.0) as usize
    }

    pub fn trace(&mut self) -> TraceInfo {
        TraceInfo::new(self.organization_id())
    }

    pub fn segment<'b>(&mut self, trace: &'b TraceInfo) -> SegmentInfo<'b> {
        SegmentInfo::new(trace, self.project_id(trace.organization_id))
    }

    /// Builds a randomized span tree with defined number of spans and depth.
    ///
    /// The tree is returned serialized in post-order: children first and then their parents.
    pub fn span_refs(&mut self) -> Vec<SpanRef> {
        let len = self.span_count();
        let depth = self.config.tree_depth;

        let mut levels = Vec::with_capacity(len);
        let mut spans = Vec::with_capacity(len);

        levels.push(0);
        spans.push(SpanRef {
            span_id: SpanId::default(),
            parent_id: None,
        });

        while spans.len() < len {
            let mut index = rand::random_range(0..spans.len());

            while levels[index] >= depth {
                index -= 1;
            }

            levels.push(levels[index] + 1);
            spans.push(SpanRef {
                span_id: SpanId::default(),
                parent_id: Some(spans[index].span_id),
            });
        }

        spans.reverse();
        spans
    }

    pub fn span<'b>(&mut self, segment: &'b SegmentInfo<'b>, span_ref: SpanRef) -> Span {
        let now = OffsetDateTime::now_utc();
        let lower = now - Duration::from_secs(60 * 60);

        let end_timestamp = DateTimeBetween(lower, now).fake();
        let duration_ms: u32 = (1..2000).fake();
        let start_timestamp = end_timestamp - Duration::from_millis(duration_ms.into());

        Span {
            trace_id: segment.trace.trace_id,
            span_id: span_ref.span_id,
            parent_span_id: span_ref.parent_id,
            is_remote: false,
            organization_id: segment.trace.organization_id,
            project_id: segment.project_id,

            description: Sentence(3..6).fake(),
            origin: DirPath().fake(),
            data: Faker.fake(),
            received: to_float(now),
            start_timestamp_precise: to_float(start_timestamp),
            end_timestamp_precise: to_float(end_timestamp),
            start_timestamp_ms: start_timestamp.unix_timestamp() as u64 * 1000,
            duration_ms,
            platform: "other",
            retention_days: 30,
        }
    }
}

pub struct TraceInfo {
    pub trace_id: TraceId,
    pub organization_id: u64,
}

impl TraceInfo {
    pub fn new(organization_id: u64) -> Self {
        Self {
            trace_id: TraceId::default(),
            organization_id,
        }
    }
}

pub struct SegmentInfo<'a> {
    pub trace: &'a TraceInfo,
    pub project_id: u64,
}

impl<'a> SegmentInfo<'a> {
    pub fn new(trace: &'a TraceInfo, project_id: u64) -> Self {
        Self { trace, project_id }
    }
}

/// A span id and its optional parent id.
#[derive(Debug, Clone, Copy)]
pub struct SpanRef {
    pub span_id: SpanId,
    pub parent_id: Option<SpanId>,
}

/// A complete span populated with fake data.
#[derive(Debug, Serialize)]
pub struct Span {
    pub trace_id: TraceId,
    pub span_id: SpanId,
    pub parent_span_id: Option<SpanId>,
    pub is_remote: bool,
    pub organization_id: u64,
    pub project_id: u64,
    pub description: String,
    pub origin: String,
    pub data: BTreeMap<String, String>,
    pub received: f64,
    pub start_timestamp_precise: f64,
    pub end_timestamp_precise: f64,
    pub start_timestamp_ms: u64,
    pub duration_ms: u32,
    pub platform: &'static str,
    pub retention_days: u16,
}

fn to_float(date_time: OffsetDateTime) -> f64 {
    let timestamp = date_time.unix_timestamp() as f64;
    let subsecond = date_time.nanosecond() as f64 / 1_000_000_000.0;
    timestamp + subsecond
}
