use crate::{
    connection::obc::OneBotConnection,
    error::DafunkError,
    types::{
        action::{GetLatestEvents, OnebotAction},
        response::OnebotActionResponse,
    },
};
use http::Method;
use hyper::Request;
use serde::de::DeserializeOwned;

use super::obc::EventStream;
#[derive(Debug, Clone)]
pub struct HttpConn {
    client: hyper::Client<hyper::client::HttpConnector>,
    url: hyper::Uri,
}
impl HttpConn {
    pub fn new(url: &str) -> Self {
        Self {
            client: hyper::Client::new(),
            url: url.parse().expect("Invalid url"),
        }
    }
}

impl OneBotConnection for HttpConn {
    type Error = DafunkError;
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
        let req = Request::builder()
            .method(Method::POST)
            .uri(&self.url)
            .header("Content-Type", "application/json")
            .header("Authorization", "Bearer token")
            .body(hyper::Body::from(payload))
            .expect("Invalid request");

        let resp = self.client.request(req).await?;
        let body = hyper::body::to_bytes(resp.into_body()).await?;
        let resp: crate::types::response::OnebotActionResponse<A::Output> =
            serde_json::from_slice(&body)?;
        Ok(resp)
    }

    async fn receive<E>(&mut self) -> EventStream<E>
    where
        E: DeserializeOwned + std::fmt::Debug + Send + Sync + 'static,
    {
        let (tx, rx) = tokio::sync::mpsc::unbounded_channel();
        let conn = self.clone();
        let fut = async move {
            let action = GetLatestEvents {
                limit: 1,
                timeout: 0,
            };
            let action = OnebotAction::new(action);

            let mut conn = conn.clone();
            let res = conn.send(action).await;
            if let Ok(res) = res {
                let value = res.data[0].clone();
                let event = serde_json::from_value(value);
                if let Ok(event) = event {
                    let res = tx.send(event);
                    if res.is_err() {
                        log::error!("send event failed");
                    }
                }
            }
        };
        tokio::spawn(fut);

        EventStream::new(rx)
    }
}

#[cfg(test)]
mod tests {
    // use futures::{stream, StreamExt};

    // use super::*;

    #[tokio::test]
    async fn test() {
        todo!()
    }
}
