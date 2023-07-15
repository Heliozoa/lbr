use crate::authentication::Authentication;
use axum::Json;

pub async fn current(user: Option<Authentication>) -> Json<Option<i32>> {
    Json(user.map(|u| u.user_id))
}
