use std::fmt::Debug;
use std::pin::Pin;
use std::task::{Context, Poll};

use crate::payload::Payload;
use crate::types::{action::OnebotAction, response::OnebotActionResponse, Event};
use futures::Stream;
use serde::de::DeserializeOwned;

pub trait OneBotConnection
{
    type Error;
    type StreamOutput<E>: Stream<Item =E>;
    async fn send<A>(
        &mut self,
        action: OnebotAction<A>,
    ) -> Result<OnebotActionResponse<A::Output>, Self::Error>
    where
        A: 'static,
        A: Payload;
    async fn receive<E>(&mut self) -> Self::StreamOutput<E>
    where
        E: 'static+Send+Sync,
        E: DeserializeOwned,
        E: Debug;
}
pub struct EventStream<E = Event> {
    receiver: tokio::sync::mpsc::UnboundedReceiver<E>,
}

impl<E> EventStream<E> {
    pub fn new(receiver: tokio::sync::mpsc::UnboundedReceiver<E>) -> Self {
        Self { receiver }
    }
}

impl<E> Stream for EventStream<E> {
    type Item = E;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        match self.receiver.poll_recv(cx) {
            Poll::Ready(Some(e)) => Poll::Ready(Some(e)),
            Poll::Ready(None) => Poll::Ready(None),
            Poll::Pending => Poll::Pending,
        }
    }
}
