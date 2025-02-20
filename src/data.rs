use std::collections::BTreeMap;
use std::time::Duration;

use fake::faker::filesystem::en::DirPath;
use fake::faker::lorem::en::Sentence;
use fake::faker::time::en::DateTimeBetween;
use fake::{Fake, Faker};
use serde::Serialize;
use time::OffsetDateTime;

use crate::types::{SpanId, TraceId};

/// Basic information to construct a span.
#[derive(Debug, Serialize)]
pub struct SpanInfo {
    pub trace_id: TraceId,
    pub span_id: SpanId,
    pub parent_span_id: Option<SpanId>,
    pub is_remote: bool,
    pub organization_id: u64,
    pub project_id: u64,
}

/// A complete span populated with fake data.
#[derive(Debug, Serialize)]
pub struct FakeSpan<'a> {
    #[serde(flatten)]
    info: &'a SpanInfo,

    description: String,
    origin: String,
    data: BTreeMap<String, String>,
    received: f64,
    start_timestamp_precise: f64,
    end_timestamp_precise: f64,
    duration_ms: u32,
    platform: &'static str,
    retention_days: u16,
}

impl<'a> FakeSpan<'a> {
    pub fn new(info: &'a SpanInfo) -> Self {
        // TODO: Support size in kB

        let now = OffsetDateTime::now_utc();
        let lower = now - Duration::from_secs(60 * 60);

        let end_timestamp = DateTimeBetween(lower, now).fake();
        let duration_ms: u32 = (1..2000).fake();
        let start_timestamp = end_timestamp - Duration::from_millis(duration_ms.into());

        Self {
            info,
            description: Sentence(3..6).fake(),
            origin: DirPath().fake(),
            data: Faker.fake(),
            // TODO: Support custom receive time
            received: to_float(now),
            start_timestamp_precise: to_float(start_timestamp),
            end_timestamp_precise: to_float(end_timestamp),
            duration_ms,
            platform: "other",
            retention_days: 30,
        }
    }
}

fn to_float(date_time: OffsetDateTime) -> f64 {
    let timestamp = date_time.unix_timestamp() as f64;
    let subsecond = date_time.nanosecond() as f64 / 1_000_000_000.0;
    timestamp + subsecond
}
