use std::{collections::HashMap, fmt::Debug, net::SocketAddr, sync::Arc};

use futures::{stream::SplitSink, SinkExt, StreamExt};
use rand::{distributions::Alphanumeric, Rng};
use serde::de::DeserializeOwned;
use serde_json::Value;
use tokio::{
    net::{TcpListener, TcpStream},
    sync::{mpsc::UnboundedSender, oneshot::Sender, Mutex, RwLock},
};
use tokio_tungstenite::{self, accept_async, tungstenite::Message, WebSocketStream};

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
type Wrapper<S> = RwLock<HashMap<Selft, Arc<Mutex<S>>>>;
type WriteStream = SplitSink<WebSocketStream<TcpStream>, Message>;

pub struct WSRConn {
    access_token: Option<String>,
    tcp_listener: TcpListener,
    write_connections: Wrapper<WriteStream>,
    responses: Mutex<HashMap<String, Sender<OnebotActionResponse<Value>>>>,
}

impl WSRConn {
    async fn read_response(
        self: Arc<Self>,
        echo: String,
    ) -> Result<OnebotActionResponse<Value>, DafunkError> {
        let (tx, rx) = tokio::sync::oneshot::channel::<OnebotActionResponse<Value>>();
        self.responses.lock().await.insert(echo, tx);
        rx.await.map_err(|_| DafunkError::Unknown)
    }
    pub async fn start_with_token(
        addr: SocketAddr,
        access_token: String,
    ) -> Result<Arc<Self>, DafunkError> {
        let tcp_listener = TcpListener::bind(addr).await.expect("Failed to bind");
        let write_connections = RwLock::new(HashMap::new());
        let responses = Mutex::new(HashMap::new());
        let conn = Self {
            access_token: Some(access_token),
            tcp_listener,
            write_connections,
            responses,
        };
        let conn = Arc::new(conn);
        Ok(conn)
    }
    pub async fn start(addr: SocketAddr) -> Result<Arc<Self>, DafunkError> {
        let tcp_listener = TcpListener::bind(addr).await.expect("Failed to bind");
        let write_connections = RwLock::new(HashMap::new());
        let responses = Mutex::new(HashMap::new());
        let conn = Self {
            access_token: None,
            tcp_listener,
            write_connections,
            responses,
        };
        let conn = Arc::new(conn);
        Ok(conn)
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
        self: Arc<Self>,
        action: OnebotAction<A>,
    ) -> Result<OnebotActionResponse<A::Output>, Self::Error>
    where
        A: 'static,
        A: Payload,
    {
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
        let binding = self.clone();
        tokio::spawn(async move {
            let wc = binding.write_connections.read().await;
            let w_stream = wc.get(&selft).unwrap().clone();
            w_stream.lock().await.send(msg).await.unwrap();
        });
        println!("Sent");
        let res = self.read_response(random).await?;
        Ok(res.into_response())
    }

    async fn receive<E>(self: Arc<Self>) -> super::obc::EventStream<E, Self::Error>
    where
        E: DeserializeOwned + std::fmt::Debug + Send + Sync + 'static,
    {
        let (tx, rx) = tokio::sync::mpsc::unbounded_channel();

        let fut = async move {
            while let Ok((tcp_stream, _)) = self.tcp_listener.accept().await {
                if tcp_stream.peer_addr().is_err() {
                    tx.send(Err(DafunkError::Unknown)).ok();
                    continue;
                }
                tokio::spawn(read_ws_stream(self.clone(), tcp_stream, tx.clone()));
            }
        };
        tokio::spawn(fut);
        EventStream::new(rx)
    }
}

async fn read_ws_stream<E: DeserializeOwned + Debug + 'static>(
    conn: Arc<WSRConn>,
    tcp_stream: TcpStream,
    tx: UnboundedSender<Result<E, DafunkError>>,
) -> Result<(), DafunkError> {
    let mut ws_stream = accept_async(tcp_stream)
        .await
        .expect("Error during the websocket handshake occurred");

    let selt = get_seft(&mut ws_stream).await?;

    let (write, read) = ws_stream.split();
    let (write, mut read) = (Arc::new(Mutex::new(write)), read);

    for selt in selt {
        let mut write_cons = conn.write_connections.write().await;
        write_cons.insert(selt, write.clone());
    }

    while let Some(Ok(msg)) = read.next().await {
        let msg = msg;

        if let Message::Text(text) = msg {
            let response = serde_json::from_str(&text);
            if response.is_ok() {
                let response: OnebotActionResponse<Value> = response.unwrap();
                let mut res_sender = conn.responses.lock().await;
                let echo = response.echo.clone().unwrap_or_else(|| "".to_string());
                res_sender.remove(&echo).map(|tx| tx.send(response).ok());
                continue;
            }

            let event = serde_json::from_str::<E>(&text).map_err(DafunkError::SerdeError);
            tx.send(event).ok();
        } else if let Message::Binary(msg) = msg {
            let response = serde_json::from_slice(&msg);
            if response.is_ok() {
                let response: OnebotActionResponse<Value> = response.unwrap();
                let mut res_sender = conn.responses.lock().await;
                let echo = response.echo.clone().unwrap_or_else(|| "".to_string());
                res_sender.remove(&echo).map(|tx| tx.send(response).ok());
                continue;
            }
            let event = serde_json::from_slice::<E>(&msg).map_err(DafunkError::SerdeError);
            tx.send(event).ok();
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use crate::types::action::GetStatus;

    use super::*;
    use futures::StreamExt;
    #[tokio::test]
    async fn test_connect() {
        let conn = WSRConn::start("127.0.0.1:6703".parse().unwrap())
            .await
            .unwrap();
        let mut stream = conn.clone().receive::<Event>().await;

        while let Some(Ok(e)) = stream.next().await {
            println!("{e:?}");
            let action = GetStatus {};
            let action = OnebotAction::with_self(
                action,
                Selft {
                    platform: "qq".into(),
                    user_id: "1057584970".into(),
                },
            );
            conn.clone().send(action).await.ok();
        }
    }
}
