use leptos::prelude::*;
use leptos_video::App;

fn main() {
    console_error_panic_hook::set_once();
    // set up logging
    _ = console_log::init_with_level(log::Level::Debug);

    mount_to_body(App);
}
