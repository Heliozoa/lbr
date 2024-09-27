use lbr_web::App;
use tracing::Level;
use tracing_subscriber::{fmt::format::Pretty, prelude::*};
use tracing_web::{performance_layer, MakeWebConsoleWriter};
use wasm_bindgen::prelude::wasm_bindgen;

#[wasm_bindgen]
pub fn hydrate() {
    console_error_panic_hook::set_once();

    let wasm_log = option_env!("WASM_LOG")
        .and_then(|var| var.parse().ok())
        .unwrap_or(Level::INFO);
    let writer = MakeWebConsoleWriter::new().with_max_level(wasm_log);
    let fmt_layer = tracing_subscriber::fmt::layer()
        .with_ansi(false)
        .without_time()
        .with_writer(writer);
    let perf_layer = performance_layer().with_details_from_fields(Pretty::default());
    tracing_subscriber::registry()
        .with(fmt_layer)
        .with(perf_layer)
        .init();

    tracing::info!("Hydrating, logging level `{wasm_log}`");
    leptos::mount::hydrate_body(App);
}
