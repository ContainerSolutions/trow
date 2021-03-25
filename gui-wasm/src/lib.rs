#![recursion_limit = "1024"]

cfg_if::cfg_if! {
    if #[cfg(feature = "wee_alloc")] {
        #[global_allocator]
        static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;
    }
}

const REGISTRY_KEY: &str = "trow.gui.registry_url";
const AUTH_TOKEN_KEY: &str = "trow.gui.auth_token";
const DEFAULT_REGISTRY_URL: &str = "https://0.0.0.0:8443";

use wasm_bindgen::prelude::*;
use yew::prelude::*;

mod app;
mod components;
mod error;
mod services;
mod switch;
mod types;

#[wasm_bindgen(start)]
pub fn run_app() {
    wasm_logger::init(wasm_logger::Config::default());
    App::<app::Model>::new().mount_to_body();
}