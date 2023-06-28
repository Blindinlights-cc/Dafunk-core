use futures::{stream::SplitSink, SinkExt, StreamExt};
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
}

impl WSConn {
    pub fn new(url: &str) -> Self {
        Self {
            write_stream: None,
            response: Arc::new(Mutex::new(HashMap::new())),
            addr: url.parse().unwrap(),
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
    type StreamOutput<E> = EventStream<E, Self::Error>;
    async fn send<A>(
        &mut self,
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

    async fn receive<E>(&mut self) -> Self::StreamOutput<E>
    where
        E: DeserializeOwned + Debug + Send + Sync + 'static,
    {
        let (ws_stream, _) = connect_async(&self.addr).await.expect("");
        let (write, mut read) = ws_stream.split();
        let (tx, rx) = unbounded_channel();
        self.write_stream = Some(Arc::new(Mutex::new(write)));
        let response_sender = self.response.clone();
        let fut = async move {
            while let Some(Ok(Message::Text(e))) = read.next().await {
                let resp: Result<OnebotActionResponse<Value>, serde_json::Error> =
                    serde_json::from_str(&e);
                if resp.is_ok() {
                    let resp = resp.unwrap();
                    let echo = resp.echo.clone().unwrap();
                    response_sender
                        .lock()
                        .await
                        .remove(&echo)
                        .map(|tx| tx.send(resp));
                    continue;
                }
                let event = serde_json::from_str(&e).map_err(DafunkError::SerdeError);
                tx.send(event).ok();
            }
        };
        tokio::spawn(fut);
        EventStream::new(rx)
    }
}

#[cfg(test)]
mod tests {
    use futures::StreamExt;

    use crate::types::{action::GetStatus, Event};

    use super::*;

    #[tokio::test]
    async fn test() {
        let url = "ws://127.0.0.1:6701";
        let mut conn = WSConn::new(url);
        let mut stream = conn.receive::<Event>().await;
        while let Some(Ok(_)) = stream.next().await {
            let mut conn = conn.clone();
            let action = GetStatus {};
            let action = OnebotAction::new(action);

            tokio::spawn(async move {
                let res = conn.send(action).await.unwrap();
                println!("{:#?}", res);
            });
        }
    }
}
