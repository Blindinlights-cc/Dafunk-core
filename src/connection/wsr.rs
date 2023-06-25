use std::{collections::HashMap, net::SocketAddr, sync::Arc, time::Duration};

use futures::{stream::SplitSink, SinkExt, StreamExt};
use serde::de::DeserializeOwned;
use serde_json::Value;
use tokio::{
    net::{TcpListener, TcpStream},
    sync::{
        mpsc::{UnboundedReceiver, UnboundedSender},
        Mutex,
    },
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
    reciever: UnboundedReceiver<Value>,
    sender: UnboundedSender<Value>,
}
impl WSRConn {
    pub fn new(address: &str) -> Self {
        let address = address.parse().expect("Invalid socket address");

        let write_connections = Arc::new(Mutex::new(HashMap::new()));
        let (res_sender, res_reciever) = tokio::sync::mpsc::unbounded_channel();
        Self {
            write_connections,
            address,
            reciever: res_reciever,
            sender: res_sender,
        }
    }
}

async fn get_seft(
    ws_stream: &mut WebSocketStream<TcpStream>,
) -> Result<Vec<Selft>, tokio_tungstenite::tungstenite::Error> {
    println!("get_seft");
    let action = action::GetStatus {};
    let payload = OnebotAction::new(action).json();
    println!("{}", payload);
    let msg = Message::Text(payload);
    ws_stream.send(msg).await?;
    while let Some(msg) = ws_stream.next().await {
        let msg = msg?;
        if let Message::Text(text) = msg {
            let res: Result<OnebotActionResponse<BotsStatus>, serde_json::Error> =
                serde_json::from_str::<OnebotActionResponse<BotsStatus>>(&text);
            if let Ok(resp) = res {
                let selts = resp
                    .data
                    .bots
                    .iter()
                    .map(|bot| bot.self_.clone())
                    .collect::<Vec<Selft>>();
                return Ok(selts);
            } else {
                continue;
            }
        }
    }
    Err(tokio_tungstenite::tungstenite::Error::ConnectionClosed)
}
impl OneBotConnection for WSRConn {
    type Error = DafunkError;
    type StreamOutput<E> = EventStream<E>;
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

        use tokio::time::timeout;
        let res = timeout(Duration::from_secs(10), self.reciever.recv()).await?;
        if let Some(res) = res {
            let res = serde_json::from_value::<OnebotActionResponse<A::Output>>(res)?;
            Ok(res)
        } else {
            Err(DafunkError::Unknown)
        }
    }

    async fn receive<E>(&mut self) -> super::obc::EventStream<E>
    where
        E: DeserializeOwned + std::fmt::Debug + Send + Sync + 'static,
    {
        let write_cons = self.write_connections.clone();
        let address = self.address.clone();
        let res_tx = self.sender.clone();
        let (tx, rx) = tokio::sync::mpsc::unbounded_channel();
        let fut = async move {
            let listener = TcpListener::bind(&address)
                .await
                .expect("Can't bind to address");
            while let Ok((tcp_stream, _)) = listener.accept().await {
                if tcp_stream.peer_addr().is_err() {
                    continue;
                }
                let write_cons = write_cons.clone();
                let res_tx = res_tx.clone();
                let tx = tx.clone();
                tokio::spawn(async move {
                    let mut ws_stream = tokio_tungstenite::accept_async(tcp_stream)
                        .await
                        .expect("Error during the websocket handshake occurred");

                    let selt = get_seft(&mut ws_stream).await.unwrap();

                    let (write, read) = ws_stream.split();
                    let (write, mut read) = (Arc::new(Mutex::new(write)), read);

                    for selt in selt {
                        let mut write_cons = write_cons.lock().await;
                        write_cons.insert(selt.clone(), write.clone());
                    }

                    while let Some(msg) = read.next().await {
                        let msg = msg.unwrap();
                        if let Message::Text(text) = msg {
                            let response = serde_json::from_str::<
                                OnebotActionResponse<serde_json::Value>,
                            >(&text);
                            if let Ok(_) = response {
                                let value: Value = serde_json::from_str(&text).unwrap();
                                res_tx.clone().send(value).unwrap();
                                continue;
                            }
                            let event: E = serde_json::from_str(&text).unwrap();
                            tx.send(event).unwrap();
                        }
                    }
                });
            }
        };

        tokio::spawn(fut);
        EventStream::new(rx)
    }
}

#[cfg(test)]
mod tests {
    use futures::StreamExt;

    use crate::types::Event;

    use super::*;

    #[tokio::test]
    async fn test_connect() {
        let addr = "127.0.0.1:6703";

        let mut conn = WSRConn::new(addr);
        let mut stream = conn.receive::<Event>().await;

        while let Some(event) = stream.next().await {
            match event{
                Event::GroupMessage(msg) => {
                    let user_name = msg.user_id;
                    let group_id = msg.group_id;
                    let text = msg.alt_message;
                    println!("{} {} {}", user_name, group_id, text);

                }
                _ => {}
            }
        }
    }
}
