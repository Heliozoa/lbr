use serde::{Deserialize, Serialize};
use std::fmt::Debug;
use thiserror::Error;

pub type WebResult<T> = Result<T, WebError>;

#[derive(Debug, Clone, Error, Serialize, Deserialize)]
#[error("{message}")]
pub struct WebError {
    pub message: String,
}

impl WebError {
    pub fn new(message: impl ToString) -> Self {
        Self {
            message: message.to_string(),
        }
    }
    pub fn from<E: std::error::Error>(e: E) -> Self {
        Self {
            message: e.to_string(),
        }
    }
}
