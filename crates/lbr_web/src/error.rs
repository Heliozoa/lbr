//! Our custom error type.

use serde::{Deserialize, Serialize};
use std::fmt::Debug;
use thiserror::Error;
use wasm_bindgen::JsValue;

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
    pub fn from_js(js: JsValue) -> Self {
        Self {
            message: format!("{js:?}"),
        }
    }
}

impl From<eyre::Report> for WebError {
    fn from(value: eyre::Report) -> Self {
        Self {
            message: value.to_string(),
        }
    }
}

impl From<JsValue> for WebError {
    fn from(value: JsValue) -> Self {
        Self {
            message: format!("{value:#?}"),
        }
    }
}
