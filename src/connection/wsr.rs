use std::{
    collections::HashMap, convert::Infallible, fmt::Debug, net::SocketAddr, sync::Arc,
    time::Duration,
};
extern crate log;
use futures::{future, SinkExt, StreamExt, TryStreamExt};
use http::{
    header::{CONNECTION, SEC_WEBSOCKET_ACCEPT, SEC_WEBSOCKET_KEY, UPGRADE},
    Request, Response, StatusCode,
};
use hyper::{upgrade::Upgraded, Body};

use log::info;
use serde::de::DeserializeOwned;
use serde_json::Value;
use std::sync::Mutex;
use tokio::sync::{
    mpsc::{unbounded_channel, UnboundedSender},
    oneshot::Sender,
};
use tokio_tungstenite::{
    self,
    tungstenite::{handshake::derive_accept_key, protocol::Role, Message},
    WebSocketStream,
};

use crate::{
    error::DafunkError,
    payload::Payload,
    types::{action::OnebotAction, response::OnebotActionResponse, *},
};

use super::obc::{EventStream, OneBotConnection};
type ActionMap = Mutex<HashMap<Selft, UnboundedSender<String>>>;
type ResponseSender = Sender<OnebotActionResponse<Value>>;
type ResponseMap = Mutex<HashMap<String, ResponseSender>>;
/// A connection to a OneBot implementation using the WebSocket Reverse protocol.
///
/// See [Onebot-反向Websocket](https://12.onebot.dev/connect/communication/websocket-reverse/) for more information.
pub struct WSRConn {
    access_token: Option<String>,
    addr: SocketAddr,
    responses: ResponseMap,
    actions: ActionMap,
}
impl WSRConn {
    /// Create a new [WSRConn]
    /// ```
    /// use dafunk::connection::wsr::WSRConn;
    ///
    /// let conn = WSRConn::new("127.0.0.1:6700".parse().unwrap());
    /// ```
    ///
    pub fn new(addr: SocketAddr) -> Self {
        Self {
            access_token: None,
            addr,
            responses: Mutex::new(HashMap::new()),
            actions: Mutex::new(HashMap::new()),
        }
    }

    /// Set the access token for the connection
    pub fn token(self, access_token: String) -> Self {
        Self {
            access_token: Some(access_token),
            ..self
        }
    }
}

async fn handle_connection<E>(
    ws_stream: WebSocketStream<Upgraded>,
    tx: UnboundedSender<E>,
    wsr: Arc<WSRConn>,
) where
    E: DeserializeOwned + Debug + Send + Sync + 'static,
{
    let (mut outgoing, incoming) = ws_stream.split();

    let (a_tx, mut a_rx) = unbounded_channel::<String>();
    let broadcast_incoming = incoming.try_for_each(|msg| {
        let value = match msg {
            Message::Text(ref text) => serde_json::from_str::<Value>(&text).unwrap(),
            Message::Binary(ref bin) => serde_json::from_slice::<Value>(&bin).unwrap(),
            _ => return future::ok(()),
        };

        let action_response = serde_json::from_value::<OnebotActionResponse<Value>>(value.clone());
        if let Ok(action_response) = action_response {
            println!("action_response");
            if let Some(tx) = wsr
                .responses
                .lock()
                .unwrap()
                .remove(&action_response.echo.clone().unwrap_or_default())
            {
                tx.send(action_response).ok();
                println!("action_response send");
            }
            return future::ok(());
        }

        let selft = serde_json::from_value::<Selft>(value.clone()["self"].clone());

        if let Ok(selft) = selft {
            wsr.actions
                .lock()
                .unwrap()
                .insert(selft.clone(), a_tx.clone());
        }
        let event = serde_json::from_value::<E>(value);
        if let Ok(event) = event {
            tx.send(event).ok();
        }
        future::ok(())
    });
    let fut = async move {
        while let Some(action) = a_rx.recv().await {
            outgoing.send(Message::Text(action)).await.ok();
        }
    };
    tokio::select! {
        _ = broadcast_incoming => {}
        _ = fut => {}
    }
}
async fn handle_request<E>(
    mut req: Request<Body>,
    wsr: Arc<WSRConn>,
    tx: UnboundedSender<E>,
) -> Result<Response<Body>, Infallible>
where
    E: DeserializeOwned + Debug + Send + Sync + 'static,
{
    if wsr.access_token.is_some() {
        match req.headers().get("Authorization") {
            Some(auth) if auth == wsr.access_token.as_ref().unwrap() => {}
            _ => return Ok(Response::builder().status(401).body(Body::empty()).unwrap()),
        };
    }
    let headers = req.headers();
    let key = headers.get(SEC_WEBSOCKET_KEY);
    let derived = key.map(|k| derive_accept_key(k.as_bytes()));
    let wsr = wsr.clone();
    tokio::spawn(async move {
        match hyper::upgrade::on(&mut req).await {
            Ok(upgraded) => {
                handle_connection(
                    WebSocketStream::from_raw_socket(upgraded, Role::Server, None).await,
                    tx,
                    wsr,
                )
                .await
            }
            Err(e) => log::error!("upgrade error: {}", e),
        }
    });
    let mut res = Response::new(Body::empty());
    *res.status_mut() = StatusCode::SWITCHING_PROTOCOLS;
    res.headers_mut()
        .append(CONNECTION, "Upgrade".parse().unwrap());
    res.headers_mut()
        .append(UPGRADE, "websocket".parse().unwrap());
    res.headers_mut().append(
        SEC_WEBSOCKET_ACCEPT,
        derived.unwrap_or("".into()).parse().unwrap(),
    );

    Ok(res)
}

impl OneBotConnection for WSRConn {
    type Error = DafunkError;
    type StreamOutput<E> = EventStream<E>;
    async fn send<A>(
        self: Arc<Self>,
        action: OnebotAction<A>,
    ) -> Result<OnebotActionResponse<A::Output>, Self::Error>
    where
        A: 'static,
        A: Payload,
    {
        let mut action = action;

        let (tx, rx) = tokio::sync::oneshot::channel::<OnebotActionResponse<Value>>();
        let echo = action
            .echo
            .clone()
            .unwrap_or(uuid::Uuid::new_v4().to_string());
        action.echo = Some(echo.clone());
        let selft = action.self_.clone().unwrap();
        let payload = action.json();

        self.responses.lock().unwrap().insert(echo, tx);
        let tx = self.actions.lock().unwrap().get(&selft).cloned();
        if let Some(tx) = tx {
            tx.send(payload).ok();
        } else {
            log::error!("No connection for selft: {:?}", selft);
            return Err(DafunkError::Unknown);
        }
        tokio::time::timeout(Duration::from_secs(10), rx)
            .await?
            .map_err(|_| DafunkError::Unknown)
            .map(|res| res.into_response())
    }

    async fn receive<E>(self: Arc<Self>) -> Self::StreamOutput<E>
    where
        E: 'static + Send + Sync,
        E: DeserializeOwned,
        E: Debug,
    {
        let (tx, rx) = unbounded_channel::<E>();
        let wsr = self.clone();
        let make_svc_fn = hyper::service::make_service_fn(move |_| {
            let wsr = wsr.clone();
            let tx = tx.clone();
            async move {
                Ok::<_, Infallible>(hyper::service::service_fn(move |req: Request<Body>| {
                    let wsr = wsr.clone();
                    let tx: UnboundedSender<E> = tx.clone();
                    async move { handle_request(req, wsr, tx).await }
                }))
            }
        });
        let server = hyper::Server::bind(&self.addr).serve(make_svc_fn);
        tokio::spawn(server);
        info!("Listening on {}", self.addr);
        EventStream::new(rx)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use futures::StreamExt;
    use crate::types::action::GetStatus;

    #[tokio::test]
    async fn test() {
        let wsr = WSRConn::new("127.0.0.1:6703".parse().unwrap());
        let wsr = Arc::new(wsr);
        let mut stream = wsr.clone().receive::<Event>().await;

        while let Some(_) = stream.next().await {
            let wsr = wsr.clone();
            tokio::spawn(async move {
                let get_status = GetStatus {};
                let mut action = OnebotAction::new(get_status);
                action.self_ = Some(Selft {
                    platform: "qq".into(),
                    user_id: "1057584970".into(),
                });
                let res = wsr.send(action).await;
                println!("{:?}", res);
            });
        }
    }
}
