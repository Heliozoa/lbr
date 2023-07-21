//! /auth
//! Handlers related to authentication.

use super::prelude::*;
use crate::authentication;
pub use tower_cookies::Cookies;

// handlers

/// Registers the user.
#[instrument]
pub async fn register(
    State(state): State<LbrState>,
    Json(register): Json<req::Register<'static>>,
) -> LbrResult<()> {
    use schema::users as u;

    let req::Register { email, password } = register;
    tokio::task::spawn_blocking(move || {
        let mut conn = state.lbr_pool.get()?;

        let password_hash = authentication::hash_password(&password)?;
        diesel::insert_into(u::table)
            .values(eq!(u, email, password_hash))
            .execute(&mut conn)?;

        EyreResult::Ok(())
    })
    .await??;

    Ok(())
}

/// Logs the user in.
#[instrument]
pub async fn login(
    State(state): State<LbrState>,
    cookies: Cookies,
    Json(login): Json<req::Login<'static>>,
) -> LbrResult<()> {
    use schema::users as u;

    let task_state = state.clone();
    let req::Login { email, password } = login;
    let user_id = tokio::task::spawn_blocking(move || {
        let mut conn = task_state.lbr_pool.get()?;

        let User { id, password_hash } = u::table
            .select(User::as_select())
            .filter(eq!(u, email))
            .get_result(&mut conn)?;
        authentication::verify_password(&password, &password_hash)?;

        EyreResult::Ok(id)
    })
    .await??;

    let signed_cookies = cookies.signed(&state.private_cookie_key);
    authentication::save_session(user_id, signed_cookies, &state.sessions).await?;

    Ok(())
}

/// Fetches the currently logged in user, if any.
#[instrument]
pub async fn current(user: Option<Authentication>) -> LbrResult<Json<Option<i32>>> {
    Ok(Json(user.map(|u| u.user_id)))
}

/// Logs the currently logged in user out.
#[instrument]
pub async fn logout(
    State(state): State<LbrState>,
    cookies: Cookies,
    user: Authentication,
) -> LbrResult<()> {
    let signed_cookies = cookies.signed(&state.private_cookie_key);
    authentication::forget_session(user.session_id, &signed_cookies, &state.sessions).await?;

    Ok(())
}

// queries

query! {
    struct User {
        id: i32 = users::id,
        password_hash: String = users::password_hash,
    }
}
