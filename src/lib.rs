#![recursion_limit = "256"]

pub mod app;
pub mod components;
pub mod timecode;
pub mod utils;

#[cfg(feature = "hydrate")]
#[wasm_bindgen::prelude::wasm_bindgen]
pub fn hydrate() {
    use crate::app::*;
    console_error_panic_hook::set_once();
    leptos::mount::hydrate_body(App);
}

// use js_sys::Function;
// use wasm_bindgen::prelude::*;
// #[wasm_bindgen]
// extern "C" {
//     // JS interop for requestVideoFrameCallback
//     fn request_video_frame_callback(callback: &Function);
// }
