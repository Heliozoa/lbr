//! Various utility functions.

use crate::{
    context::{self, client::Client},
    error::{WebError, WebResult},
};
pub use crate::{logged_in_resource, untangle};
use leptos::{prelude::*, IntoView, *};
use leptos_router::Params;
use serde::{de::DeserializeOwned, Serialize};
use std::future::Future;

/// Generic loading fallback view.
pub fn loading_fallback(cx: Scope, text: &'static str) -> View {
    view! { cx, <div>{text}</div> }.into_view(cx)
}

/// Generic error fallback view.
pub fn errors_fallback(cx: Scope, errors: RwSignal<Errors>) -> View {
    let errors = errors.get_untracked().into_iter().collect::<Vec<_>>();
    if errors.len() == 1 {
        let (_, error) = &errors[0];
        view! { cx,
            <div>{format!("{error}")}</div>
        }
        .into_view(cx)
    } else {
        let errors = errors
            .into_iter()
            .map(|(_, err)| {
                view! { cx, <li>{format!("Error: {err}")}</li> }
            })
            .collect_view(cx);

        view! { cx,
            <div class="content">
                <div>"Errors"</div>
                <ul>
                    {errors}
                </ul>
            </div>
        }
        .into_view(cx)
    }
}

#[macro_export]
macro_rules! logged_in_resource {
    ($cx:ident, $($f:tt)*) => {
        crate::utils::logged_in_resource(
            $cx,
            move |client| async move { client.$($f)*.await }
        )
    };
}

pub fn logged_in_resource<T, A, F>(
    cx: Scope,
    f: A,
) -> Resource<Option<bool>, Result<Option<T>, WebError>>
where
    T: Clone + Serialize + DeserializeOwned + 'static,
    A: Fn(Client) -> F + Copy + 'static,
    F: Future<Output = Result<T, WebError>> + 'static,
{
    leptos::create_resource(
        cx,
        move || context::get_session(cx).logged_in(),
        move |logged_in| {
            let client = context::get_client(cx);
            async move {
                let data = match logged_in {
                    Some(true) => {
                        let data = f(client).await?;
                        Some(data)
                    }
                    _ => None,
                };
                WebResult::Ok(data)
            }
        },
    )
}

pub fn params<T>(cx: Scope) -> WebResult<T>
where
    T: Params + Clone + PartialEq + 'static,
{
    leptos_router::use_params(cx)
        .get_untracked()
        .map_err(WebError::from)
}

#[macro_export]
macro_rules! untangle {
    ($cx:ident, $resource:ident) => {
        match $resource.read($cx) {
            Some(Ok(Some(res))) => Some(res),
            Some(Ok(None)) => return Ok(None),
            Some(Err(err)) => return Err(err.into()),
            None => None,
        }
    };
}
