use futures::Stream;
use serde::de::DeserializeOwned;
use crate::types::action::Action;
pub trait OneBotConn {
    type Error;
    type EventStream<E>: Stream<Item = Result<E, Self::Error>>;
    async fn event_stream<E: DeserializeOwned>(&self) -> Self::EventStream<E>;
    async fn request<A: Action>(&self, action: A) -> Result<A::ActionResponse, Self::Error>;
}

