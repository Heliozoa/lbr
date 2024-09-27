//! Various utility functions.

use crate::{
    context::{self, client::Client},
    error::{WebError, WebResult},
};
pub use crate::{logged_in_resource, untangle};
use leptos::{prelude::*, IntoView};
use leptos_router::params::Params;
use serde::{de::DeserializeOwned, Serialize};
use std::{fmt::Debug, future::Future};

/// Generic loading fallback view.
pub fn loading_fallback(text: &'static str) -> impl IntoView {
    view! { <div>{text}</div> }.into_view()
}

/// Generic error fallback view.
pub fn errors_fallback(errors: ArcRwSignal<Errors>) -> impl IntoView {
    let errors = errors.get_untracked().into_iter().collect::<Vec<_>>();
    if errors.len() == 1 {
        let (_, error) = &errors[0];
        view! {
            <div>{format!("{error}")}</div>
        }
        .into_any()
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
        .into_any()
    }
}

#[macro_export]
macro_rules! logged_in_resource {
    ($($f:tt)*) => {
        $crate::utils::logged_in_resource(
            move |client| async move { send_wrapper::SendWrapper::new(client.$($f)*).await }
        )
    };
}

pub fn logged_in_resource<T, A, F>(f: A) -> Resource<Result<Option<T>, WebError>>
where
    T: Debug + Clone + Serialize + DeserializeOwned + 'static + Send + Sync,
    A: Fn(Client) -> F + Copy + 'static + Send + Sync,
    F: Future<Output = Result<T, WebError>> + 'static + Send + Sync,
{
    Resource::new(
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
    T: Params + Clone + PartialEq + 'static + Send + Sync,
{
    leptos_router::hooks::use_params()
        .get()
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
