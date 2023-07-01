use std::fmt::Debug;
use std::pin::Pin;
use std::sync::Arc;
use std::task::{Context, Poll};
use crate::payload::Payload;
use crate::types::{action::OnebotAction, response::OnebotActionResponse};
use futures::Stream;
use serde::de::DeserializeOwned;

pub trait OneBotConnection
{
    type Error;
    type StreamOutput<E>: Stream<Item =Result<E,Self::Error>>;

    
    async fn send<A>(
        self:Arc<Self>,
        action: OnebotAction<A>,
    ) -> Result<OnebotActionResponse<A::Output>, Self::Error>
    where
        A: 'static,
        A: Payload;
    async fn receive<E>(self:Arc<Self>,) -> Self::StreamOutput<E>
    where
        E: 'static+Send+Sync,
        E: DeserializeOwned,
        E: Debug;
}
pub struct EventStream<E,Err> {
    receiver: tokio::sync::mpsc::UnboundedReceiver<Result<E,Err>>,
}

impl<E,Err> EventStream<E,Err> {
    pub fn new(receiver: tokio::sync::mpsc::UnboundedReceiver<Result<E,Err>>) -> Self {
        Self { receiver }
    }
}

impl<E,Err> Stream for EventStream<E,Err> {
    type Item = Result<E,Err>;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        match self.receiver.poll_recv(cx) {
            Poll::Ready(Some(event)) => Poll::Ready(Some(event)),
            Poll::Ready(None) => Poll::Ready(None),
            Poll::Pending => Poll::Pending,
        }
    }
}
