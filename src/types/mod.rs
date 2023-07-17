/// Onebot Action types
pub mod action;
/// Onebot Event types
pub mod event;

mod segment;
pub use event::*;
use serde::Deserialize;
pub use serde_json::Value;
/// Bot info
#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct BotsInfo {
    #[serde(rename = "self")]
    pub self_: Selft,
    pub online: bool,
    #[serde(flatten)]
    pub extra: Value,
}
