pub mod action;
pub mod event;
pub mod segment;
pub use event::*;
use serde::Deserialize;
pub use serde_json::Value;
#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct BotsInfo {
    #[serde(rename = "self")]
    pub self_:Selft,
    pub online:bool,
    #[serde(flatten)]
    pub extra:Value,
}
