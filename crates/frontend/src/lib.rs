use lbr_web::Root;
use leptos::*;
use tracing::Level;
use tracing_wasm::WASMLayerConfigBuilder;
use wasm_bindgen::prelude::wasm_bindgen;

#[wasm_bindgen]
pub fn hydrate() {
    console_error_panic_hook::set_once();

    let wasm_log = option_env!("WASM_LOG")
        .and_then(|var| var.parse().ok())
        .unwrap_or(Level::INFO);
    tracing_wasm::set_as_global_default_with_config(
        WASMLayerConfigBuilder::default()
            .set_max_level(wasm_log)
            .build(),
    );

    let backend_addr = option_env!("LBR_BACKEND_ADDRESS").unwrap_or("http://localhost:3000");

    tracing::debug!(
        "initialising (backend at {}, logging level {})",
        backend_addr,
        wasm_log
    );

    leptos::mount_to_body(move |cx| {
        lbr_web::context::initialise_context(cx, backend_addr);
        view! { cx, <Root/> }
    });
}
