use std::fmt::Debug;
use futures::{SinkExt, StreamExt};
use serde::de::DeserializeOwned;
use tokio::net::TcpStream;
use tokio_tungstenite::{self, tungstenite::Message, MaybeTlsStream, WebSocketStream};

pub struct WSConn {
    ws_stream: WebSocketStream<MaybeTlsStream<TcpStream>>,
}

impl WSConn {
    pub async fn connect(url: &str) -> Result<Self, tokio_tungstenite::tungstenite::Error> {
        let (ws_stream, _) = tokio_tungstenite::connect_async(url).await?;
        Ok(Self { ws_stream })
    }
}
use crate::{
    connection::obc::OneBotConnection,
    types::{action::OnebotAction, response::OnebotActionResponse},
};

use super::obc::EventStream;

impl OneBotConnection for WSConn {
    type Error = tokio_tungstenite::tungstenite::Error;
    type StreamOutput<E> = EventStream<E>;
    async fn send<A>(
        &mut self,
        action: OnebotAction<A>,
    ) -> Result<OnebotActionResponse<A::Output>, Self::Error>
    where
        A: 'static,
        A: crate::payload::Payload,
    {
        let payload = action.json();
        let msg = Message::Text(payload);
        self.ws_stream.send(msg).await?;

        if let Some(msg) = self.ws_stream.next().await {
            let msg = msg?;
            if let Message::Text(text) = msg {
                let resp: crate::types::response::OnebotActionResponse<A::Output> =
                    serde_json::from_str(&text).unwrap();
                return Ok(resp);
            }
        }
        unreachable!()
    }

    async fn receive<E>(&mut self) -> super::obc::EventStream<E>
    where
        E: DeserializeOwned + Debug + Send + Sync + 'static,
    {
        unimplemented!()
        // let (tx, rx) = tokio::sync::mpsc::unbounded_channel();
        // tokio::spawn(async move {
        //     loop {
        //         if let Some(msg) = self.ws_stream.next().await {
        //             let msg = if msg.is_ok() { msg.unwrap() } else { continue };

        //             if let Message::Text(text) = msg {
        //                 let resp: Value = serde_json::from_str(&text).unwrap_or(Value::Null);
        //                 let resp: E = resp.into();
        //                 let res = tx.send(resp);
        //                 if res.is_err() {
        //                     continue;
        //                 }
        //             }
        //         }
        //     }
        // });
        // EventStream::new(rx)
    }
}

#[cfg(test)]
mod tests {
    use futures::StreamExt;

    use crate::{types::Event, connection::obc::EventStream};

    use super::*;

    #[tokio::test]
    async fn test() {
        let url = "ws://127.0.0.1:6701";
        let mut conn = WSConn::connect(url).await.unwrap();
        let mut stream:EventStream<Event> = conn.receive().await;
        while let Some(msg) = stream.next().await {
            println!("{:#?}", msg);
        }
    }
}
