use std::{collections::HashMap, fmt::Debug, net::SocketAddr, sync::Arc};

use futures::{stream::SplitSink, SinkExt, StreamExt};
use rand::{distributions::Alphanumeric, Rng};
use serde::de::DeserializeOwned;
use serde_json::Value;
use tokio::{
    net::{TcpListener, TcpStream},
    sync::{mpsc::UnboundedSender, oneshot::Sender, Mutex, RwLock},
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
type Wrapper<S> = Arc<RwLock<HashMap<Selft, Arc<Mutex<S>>>>>;
type WriteStream = SplitSink<WebSocketStream<TcpStream>, Message>;
#[derive(Clone)]
pub struct WSRConn {
    write_connections: Wrapper<WriteStream>,
    address: SocketAddr,
    responses: Arc<Mutex<HashMap<String, Sender<OnebotActionResponse<Value>>>>>,
}

impl WSRConn {
    pub fn new(address: &str) -> Self {
        let address = address.parse().expect("Invalid socket address");

        let write_connections = Arc::new(RwLock::new(HashMap::new()));
        let responses = Arc::new(Mutex::new(HashMap::new()));
        Self {
            write_connections,
            address,
            responses,
        }
    }
    async fn read_response(
        &self,
        echo: String,
    ) -> Result<OnebotActionResponse<Value>, DafunkError> {
        let (tx, rx) = tokio::sync::oneshot::channel::<OnebotActionResponse<Value>>();
        self.responses.lock().await.insert(echo, tx);
        rx.await.map_err(|_| DafunkError::Unknown)
    }
}

async fn get_seft(
    ws_stream: &mut WebSocketStream<TcpStream>,
) -> Result<Vec<Selft>, tokio_tungstenite::tungstenite::Error> {
    let action = action::GetStatus {};
    let payload = OnebotAction::new(action).json();
    let payload = Message::Text(payload);
    tokio::time::sleep(std::time::Duration::from_millis(1000)).await;
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
                println!("Selfts: {:?}", selfts);
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
        let random = rand::thread_rng()
            .sample_iter(&Alphanumeric)
            .take(10)
            .map(char::from)
            .collect::<String>();
        let mut action = action;
        action.echo = Some(random.clone());
        let payload = action.json();
        let selft = action.self_.clone().unwrap();
        let msg = Message::Text(payload);

        tokio::spawn(async move {
            let wc = write_conns.read().await;
            let w_stream = wc.get(&selft).unwrap().clone();
            w_stream.lock().await.send(msg).await.unwrap();
        });
        println!("Sent");
        let res = self.read_response(random).await?;
        Ok(res.into_response())
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
        let response = self.responses.clone();
        let fut = async move {
            while let Ok((tcp_stream, _)) = listener.accept().await {
                if tcp_stream.peer_addr().is_err() {
                    tx.send(Err(DafunkError::Unknown)).ok();
                    continue;
                }
                tokio::spawn(read_ws_stream(
                    tcp_stream,
                    write_cons.clone(),
                    tx.clone(),
                    response.clone(),
                ));
            }
        };
        tokio::spawn(fut);
        EventStream::new(rx)
    }
}

async fn read_ws_stream<E: DeserializeOwned + Debug + 'static>(
    tcp_stream: TcpStream,
    write_cons: Wrapper<WriteStream>,
    tx: UnboundedSender<Result<E, DafunkError>>,
    res_sender: Arc<Mutex<HashMap<String, Sender<OnebotActionResponse<Value>>>>>,
) -> Result<(), DafunkError> {
    let mut ws_stream = tokio_tungstenite::accept_async(tcp_stream)
        .await
        .expect("Error during the websocket handshake occurred");

    let selt = get_seft(&mut ws_stream).await?;

    let (write, read) = ws_stream.split();
    let (write, mut read) = (Arc::new(Mutex::new(write)), read);

    for selt in selt {
        let mut write_cons = write_cons.write().await;
        write_cons.insert(selt, write.clone());
    }

    while let Some(Ok(msg)) = read.next().await {
        let msg = msg;
        if let Message::Text(text) = msg {
            let response = serde_json::from_str(&text);
            if response.is_ok() {
                let response: OnebotActionResponse<Value> = response.unwrap();
                let mut res_sender= res_sender.lock().await;
                let echo = response.echo.clone().unwrap_or_else(|| "".to_string());
                res_sender.remove(&echo).map(|tx| tx.send(response).ok());
                continue;
            }
            let event = serde_json::from_str::<E>(&text).map_err(DafunkError::SerdeError);
            tx.send(event).ok();
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

        while let Some(Ok(_)) = stream.next().await {
            let mut conn = conn.clone();
            println!("event");
            tokio::spawn(async move {
                let action = GetStatus {};
                let selft = Selft {
                    platform: "qq".to_string(),
                    user_id: "1057584970".to_string(),
                };
                let action = OnebotAction::with_self(action, selft);
                let res = conn.send(action).await.unwrap();
                println!("{:?}", res);
            });
        }
    }
}
