use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use serde_json::Value;
pub type MessageSegments = Vec<Segment>;

#[derive(Debug, PartialEq, Deserialize, Serialize, Clone)]
pub struct Segment {
    #[serde(rename = "type")]
    pub segment_type: SegmentType,
    pub data: HashMap<String, Value>,
}
#[derive(Debug, PartialEq, Deserialize, Serialize, Clone)]
#[serde(rename_all = "snake_case")]
pub enum SegmentType {
    Text,
    Image,
    Mention,
    MentionAll,
    Voice,
    Audio,
    Video,
    File,
    Location,
    Reply,
    #[serde(other)]
    Unknown,
}
