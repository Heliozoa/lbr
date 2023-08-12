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
pub fn loading_fallback(text: &'static str) -> View {
    view! { <div>{text}</div> }.into_view()
}

/// Generic error fallback view.
pub fn errors_fallback(errors: RwSignal<Errors>) -> View {
    let errors = errors.get_untracked().into_iter().collect::<Vec<_>>();
    if errors.len() == 1 {
        let (_, error) = &errors[0];
        view! {
            <div>{format!("{error}")}</div>
        }
        .into_view()
    } else {
        let errors = errors
            .into_iter()
            .map(|(_, err)| {
                view! { <li>{format!("Error: {err}")}</li> }
            })
            .collect_view();

        view! {
            <div class="content">
                <div>"Errors"</div>
                <ul>
                    {errors}
                </ul>
            </div>
        }
        .into_view()
    }
}

#[macro_export]
macro_rules! logged_in_resource {
    ($($f:tt)*) => {
        $crate::utils::logged_in_resource(
            move |client| async move { client.$($f)*.await }
        )
    };
}

pub fn logged_in_resource<T, A, F>(f: A) -> Resource<Option<bool>, Result<Option<T>, WebError>>
where
    T: Clone + Serialize + DeserializeOwned + 'static,
    A: Fn(Client) -> F + Copy + 'static,
    F: Future<Output = Result<T, WebError>> + 'static,
{
    leptos::create_resource(
        move || context::get_session().logged_in(),
        move |logged_in| {
            let client = context::get_client();
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

pub fn params<T>() -> WebResult<T>
where
    T: Params + Clone + PartialEq + 'static,
{
    leptos_router::use_params()
        .get_untracked()
        .map_err(WebError::from)
}

#[macro_export]
macro_rules! untangle {
    ($resource:ident) => {
        match $resource.read() {
            Some(Ok(Some(res))) => Some(res),
            Some(Ok(None)) => return Ok(None),
            Some(Err(err)) => return Err(err.into()),
            None => None,
        }
    };
}
