//! /auth

use crate::{
    authentication::{self, Authentication},
    eq,
    error::{EyreResult, LbrResult},
    LbrState,
};
use axum::Json;
use diesel::prelude::*;
use lbr_api::request as req;
use tower_cookies::Cookies;

pub async fn register(state: LbrState, register: Json<req::Register<'static>>) -> LbrResult<()> {
    use crate::schema::users as u;
    tracing::info!("Registering");

    tokio::task::spawn_blocking(move || {
        let mut conn = state.lbr_pool.get()?;
        let email = &register.email;
        let password_hash = authentication::hash_password(&register.password)?;
        diesel::insert_into(u::table)
            .values(eq!(u, email, password_hash))
            .execute(&mut conn)?;
        EyreResult::Ok(())
    })
    .await??;

    Ok(())
}

pub async fn login(
    state: LbrState,
    cookies: Cookies,
    login: Json<req::Login<'static>>,
) -> LbrResult<()> {
    use crate::schema::users as u;
    tracing::info!("Logging in");

    let task_state = state.clone();
    let user_id = tokio::task::spawn_blocking(move || {
        let mut conn = task_state.lbr_pool.get()?;
        let email = &login.email;
        let (id, password_hash) = u::table
            .select((u::id, u::password_hash))
            .filter(eq!(u, email))
            .get_result::<(i32, String)>(&mut conn)?;
        authentication::verify_password(&login.password, &password_hash)?;
        EyreResult::Ok(id)
    })
    .await??;

    let signed_cookies = cookies.signed(&state.private_cookie_key);
    authentication::save_session(user_id, signed_cookies, &state.sessions).await?;

    Ok(())
}

pub async fn current(user: Option<Authentication>) -> LbrResult<Json<Option<i32>>> {
    Ok(Json(user.map(|u| u.user_id)))
}

pub async fn logout(state: LbrState, cookies: Cookies, user: Authentication) -> LbrResult<()> {
    tracing::info!("Logging out");

    let signed_cookies = cookies.signed(&state.private_cookie_key);
    authentication::forget_session(user.session_id, &signed_cookies, &state.sessions).await?;

    Ok(())
}
