use std::{fmt, str};

use serde::{Deserialize, Serialize};

/// Holds the identifier for a Span
#[derive(Serialize, Deserialize, Debug, Copy, Clone, Eq, PartialEq, Hash)]
#[serde(try_from = "String", into = "String")]
pub struct SpanId([u8; 8]);

impl Default for SpanId {
    fn default() -> Self {
        Self(rand::random())
    }
}

impl fmt::Display for SpanId {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        write!(fmt, "{}", hex::encode(self.0))
    }
}

impl From<SpanId> for String {
    fn from(span_id: SpanId) -> Self {
        span_id.to_string()
    }
}

impl str::FromStr for SpanId {
    type Err = hex::FromHexError;

    fn from_str(input: &str) -> Result<Self, Self::Err> {
        let mut buf = [0; 8];
        hex::decode_to_slice(input, &mut buf)?;
        Ok(Self(buf))
    }
}

impl TryFrom<String> for SpanId {
    type Error = hex::FromHexError;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        value.parse()
    }
}

/// Holds the identifier for a Trace
#[derive(Serialize, Deserialize, Debug, Copy, Clone, Eq, PartialEq, Hash)]
#[serde(try_from = "String", into = "String")]
pub struct TraceId([u8; 16]);

impl Default for TraceId {
    fn default() -> Self {
        Self(rand::random())
    }
}

impl fmt::Display for TraceId {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        write!(fmt, "{}", hex::encode(self.0))
    }
}

impl From<TraceId> for String {
    fn from(trace_id: TraceId) -> Self {
        trace_id.to_string()
    }
}

impl str::FromStr for TraceId {
    type Err = hex::FromHexError;

    fn from_str(input: &str) -> Result<Self, Self::Err> {
        let mut buf = [0; 16];
        hex::decode_to_slice(input, &mut buf)?;
        Ok(Self(buf))
    }
}

impl TryFrom<String> for TraceId {
    type Error = hex::FromHexError;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        value.parse()
    }
}
