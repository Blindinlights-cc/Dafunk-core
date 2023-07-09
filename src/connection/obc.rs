use crate::payload::Payload;
use crate::types::Event;
use crate::types::{action::OnebotAction, response::OnebotActionResponse};
use futures::Stream;
use serde::de::DeserializeOwned;
use std::fmt::Debug;
use std::pin::Pin;
use std::sync::Arc;
use std::task::{Context, Poll};
/// A connection to a OneBot implementation.
/// 
/// This trait is implemented by all connections to `OneBot` implementations
/// and provides a unified interface for sending and receiving messages.
/// 
pub trait OneBotConnection {
    /// Error type for OneBot connections.
    type Error;
    /// The type of the stream of events.
    type StreamOutput<E>: Stream<Item = E>;

    /// Send an action to the OneBot implementation.
    /// 
    /// This method takes an action and sends it to the OneBot implementation.
    /// 
    /// It returns a future that resolves to the response from the OneBot
    async fn send<A>(
        self: Arc<Self>,
        action: OnebotAction<A>,
    ) -> Result<OnebotActionResponse<A::Output>, Self::Error>
    where
        A: 'static,
        A: Payload;

    /// Receive a stream of events from the OneBot implementation.
    /// 
    /// # Example
    /// 
    /// ```
    /// use futures::StreamExt;
    /// let conn = WSConn::new("ws://localhost:6700");
    /// let stream = conn.receive::<Event>();
    /// let mut stream = stream.await.unwrap();
    /// assert_eq!(stream.next().await, Some(Event::Connect(_)));
    /// assert_eq!(stream.next().await, None);
    /// ```
    async fn receive<E>(self: Arc<Self>) -> Self::StreamOutput<E>
    where
        E: 'static + Send + Sync,
        E: DeserializeOwned,
        E: Debug;
}

/// A stream of events.
pub struct EventStream<E = Event> {
    receiver: tokio::sync::mpsc::UnboundedReceiver<E>,
}

impl<E> EventStream<E> {

    /// Create a new event stream.
    /// 
    /// ```
    /// let (tx, rx) = unbounded_channel::<Event>();
    /// let stream = EventStream::new(rx);
    /// ```
    /// 
    pub fn new(receiver: tokio::sync::mpsc::UnboundedReceiver<E>) -> Self {
        Self { receiver }
    }
}

impl<E> Stream for EventStream<E> {
    type Item = E;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        match self.receiver.poll_recv(cx) {
            Poll::Ready(Some(event)) => Poll::Ready(Some(event)),
            Poll::Ready(None) => Poll::Ready(None),
            Poll::Pending => Poll::Pending,
        }
    }
}
