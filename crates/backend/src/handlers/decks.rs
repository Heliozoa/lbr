//! /decks
//! Handlers related to decks.

use crate::authentication::Authentication;
use axum::{
    extract::{Path, State},
    Json,
};

pub async fn get_all(user: Authentication) -> Json<Vec<String>> {
    Json(vec!["a".to_string(), "b".to_string()])
}
