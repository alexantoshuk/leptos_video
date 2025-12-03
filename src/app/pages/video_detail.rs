use crate::app::components::video_player::VideoPlayer;
use crate::app::components::video_player::state::VideoMetadata;
use leptos::prelude::*;
use leptos::{attr::Default, logging::log};
use leptos_meta::{MetaTags, Stylesheet, Title, provide_meta_context};
use leptos_router::{
    StaticSegment,
    components::{Route, Router, Routes},
};

/// Renders the home page of your application.
#[component]
pub fn VideoDetail() -> impl IntoView {
    // Creates a reactive value to update the button
    // let count = RwSignal::new(0);
    // let on_click = move |_| *count.write() += 1;
    let metadata = RwSignal::new(VideoMetadata {
        src: "Metallborne3.mov".into(),
        proxy: Some("Metallborne3_proxy.mp4".into()),
        fps: 25.0,
        aspect_ratio: 1.77,
        ..VideoMetadata::default()
    });
    view! {
        // <h1>"Welcome to Leptos!"</h1>
        // <button on:click=on_click class="font-sans">
        // "Click Me: "
        // {move || {
        // let c = count.get();
        // log!("{c}");
        // c
        // }}
        // </button>

        <div class="p-0 w-full h-dvh">
            // <Video
            // src="https://download.blender.org/peach/bigbuckbunny_movies/big_buck_bunny_1080p_h264.mov"
            // proxy="BigBuckBunny_640x360_proxy.mp4"
            // fps=24.0
            // />

            <VideoPlayer metadata overlay_controls=false />
        </div>
    }
}
