use serde::{de::DeserializeOwned, Deserialize};
use serde_json::Value;

#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct OnebotActionResponse<R> {
    pub status: String,
    pub retcode: i32,
    pub data: R,
    pub message: String,
    pub echo: Option<String>,
}
impl OnebotActionResponse<Value> {
    pub fn into_response<R: DeserializeOwned>(self) -> OnebotActionResponse<R> {
        OnebotActionResponse {
            status: self.status,
            retcode: self.retcode,
            data: serde_json::from_value(self.data).unwrap(),
            message: self.message,
            echo: self.echo,
        }
    }
}
