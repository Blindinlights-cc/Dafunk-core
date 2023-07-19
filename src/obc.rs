use crate::types::{action::Action, Selft};
use futures::Stream;
use serde::de::DeserializeOwned;
pub trait OneBotConn {
    type Error;
    type EventStream<E>: Stream<Item = Result<E, Self::Error>>;
    async fn event_stream<E: DeserializeOwned>(&self) -> Self::EventStream<E>;
    async fn request<A: Action>(
        &self,
        self_type: Selft,
        action: A,
    ) -> Result<A::ActionResponse, Self::Error>;
}
