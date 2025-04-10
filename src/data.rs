use std::net::Ipv4Addr;
use std::time::Duration;

use fake::Fake;
use fake::faker::filesystem::en::DirPath;
use fake::faker::internet::en::{FreeEmail, IPv4};
use fake::faker::lorem::en::Sentence;
use fake::faker::time::en::DateTimeBetween;
use rand::Rng;
use rand::rngs::ThreadRng;
use rand::seq::{IndexedRandom, SliceRandom};
use rand_distr::{Distribution, Normal};
use serde::Serialize;
use time::OffsetDateTime;

use crate::cli::{Config, MAX_PROJECTS, SpanOrder};
use crate::constants::{
    BROWSER_NAMES, HTTP_METHODS, ROOT_OPS, SENTRY_ENVIRONMENTS, SENTRY_PLATFORMS, SENTRY_RELEASES,
    SENTRY_SDKS, SENTRY_TRANSACTIONS, SPAN_OPS, THREAD_NAMES,
};
use crate::types::{SpanId, TraceId};

#[derive(Clone, Debug, Default)]
pub struct Stats {
    pub spans: usize,
    pub segments: usize,
    pub traces: usize,
}

pub struct RandomGenerator<'a> {
    config: &'a Config,
    rng: ThreadRng,
    segment_dist: Normal<f64>,
    span_dist: Normal<f64>,
    #[allow(dead_code, reason = "TODO: support custom receive time")]
    batch_delay_dist: Normal<f64>,
    stats: Stats,
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
            stats: Stats::default(),
        }
    }

    pub fn stats(&self) -> &Stats {
        &self.stats
    }

    pub fn rng(&mut self) -> &mut ThreadRng {
        &mut self.rng
    }

    pub fn organization_id(&mut self) -> u64 {
        self.rng.random_range(1..self.config.orgs + 1)
    }

    pub fn project_id(&mut self, organization_id: u64) -> u64 {
        self.rng.random_range(1..self.config.projects + 1) + (organization_id - 1) * MAX_PROJECTS
    }

    pub fn segment_count(&mut self) -> usize {
        self.segment_dist.sample(&mut self.rng).round().max(1.0) as usize
    }

    pub fn span_count(&mut self) -> usize {
        self.span_dist.sample(&mut self.rng).round().max(1.0) as usize
    }

    pub fn trace(&mut self) -> TraceInfo {
        self.stats.traces += 1;
        TraceInfo::new(self.organization_id())
    }

    pub fn sentry_tags(&mut self) -> SentryTags {
        let user_id = self.rng.random_range(1..100_000);
        let user_email: String = FreeEmail().fake();

        SentryTags {
            release: SENTRY_RELEASES.choose(self.rng()).unwrap(),
            user: user_id,
            user_id,
            user_ip: IPv4().fake(),
            user_username: user_email.clone(),
            user_email,
            environment: SENTRY_ENVIRONMENTS.choose(self.rng()).unwrap(),
            op: SPAN_OPS.choose(self.rng()).unwrap(),
            transaction: SENTRY_TRANSACTIONS.choose(self.rng()).unwrap(),
            transaction_method: HTTP_METHODS.choose(self.rng()).unwrap(),
            transaction_op: ROOT_OPS.choose(self.rng()).unwrap(),
            browser_name: BROWSER_NAMES.choose(self.rng()).unwrap(),
            sdk_name: SENTRY_SDKS.choose(self.rng()).unwrap(),
            sdk_version: (
                self.rng.random_range(0..3),
                self.rng.random_range(0..10),
                self.rng.random_range(0..10),
            ),
            platform: SENTRY_PLATFORMS.choose(self.rng()).unwrap(),
            thread_id: self.rng.random(),
            thread_name: THREAD_NAMES.choose(self.rng()).unwrap(),
        }
    }

    pub fn segment<'b>(&mut self, trace: &'b TraceInfo) -> SegmentInfo<'b> {
        self.stats.segments += 1;
        SegmentInfo::new(
            trace,
            self.project_id(trace.organization_id),
            self.sentry_tags(),
        )
    }

    /// Builds a randomized span tree with defined number of spans and depth.
    ///
    /// The tree is returned serialized in post-order: children first and then their parents.
    pub fn span_refs(&mut self, segment: &SegmentInfo<'_>) -> Vec<SpanRef> {
        let len = self.span_count();
        let depth = self.config.tree_depth;

        let mut levels = Vec::with_capacity(len);
        let mut spans = Vec::with_capacity(len);

        levels.push(0);
        spans.push(SpanRef {
            span_id: segment.span_id,
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

        match self.config.order {
            SpanOrder::Post => spans.reverse(),
            SpanOrder::Pre => (),
            SpanOrder::Random => spans.shuffle(&mut self.rng),
        }

        spans
    }

    pub fn span<'s>(&mut self, segment: &'s SegmentInfo<'_>, span_ref: SpanRef) -> Span<'s> {
        self.stats.spans += 1;

        let now = OffsetDateTime::now_utc();
        let lower = now - Duration::from_secs(60 * 60);

        let end_timestamp = DateTimeBetween(lower, now).fake();
        let duration_ms: u32 = (1..2000).fake();
        let start_timestamp = end_timestamp - Duration::from_millis(duration_ms.into());

        Span {
            trace_id: segment.trace.trace_id,
            span_id: span_ref.span_id,
            parent_span_id: span_ref.parent_id,
            segment_id: Some(segment.span_id),
            is_remote: false,
            organization_id: segment.trace.organization_id,
            project_id: segment.project_id,

            description: Sentence(3..6).fake(),
            origin: DirPath().fake(),
            sentry_tags: &segment.sentry_tags,
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
    pub span_id: SpanId,
    pub sentry_tags: SentryTags,
}

impl<'a> SegmentInfo<'a> {
    pub fn new(trace: &'a TraceInfo, project_id: u64, sentry_tags: SentryTags) -> Self {
        Self {
            trace,
            project_id,
            span_id: SpanId::default(),
            sentry_tags,
        }
    }
}

/// A span id and its optional parent id.
#[derive(Debug, Clone, Copy)]
pub struct SpanRef {
    pub span_id: SpanId,
    pub parent_id: Option<SpanId>,
}

#[derive(Debug, Serialize)]
pub struct SentryTags {
    pub release: &'static str,
    #[serde(serialize_with = "serialize_user")]
    pub user: u32,
    #[serde(rename = "user.id")]
    pub user_id: u32,
    #[serde(rename = "user.ip")]
    pub user_ip: Ipv4Addr,
    #[serde(rename = "user.username")]
    pub user_username: String,
    #[serde(rename = "user.email")]
    pub user_email: String,
    pub environment: &'static str,
    pub op: &'static str,
    pub transaction: &'static str,
    #[serde(rename = "transaction.method")]
    pub transaction_method: &'static str,
    #[serde(rename = "transaction.op")]
    pub transaction_op: &'static str,
    #[serde(rename = "browser.name")]
    pub browser_name: &'static str,
    #[serde(rename = "sdk.name")]
    pub sdk_name: &'static str,
    #[serde(rename = "sdk.version", serialize_with = "serialize_version")]
    pub sdk_version: (u8, u8, u8),
    pub platform: &'static str,
    #[serde(rename = "thread.id")]
    pub thread_id: u32,
    #[serde(rename = "thread.name")]
    pub thread_name: &'static str,
}

fn serialize_user<S: serde::Serializer>(user: &u32, serializer: S) -> Result<S::Ok, S::Error> {
    serializer.collect_str(&format_args!("id:{user}"))
}

fn serialize_version<S: serde::Serializer>(
    &(major, minor, patch): &(u8, u8, u8),
    serializer: S,
) -> Result<S::Ok, S::Error> {
    serializer.collect_str(&format_args!("{}.{}.{}", major, minor, patch))
}

/// A complete span populated with fake data.
#[derive(Debug, Serialize)]
pub struct Span<'a> {
    pub trace_id: TraceId,
    pub span_id: SpanId,
    pub parent_span_id: Option<SpanId>,
    pub segment_id: Option<SpanId>,
    pub is_remote: bool,
    pub organization_id: u64,
    pub project_id: u64,
    pub description: String,
    pub origin: String,
    pub sentry_tags: &'a SentryTags,
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
