use crate::types::segment::MessageSegments;
use serde::{Deserialize, Serialize};
use serde_json::Value;
/// Base event struct;
#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct EventBase {
    /// The unique identifier of the event.
    pub id: String,

    /// Unix timestamp of the event.
    pub time: f64,

    #[serde(rename = "type")]
    /// The type of event.  `meta`|`message`|`request`|`notice`.
    pub event_type: String,

    /// The detail type of event.
    pub detail_type: String,

    /// The sub type of the event.
    pub sub_type: Option<String>,

    #[serde(rename = "self")]
    /// Bot self identification.
    pub selft: Option<Selft>,

    /// extra
    #[serde(flatten)]
    pub extra: Option<Value>,
}
#[derive(Deserialize, Serialize, Clone, Debug, PartialEq, Hash, Eq)]
pub struct Selft {
    pub platform: String,
    pub user_id: String,
}
#[derive(Deserialize, Serialize, Clone, Debug)]
///
///  detail_type: `connect`
///
pub struct Connect {
    #[serde(flatten)]
    pub base: EventBase,

    /// The version of the Onebot implementation.
    ///
    /// TODO:sepcify the type of data.
    pub version: Value,
}

#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct Heartbeat {
    #[serde(flatten)]
    pub base: EventBase,

    /// The time interval of the heartbeat.
    pub interval: f64,
}

#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct StatusUpdate {
    #[serde(flatten)]
    pub base: EventBase,

    /// The status of the bot.
    ///
    /// TODO:sepcify the type of data.
    pub status: Value,
}

#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct PrivateMessage {
    #[serde(flatten)]
    pub base: EventBase,

    /// The message id.
    pub message_id: String,

    pub message: MessageSegments,

    pub user_id: String,

    pub alt_message: String,
}

#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct FriendIncrease {
    #[serde(flatten)]
    pub base: EventBase,

    pub user_id: String,
}

#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct FriendDecrease {
    #[serde(flatten)]
    pub base: EventBase,

    pub user_id: String,
}

#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct PrivateMessageDelete {
    #[serde(flatten)]
    pub base: EventBase,

    pub message_id: String,

    pub user_id: String,
}

#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct GroupMessage {
    #[serde(flatten)]
    pub base: EventBase,

    pub message_id: String,

    pub message: MessageSegments,

    pub alt_message: String,

    pub group_id: String,

    pub user_id: String,
}

#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct GroupMemberIncrease {
    #[serde(flatten)]
    pub base: EventBase,

    pub group_id: String,

    pub user_id: String,

    pub operator_id: String,
}

#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct GroupMemberDecrease {
    #[serde(flatten)]
    pub base: EventBase,

    pub group_id: String,

    pub user_id: String,

    pub operator_id: String,
}

#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct GroupMessageDelete {
    #[serde(flatten)]
    pub base: EventBase,

    pub message_id: String,

    pub group_id: String,

    pub user_id: String,
}
#[derive(Debug, Serialize, Clone)]
#[serde(untagged)]
pub enum Event {
    Connect(Connect),
    Heartbeat(Heartbeat),
    StatusUpdate(StatusUpdate),
    PrivateMessage(PrivateMessage),
    FriendIncrease(FriendIncrease),
    FriendDecrease(FriendDecrease),
    PrivateMessageDelete(PrivateMessageDelete),
    GroupMessage(GroupMessage),
    GroupMemberIncrease(GroupMemberIncrease),
    GroupMemberDecrease(GroupMemberDecrease),
    GroupMessageDelete(GroupMessageDelete),
    Extra(Value),
}

impl From<Value> for Event {
    fn from(value: Value) -> Self {
        let detail_type = value["detail_type"].as_str().unwrap_or("");
        match detail_type {
            "connect" => Event::Connect(serde_json::from_value(value).unwrap()),
            "heartbeat" => Event::Heartbeat(serde_json::from_value(value).unwrap()),
            "status_update" => Event::StatusUpdate(serde_json::from_value(value).unwrap()),
            "private" => Event::PrivateMessage(serde_json::from_value(value).unwrap()),
            "friend_increase" => Event::FriendIncrease(serde_json::from_value(value).unwrap()),
            "friend_decrease" => Event::FriendDecrease(serde_json::from_value(value).unwrap()),
            "private_message_delete" => {
                Event::PrivateMessageDelete(serde_json::from_value(value).unwrap())
            }
            "group" => Event::GroupMessage(serde_json::from_value(value).unwrap()),
            "group_member_increase" => {
                Event::GroupMemberIncrease(serde_json::from_value(value).unwrap())
            }
            "group_member_decrease" => {
                Event::GroupMemberDecrease(serde_json::from_value(value).unwrap())
            }
            "group_message_delete" => {
                Event::GroupMessageDelete(serde_json::from_value(value).unwrap())
            }
            _ => Event::Extra(value),
        }
    }
}
impl<'de> serde::Deserialize<'de> for Event {
    fn deserialize<D>(deserializer: D) -> Result<Event, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let value = Value::deserialize(deserializer)?;
        Ok(Event::from(value))
    }
}
