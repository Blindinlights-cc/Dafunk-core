use serde::Serialize;
use serde::de::DeserializeOwned;
pub trait Payload: Serialize {
    const NAME: &'static str;
    type Output: DeserializeOwned+Clone;
}