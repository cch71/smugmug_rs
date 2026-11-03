/*
 * Copyright (c) 2025 Craig Hamilton and Contributors.
 * Licensed under either of
 *  - Apache License, Version 2.0 <http://www.apache.org/licenses/LICENSE-2.0> OR
 *  - MIT license <http://opensource.org/licenses/MIT>
 *  at your option.
 */
use crate::v2::{NodeType, PrivacyLevel};
use chrono::{DateTime, Utc};
use serde::Deserialize;
use serde_json::Value;
use std::str::FromStr;

// Parses node type
pub fn from_node_type<'de, D>(deserializer: D) -> Result<NodeType, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let s: String = Deserialize::deserialize(deserializer)?;
    NodeType::from_str(&s).or(Ok(NodeType::Unknown))
}

// Parses privacy type
pub fn from_privacy<'de, D>(deserializer: D) -> Result<Option<PrivacyLevel>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let s: String = Deserialize::deserialize(deserializer)?;
    Ok(PrivacyLevel::from_str(&s)
        .ok()
        .or(Some(PrivacyLevel::Unknown)))
}

// Skips serialization if is none or is some but empty string
pub fn is_none_or_empty_str(tst: &Option<String>) -> bool {
    tst.as_ref().filter(|v| !(*v).is_empty()).is_none()
}

// The metadata endpoints return the same field as a string, a number or an empty string
// depending on the image, so these parsers accept any of those forms and treat empty or
// unparseable values as missing instead of failing the whole response.

// Parses a string or number as an optional string, empty strings become None
pub fn from_lenient_string<'de, D>(deserializer: D) -> Result<Option<String>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    Ok(match Option::<Value>::deserialize(deserializer)? {
        Some(Value::String(s)) if !s.is_empty() => Some(s),
        Some(v @ (Value::Number(_) | Value::Bool(_))) => Some(v.to_string()),
        _ => None,
    })
}

// Parses a number or numeric string as an optional f64
pub fn from_lenient_f64<'de, D>(deserializer: D) -> Result<Option<f64>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    Ok(match Option::<Value>::deserialize(deserializer)? {
        Some(Value::Number(n)) => n.as_f64(),
        Some(Value::String(s)) => s.trim().parse().ok(),
        _ => None,
    })
}

// Parses a number or numeric string as an optional u64.  Whole floats such as `40.0` are
// accepted; negative or fractional values become None.
pub fn from_lenient_u64<'de, D>(deserializer: D) -> Result<Option<u64>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    Ok(match Option::<Value>::deserialize(deserializer)? {
        Some(Value::Number(n)) => n.as_u64().or_else(|| n.as_f64().and_then(whole_f64_to_u64)),
        Some(Value::String(s)) => {
            let s = s.trim();
            s.parse()
                .ok()
                .or_else(|| s.parse().ok().and_then(whole_f64_to_u64))
        }
        _ => None,
    })
}

// Converts a whole, non-negative f64 to a u64
fn whole_f64_to_u64(v: f64) -> Option<u64> {
    (v >= 0.0 && v.fract() == 0.0 && v <= u64::MAX as f64).then_some(v as u64)
}

// Parses an RFC 3339 date/time string, empty or unparseable values become None
pub fn from_lenient_datetime<'de, D>(deserializer: D) -> Result<Option<DateTime<Utc>>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    Ok(match Option::<Value>::deserialize(deserializer)? {
        Some(Value::String(s)) => DateTime::parse_from_rfc3339(&s)
            .ok()
            .map(|v| v.with_timezone(&Utc)),
        _ => None,
    })
}
