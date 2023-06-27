use thiserror::Error;

#[derive(Error, Debug)]
pub enum DafunkError {
    #[error("Serde Error: {0}")]
    SerdeError(#[from] serde_json::Error),
    #[error("WebSocket Error: {0}")]
    WSError(#[from] tokio_tungstenite::tungstenite::Error),
    #[error("Http Error: {0}")]
    HttpError(#[from] hyper::Error),
    #[error("Time out {0}")]
    Timeout(#[from] tokio::time::error::Elapsed),
    #[error("Unknown Error: ")]
    Unknown,

}