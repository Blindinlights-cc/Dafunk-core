#![allow(unused)]
use futures::{stream::SplitSink, SinkExt, StreamExt};
use http::{Request, Uri};
use rand::{distributions::Alphanumeric, Rng};
use serde::de::DeserializeOwned;
use serde_json::Value;
use std::{collections::HashMap, fmt::Debug, sync::Arc};
use tokio::{
    net::TcpStream,
    sync::oneshot::Sender,
    sync::{mpsc::unbounded_channel, Mutex},
};
use tokio_tungstenite::{
    self, connect_async, tungstenite::Message, MaybeTlsStream, WebSocketStream,
};
type WriteStream = SplitSink<WebSocketStream<MaybeTlsStream<TcpStream>>, Message>;
#[derive(Clone)]
pub struct WSConn {
    write_stream: Option<Arc<Mutex<WriteStream>>>,
    response: Arc<Mutex<HashMap<String, Sender<OnebotActionResponse<Value>>>>>,
    addr: String,
    access_token: Option<String>,
}

impl WSConn {
    pub fn new(url: &str) -> Self {
        Self {
            write_stream: None,
            response: Arc::new(Mutex::new(HashMap::new())),
            addr: url.parse().unwrap(),
            access_token: None,
        }
    }
    async fn read_response(
        &self,
        echo: String,
    ) -> Result<OnebotActionResponse<Value>, DafunkError> {
        let (tx, rx) = tokio::sync::oneshot::channel::<OnebotActionResponse<Value>>();
        self.response.lock().await.insert(echo, tx);
        rx.await.map_err(|_| DafunkError::Unknown)
    }
}
use crate::{
    connection::obc::OneBotConnection,
    error::DafunkError,
    types::{action::OnebotAction, response::OnebotActionResponse},
};

use super::obc::EventStream;

impl OneBotConnection for WSConn {
    type Error = DafunkError;
    type StreamOutput<E> = EventStream<E>;
    async fn send<A>(
        self: Arc<Self>,
        action: OnebotAction<A>,
    ) -> Result<OnebotActionResponse<A::Output>, Self::Error>
    where
        A: 'static,
        A: crate::payload::Payload,
    {
        let random = rand::thread_rng()
            .sample_iter(&Alphanumeric)
            .take(10)
            .map(char::from)
            .collect::<String>();
        let mut action = action;
        action.echo = Some(random.clone());

        let stream = self.write_stream.clone().unwrap();
        let payload = action.json();
        tokio::spawn(async move {
            stream.lock().await.send(Message::Text(payload)).await.ok();
        });
        let res = self.read_response(random).await?;
        Ok(res.into_response())
    }

    async fn receive<E>(self: Arc<Self>) -> Self::StreamOutput<E>
    where
        E: DeserializeOwned + Debug + Send + Sync + 'static,
    {
        let req = if let Some(ref token) = self.access_token {
            let bear = format!("Bear {token}");
            Request::builder()
                .uri(&self.addr)
                .header("Authorization", &bear)
                .body(())
                .unwrap()
        } else {
            Request::builder().uri(&self.addr).body(()).unwrap()
        };

        let (ws_stream, _) = connect_async(req).await.expect("");
        let (write, mut read) = ws_stream.split();
        todo!()
    }
}

#[cfg(test)]
mod tests {
    use futures::StreamExt;

    use crate::types::{action::GetStatus, Event};

    use super::*;

    #[tokio::test]
    async fn test() {
        todo!();
    }
}
