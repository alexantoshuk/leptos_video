#![recursion_limit = "256"]
pub mod app;

#[cfg_attr(feature = "csr", wasm_bindgen::prelude::wasm_bindgen)]
pub fn hydrate() {
    use crate::app::*;
    console_error_panic_hook::set_once();
    leptos::mount::mount_to_body(App);
}

macro_rules! get {
    ($signal:ident.$field:ident) => {
        $signal.with(|$signal| $signal.$field)
    };
    ($signal:ident) => {
        $signal.get()
    };
}
pub(crate) use get;

macro_rules! get_untracked {
    ($signal:ident.$field:ident($($a:expr),*)) => {
        $signal.with_untracked(|$signal| $signal.$field($($a),*))
    };
    ($signal:ident.$field:ident) => {
        $signal.with_untracked(|$signal| $signal.$field)
    };
    ($signal:ident) => {
        $signal.get_untracked()
    };
}
pub(crate) use get_untracked;
