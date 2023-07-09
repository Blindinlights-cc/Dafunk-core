use std::{collections::HashMap, convert::Infallible, sync::Mutex};

use http::{Request, Response};
use hyper::Body;
use serde_json::Value;
use tokio::{
    sync::{mpsc::UnboundedSender, oneshot::Sender},
    time::timeout,
};

use crate::error::DafunkError;

use super::obc::{EventStream, OneBotConnection};
#[allow(unused)]
pub struct Webhook {
    url: String,
    token: Option<String>,
    event_map: Mutex<HashMap<String, Sender<String>>>,
}

impl OneBotConnection for Webhook {
    type Error = DafunkError;
    type StreamOutput<E> = EventStream<E>;

    async fn send<A>(
        self: std::sync::Arc<Self>,
        _action: crate::types::action::OnebotAction<A>,
    ) -> Result<crate::types::response::OnebotActionResponse<A::Output>, Self::Error>
    where
        A: 'static,
        A: crate::payload::Payload,
    {
        todo!()
    }

    async fn receive<E>(self: std::sync::Arc<Self>) -> Self::StreamOutput<E>
    where
        E: 'static + Send + Sync,
        E: serde::de::DeserializeOwned,
        E: std::fmt::Debug,
    {
        todo!()
    }
}
async fn handle_request<E>(
    req: Request<Body>,
    conn: std::sync::Arc<Webhook>,
    tx: UnboundedSender<E>,
) -> Result<Response<Body>, Infallible>
where
    E: 'static + Send + Sync,
    E: serde::de::DeserializeOwned,
    E: std::fmt::Debug,
{
    if let Some(token) = &conn.token {
        match req.headers().get("Authorization") {
            Some(auth) if auth == format!("Bearer {}", token).as_str() => {}
            _ => {
                return Ok(Response::builder()
                    .status(401)
                    .body(Body::from("Unauthorized"))
                    .unwrap());
            }
        }
    }
    let body = req.into_body();
    let json = hyper::body::to_bytes(body).await.unwrap();
    let event: E = serde_json::from_slice(&json).unwrap();
    tx.send(event).ok();
    let value: Value = serde_json::from_slice(&json).unwrap();
    let event_id = value["id"].as_str();
    if let Some(id) = event_id {
        let (tx, rx) = tokio::sync::oneshot::channel();
        conn.event_map.lock().unwrap().insert(id.to_string(), tx);
        let res = timeout(std::time::Duration::from_secs(60), rx)
            .await
            .map(|res| {
                Response::builder()
                    .status(200)
                    .header("Content-Type", "application/json")
                    .body(Body::from(res.unwrap_or_else(|_| "".to_string())))
                    .unwrap()
            })
            .unwrap_or_else(|_| Response::builder().status(500).body(Body::empty()).unwrap());
        conn.event_map.lock().unwrap().remove(id);
        return Ok(res);
    }

    let res = Response::builder().status(204).body(Body::empty()).unwrap();
    Ok(res)
}
