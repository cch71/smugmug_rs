/*
 * Copyright (c) 2025 Craig Hamilton and Contributors.
 * Licensed under either of
 *  - Apache License, Version 2.0 <http://www.apache.org/licenses/LICENSE-2.0> OR
 *  - MIT license <http://opensource.org/licenses/MIT>
 *  at your option.
 */
use crate::v2::{NodeType, PrivacyLevel};
use serde::Deserialize;
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
    Ok(PrivacyLevel::from_str(&s).ok().or(Some(PrivacyLevel::Unknown)))
}

// Parses strings that may be "" and sets to None
pub fn from_empty_str_to_none
<'de, D>(deserializer: D) -> Result<Option<String>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let s: String = Deserialize::deserialize(deserializer)?;
    if s.is_empty() {
        Ok(None)
    } else {
        Ok(Some(s))
    }
}
