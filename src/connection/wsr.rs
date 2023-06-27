use std::{
    collections::HashMap, error::Error, fmt::Debug, net::SocketAddr, sync::Arc, time::Duration,
};

use futures::{channel::mpsc::UnboundedReceiver, stream::SplitSink, SinkExt, StreamExt};
use serde::de::DeserializeOwned;
use serde_json::Value;
use tokio::{
    net::{TcpListener, TcpStream},
    sync::{mpsc::UnboundedSender, Mutex},
};
use tokio_tungstenite::{self, tungstenite::Message, WebSocketStream};

use crate::{
    error::DafunkError,
    payload::Payload,
    types::{
        action::{BotsStatus, OnebotAction},
        response::OnebotActionResponse,
        *,
    },
};

use super::obc::{EventStream, OneBotConnection};
type Wrapper<S> = Arc<Mutex<HashMap<Selft, Arc<Mutex<S>>>>>;
type WriteStream = SplitSink<WebSocketStream<TcpStream>, Message>;

pub struct WSRConn {
    write_connections: Wrapper<WriteStream>,
    address: SocketAddr,
}

impl WSRConn {
    pub fn new(address: &str) -> Self {
        let address = address.parse().expect("Invalid socket address");

        let write_connections = Arc::new(Mutex::new(HashMap::new()));

        Self {
            write_connections,
            address,
        }
    }
}

async fn get_seft(
    ws_stream: &mut WebSocketStream<TcpStream>,
) -> Result<Vec<Selft>, tokio_tungstenite::tungstenite::Error> {
    let action = action::GetStatus {};
    let payload = OnebotAction::new(action).json();
    let payload = Message::Text(payload);
    ws_stream.send(payload.clone()).await?;
    while let Some(msg) = ws_stream.next().await {
        let msg = msg?;
        if let Message::Text(text) = msg {
            let res: Result<OnebotActionResponse<BotsStatus>, serde_json::Error> =
                serde_json::from_str::<OnebotActionResponse<BotsStatus>>(&text);
            if let Ok(resp) = res {
                let selfts = resp
                    .data
                    .bots
                    .iter()
                    .map(|bot| bot.self_.clone())
                    .collect::<Vec<Selft>>();
                return Ok(selfts);
            } else {
                ws_stream.send(payload.clone()).await?;
            }
        }
    }
    Err(tokio_tungstenite::tungstenite::Error::ConnectionClosed)
}
impl OneBotConnection for WSRConn {
    type Error = DafunkError;
    type StreamOutput<E> = EventStream<E, Self::Error>;
    async fn send<A>(
        &mut self,
        action: OnebotAction<A>,
    ) -> Result<OnebotActionResponse<A::Output>, Self::Error>
    where
        A: 'static,
        A: Payload,
    {
        let write_conns = self.write_connections.clone();
        let len = write_conns.lock().await.len();

        if action.self_.is_none() && len > 0 {
            panic!("No self_ specified")
        }
        let payload = action.json();
        let selft = action.self_.clone().unwrap();
        let msg = Message::Text(payload);

        let mut wc = write_conns.lock().await;
        let w_stream = wc.get_mut(&selft).expect("No such self_");
        w_stream.lock().await.send(msg).await?;
        unimplemented!()
    }

    async fn receive<E>(&mut self) -> super::obc::EventStream<E, Self::Error>
    where
        E: DeserializeOwned + std::fmt::Debug + Send + Sync + 'static,
    {
        let write_cons = self.write_connections.clone();
        let address = self.address;
        let (tx, rx) = tokio::sync::mpsc::unbounded_channel();

        let listener = TcpListener::bind(&address)
            .await
            .expect("Can't bind to address");

        let fut = async move {
            while let Ok((tcp_stream, _)) = listener.accept().await {
                if tcp_stream.peer_addr().is_err() {
                    tx.send(Err(DafunkError::Unknown)).ok();
                    continue;
                }

                tokio::spawn(read_ws_stream(tcp_stream, write_cons.clone(), tx.clone()));
            }
        };

        tokio::spawn(fut);
        EventStream::new(rx)
    }
}

async fn send_message<E: DeserializeOwned + Debug + 'static>(
    msg: Message,
    tx: &UnboundedSender<Result<E, DafunkError>>,
) -> Result<(), Box<dyn Error>> {
    let msg = msg;
    if let Message::Text(text) = msg {
        let response = serde_json::from_str::<OnebotActionResponse<serde_json::Value>>(&text);
        if response.is_ok() {
            let value: Value = serde_json::from_str(&text)?;
            //res_tx.clone().send(value)?;
            return Ok(());
        }
        let event: E = serde_json::from_str(&text)?;
        tx.send(Ok(event))?;
    }
    Ok(())
}

async fn read_ws_stream<E: DeserializeOwned + Debug + 'static>(
    tcp_stream: TcpStream,
    write_cons: Wrapper<WriteStream>,
    tx: UnboundedSender<Result<E, DafunkError>>,
) -> Result<(), DafunkError> {
    let mut ws_stream = tokio_tungstenite::accept_async(tcp_stream)
        .await
        .expect("Error during the websocket handshake occurred");

    let selt = get_seft(&mut ws_stream).await?;
    println!("selt: {:?}", selt);

    let (write, read) = ws_stream.split();
    let (write, mut read) = (Arc::new(Mutex::new(write)), read);

    for selt in selt {
        let mut write_cons = write_cons.lock().await;
        write_cons.insert(selt, write.clone());
    }

    while let Some(Ok(msg)) = read.next().await {
        let res = send_message(msg, &tx).await;
        if let Err(e) = res {
            log::error!("Error: {}", e);
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use futures::StreamExt;

    use crate::types::{action::GetStatus, Event};

    use super::*;

    #[tokio::test]
    async fn test_connect() {
        let addr = "127.0.0.1:6703";

        let mut conn = WSRConn::new(addr);
        let mut stream = conn.receive::<Event>().await;

        while let Some(Ok(event)) = stream.next().await {
            tokio::spawn(async move {
                let action = GetStatus {};
                let action = OnebotAction::new(action);
                conn.send(action).await.unwrap();
            });
        }
    }
}
