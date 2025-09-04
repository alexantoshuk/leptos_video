use std::any::Any;

use leptos::ev::{DragEvent, Event};
use leptos::logging::log;
use leptos::prelude::*;
use leptos::wasm_bindgen::JsCast;
use leptos_meta::{provide_meta_context, MetaTags, Stylesheet, Title};
use leptos_router::{
    components::{Route, Router, Routes},
    StaticSegment,
};
use leptos_use::core::Position;
use leptos_use::{
    use_debounce_fn, use_draggable_with_options, use_mouse_in_element, use_timeout_fn,
    UseDraggableOptions, UseDraggableReturn,
};
use web_sys;

pub fn shell(options: LeptosOptions) -> impl IntoView {
    view! {
        <!DOCTYPE html>
        <html lang="en">
            <head>
                <meta charset="utf-8" />
                <meta name="viewport" content="width=device-width, initial-scale=1" />
                <AutoReload options=options.clone() />
                <HydrationScripts options />
                <MetaTags />
            </head>
            <body>
                <App />
            </body>
        </html>
    }
}

#[component]
pub fn App() -> impl IntoView {
    // Provides context that manages stylesheets, titles, meta tags, etc.
    provide_meta_context();

    view! {
        // injects a stylesheet into the document <head>
        // id=leptos means cargo-leptos will hot-reload this stylesheet
        <Stylesheet id="leptos" href="/pkg/leptos_video.css" />

        // sets the document title
        <Title text="Welcome  to Leptos" />

        // content for this welcome page
        <Router>
            <main>
                <Routes fallback=|| "Page not found.".into_view()>
                    <Route path=StaticSegment("") view=HomePage />
                </Routes>
            </main>
        </Router>
    }
}

/// Renders the home page of your application.
#[component]
fn HomePage() -> impl IntoView {
    // Creates a reactive value to update the button
    let count = RwSignal::new(0);
    let on_click = move |_| *count.write() += 1;

    view! {
        <h1>"Welcome to Leptos!"</h1>
        <button on:click=on_click>"Click Me: " {count}</button>

        <div class="p-1" style="width:400px; height:400px;">
            <VideoPlayer src="https://download.blender.org/peach/bigbuckbunny_movies/BigBuckBunny_640x360.m4v"
                .to_string() />
        // <VideoPlayer src="https://www.pexels.com/ru-ru/download/video/3150562/".to_string() />

        // <VideoPlayer src="SoftSwissPost_1080x1350.v010.mp4".to_string() />
        </div>
    }
}
use leptos::*;
use web_sys::js_sys::Object;
use web_sys::wasm_bindgen::convert::OptionIntoWasmAbi;
use web_sys::MouseEvent;

#[component]
pub fn VideoPlayer(src: String) -> impl IntoView {
    let video_container_ref = NodeRef::<html::Div>::new();
    let video_ref = NodeRef::<html::Video>::new();
    let progress_ref = NodeRef::<html::Div>::new();
    let (is_playing, set_is_playing) = signal(false);
    let (progress, set_progress) = signal(0.0);
    let (preload_progress, set_preload_progress) = signal(0.0);
    let (hover_progress, set_hover_progress) = signal(0.0);
    let (duration, set_duration) = signal(0.0);
    let (controls_visible, set_controls_visible) = signal(false);
    let (info, set_info) = signal(0.0);
    let (current_time, set_current_time) = signal(0.0);
    let (is_muted, set_is_muted) = signal(false);
    let (volume, set_volume) = signal(1.0);
    let (is_fullscreen, set_is_fullscreen) = signal(false);
    let (drag_offset, set_drag_offset) = signal(0.0);

    let video_container_mouse = use_mouse_in_element(video_container_ref);
    // let progress_mouse = use_mouse_in_element(progress_ref);

    // Calculate seek position from mouse event
    let norm_pos = move |client_x: f64| -> f64 {
        if let Some(target) = progress_ref.get() {
            let rect = target.get_bounding_client_rect();
            let pos = (client_x - rect.left()) / rect.width();
            pos.max(0.0).min(1.0)
        } else {
            0.0
        }
    };

    let controls_hide = use_debounce_fn(
        move || {
            set_controls_visible.set(false);
        },
        2000.0, // 2 seconds
    );

    // Handle seek (click or drag)
    let seek_to_position = move |pos: f64| {
        if let Some(video) = video_ref.get() {
            let seek_time = pos * video.duration();
            video.set_current_time(seek_time);
            set_progress.set(pos);
            set_current_time.set(seek_time);
        }
    };

    let UseDraggableReturn { is_dragging, .. } = use_draggable_with_options(
        progress_ref,
        UseDraggableOptions::default()
            .initial_value(Position { x: 0.0, y: 0.0 })
            .target_offset(move |ev| (0.0, 0.0))
            .on_start(move |ev| {
                if ev.event.pointer_type() == "touch" {
                    if let Some(p) = progress_ref.get() {
                        let _ = p.focus();
                    }
                }
                let x = ev.event.x() as f64;
                set_drag_offset.set(x);
                let pos = norm_pos(x);
                let pos = norm_pos(x);
                seek_to_position(pos);
                true
            })
            .on_move(move |ev| {
                let pos = norm_pos(ev.position.x + drag_offset.get());
                seek_to_position(pos);
            })
            // .stop_propagation(true)
            .prevent_default(true),
    );

    let load_metadata = move || {
        if let Some(video) = video_ref.get() {
            let d = video.duration();
            set_duration.set(d);
        }
    };

    let time_update = move |_| {
        if !is_dragging.get() {
            if let Some(video) = video_ref.get() {
                let d = video.duration();
                let time = video.current_time();
                set_current_time.set(time);
                if d > 0.0 {
                    set_progress.set(time / d);
                }
            }
        }
    };

    let preload_update = move |_| {
        if let Some(video) = video_ref.get() {
            let n = video.buffered().length();
            if n == 0 {
                return;
            }
            let n = n - 1;

            if let Ok(loaded) = video.buffered().end(n) {
                set_preload_progress.set(loaded / duration.get())
            }
        }
    };

    let toggle_play = move |_| {
        if let Some(video) = video_ref.get() {
            if is_playing.get() {
                let _ = video.pause();
            } else {
                let _ = video.play();
            }
            set_is_playing.update(|p| *p = !*p);
        }
    };

    let change_volume = move |ev: Event| {
        ev.stop_propagation();
        if let Some(video) = video_ref.get() {
            let target = event_target::<web_sys::HtmlInputElement>(&ev);
            let vol = target.value_as_number();
            set_volume.set(vol);
            video.set_volume(vol);
            set_is_muted.set(vol == 0.0);
        }
    };

    let toggle_mute = move |ev: MouseEvent| {
        ev.stop_propagation();
        let muted = !is_muted.get();
        set_is_muted.set(muted);
        if let Some(video) = video_ref.get() {
            if muted {
                set_volume.set(0.0);
            } else {
                let vol = video.volume();
                let vol = if vol == 0.0 { 1.0 } else { vol };
                set_volume.set(vol);
            }
            video.set_muted(muted);
        }
    };

    let toggle_fullscreen = move |ev: MouseEvent| {
        ev.stop_propagation();
        if let Some(el) = video_container_ref.get() {
            if is_fullscreen.get() {
                document().exit_fullscreen();
            } else {
                el.request_fullscreen();
            }
        }
    };

    let fullscreenchange = move |_| {
        if let Some(el) = video_container_ref.get() {
            set_is_fullscreen.set(document().fullscreen_element() == Some(el.into()));
        }
    };

    // Helper functions
    let format_time = move |time: f64| {
        let minutes = (time / 60.0).floor() as i32;
        let seconds = (time % 60.0).floor() as i32;
        format!("{minutes:02}:{seconds:02}")
    };

    create_effect(move |_| {
        load_metadata();
    });

    // Show button on mouse movement and reset hide timer
    create_effect(move |_| {
        let _ = video_container_mouse.x.get(); // Track mouse movement
        let _ = video_container_mouse.y.get();

        set_controls_visible.set(true);
        controls_hide();
    });

    view! {
        <div
            node_ref=video_container_ref
            class="size-full flex bg-black flex-col overflow-hidden shadow-xl touch-none group"
            on:fullscreenchange=fullscreenchange
        >
            // Video element
            <div class="relative flex-auto m-[1px] group-fullscreen:m-0">
                <video
                    // controls
                    playsinline
                    disablepictureinpicture
                    node_ref=video_ref
                    src=src
                    preload="auto"
                    class="cursor-pointer absolute size-full object-contain"
                    on:loadedmetadata=move |_| load_metadata()
                    on:durationchange=move |_| load_metadata()
                    on:timeupdate=time_update
                    on:click=toggle_play
                    on:progress=preload_update
                >
                    "Your browser doesn't support HTML video."
                </video>
            </div>

            // Controls
            <div class=move || {
                format!(
                    "flex-none bg-gray-900 bottom-0 group-fullscreen:absolute group-fullscreen:backdrop-blur-md group-fullscreen:bg-black/70 group-fullscreen:inset-x-0 group-fullscreen:w-full group-fullscreen:pt-2 px-2 transition-opacity duration-200 {} hover:opacity-100",
                    if is_fullscreen.get() && !controls_visible.get() {
                        "opacity-0"
                    } else {
                        "opacity-100"
                    },
                )
            }>
                <div class="relative">
                    // Progress bar
                    <div
                        node_ref=progress_ref
                        tabindex="-1"
                        class="absolute outline-none group/progress origin-bottom w-full h-1 expand-clickable-area hover:scale-y-200 focus:scale-y-200 bg-gray-600 group-fullscreen:bg-white/20 cursor-pointer transform transition-all duration-200"
                    >
                        <div
                            class=move || {
                                format!(
                                    "absolute origin-left h-full w-full bg-white/20 opacity-0 transition-opacity duration-200 {}",
                                    if is_dragging.get() {
                                        ""
                                    } else {
                                        "group-hover/progress:opacity-100"
                                    },
                                )
                            }
                            style:transform=move || { format!("scaleX({})", hover_progress.get()) }
                            on:mousemove=move |ev| {
                                set_hover_progress.set(norm_pos(ev.client_x() as f64))
                            }
                        />
                        <div
                            class="absolute origin-left h-full w-full bg-white/20"
                            style:transform=move || {
                                format!("scaleX({})", preload_progress.get())
                            }
                        />
                        <div
                            class="absolute origin-left h-full w-full bg-blue-500"
                            style:transform=move || { format!("scaleX({})", progress.get()) }
                        />
                    </div>

                    // Control buttons
                    <div class="flex items-center justify-between px-4 py-3">
                        // Left side
                        <div class="flex items-center space-x-4">
                            // Play/Pause button
                            <button
                                on:click=toggle_play
                                class="text-white hover:text-blue-400 transition-colors p-1 rounded cursor-pointer"
                            >
                                {move || play_pause_icon(!is_playing.get())}
                            </button>

                            // Time display
                            <div class="flex items-center text-white text-sm font-mono">
                                <span>{move || format_time(current_time.get())}</span>
                                <span class="mx-1 text-gray-400">/</span>
                                <span class="text-gray-400">
                                    {move || format_time(duration.get())}
                                </span>
                            </div>
                        </div>

                        // Right side
                        <div class="flex items-center space-x-4">
                            // Volume control
                            <div class="flex items-center">
                                <button
                                    on:click=toggle_mute
                                    class="text-white hover:text-blue-400 transition-colors p-1 rounded mr-2 cursor-pointer"
                                >
                                    {move || volume_icon(volume.get())}
                                </button>
                                <input
                                    type="range"
                                    min="0.0"
                                    max="1.0"
                                    step="0.01"
                                    prop:value=move || volume.get()
                                    on:input=change_volume
                                    class="appearance-none w-16 text-blue-500"
                                />
                            </div>

                            // Fullscreen button
                            <button
                                on:click=toggle_fullscreen
                                class="text-white hover:text-blue-400 transition-colors p-1 rounded cursor-pointer"
                            >
                                {move || fullscreen_icon(is_fullscreen.get())}
                            </button>
                        </div>
                    </div>
                </div>
            </div>
        </div>
    }
}

fn play_pause_icon(play: bool) -> impl IntoView {
    if play {
        view! {
            <svg
                class="group-hover:text-emphasis group-hover:dark:text-emphasis-dark transition-colors delay-75 duration-200 ease-in-out"
                width="22"
                height="22"
                viewBox="0 0 24 24"
                stroke-linecap="round"
                stroke-linejoin="round"
                stroke-width="2"
                stroke="currentColor"
                fill="none"
                role="graphics-symbol"
            >
                <path stroke="none" d="M0 0h24v24H0z" fill="none"></path>
                <path
                    d="M6 4v16a1 1 0 0 0 1.524 .852l13 -8a1 1 0 0 0 0 -1.704l-13 -8a1 1 0 0 0 -1.524 .852z"
                    stroke-width="0"
                    fill="currentColor"
                ></path>
            </svg>
        }
    } else {
        view! {
            <svg
                class="group-hover:text-emphasis group-hover:dark:text-emphasis-dark transition-colors delay-75 duration-200 ease-in-out"
                width="22"
                height="22"
                viewBox="0 0 24 24"
                stroke-linecap="round"
                stroke-linejoin="round"
                stroke-width="2"
                stroke="currentColor"
                fill="none"
                role="graphics-symbol"
            >
                <path stroke="none" d="M0 0h24v24H0z" fill="none"></path>
                <path
                    d="M9 4h-2a2 2 0 0 0 -2 2v12a2 2 0 0 0 2 2h2a2 2 0 0 0 2 -2v-12a2 2 0 0 0 -2 -2z"
                    stroke-width="0"
                    fill="currentColor"
                ></path>
                <path
                    d="M17 4h-2a2 2 0 0 0 -2 2v12a2 2 0 0 0 2 2h2a2 2 0 0 0 2 -2v-12a2 2 0 0 0 -2 -2z"
                    stroke-width="0"
                    fill="currentColor"
                ></path>
            </svg>
        }
    }
}

fn volume_icon(volume: f64) -> impl IntoView {
    if volume <= 0.0 {
        view! {
            <svg
                class="group-hover:text-emphasis group-hover:dark:text-emphasis-dark transition-colors delay-75 duration-200 ease-in-out"
                width="24"
                height="24"
                viewBox="0 0 24 24"
                stroke-linecap="round"
                stroke-linejoin="round"
                stroke-width="1.5"
                stroke="currentColor"
                fill="none"
                role="graphics-symbol"
            >
                <polygon points="11 5 6 9 2 9 2 15 6 15 11 19 11 5"></polygon>
                <line x1="23" y1="9" x2="17" y2="15"></line>
                <line x1="17" y1="9" x2="23" y2="15"></line>
            </svg>
        }
    } else if volume < 0.5 {
        view! {
            <svg
                class="group-hover:text-emphasis group-hover:dark:text-emphasis-dark transition-colors delay-75 duration-200 ease-in-out"
                width="24"
                height="24"
                viewBox="0 0 24 24"
                stroke-linecap="round"
                stroke-linejoin="round"
                stroke-width="1.5"
                stroke="currentColor"
                fill="none"
                role="graphics-symbol"
            >
                <polygon points="11 5 6 9 2 9 2 15 6 15 11 19 11 5"></polygon>
                <path d="M15.54 8.46a5 5 0 0 1 0 7.07"></path>
            </svg>
        }
    } else {
        view! {
            <svg
                class="group-hover:text-emphasis group-hover:dark:text-emphasis-dark transition-colors delay-75 duration-200 ease-in-out"
                width="24"
                height="24"
                viewBox="0 0 24 24"
                stroke-linecap="round"
                stroke-linejoin="round"
                stroke-width="1.5"
                stroke="currentColor"
                fill="none"
                role="graphics-symbol"
            >
                <polygon points="11 5 6 9 2 9 2 15 6 15 11 19 11 5"></polygon>
                <path d="M19.07 4.93a10 10 0 0 1 0 14.14M15.54 8.46a5 5 0 0 1 0 7.07"></path>
            </svg>
        }
    }
}

fn fullscreen_icon(fullscreen: bool) -> impl IntoView {
    if fullscreen {
        view! {
            <svg
                class="group-hover:text-emphasis group-hover:dark:text-emphasis-dark transition-colors delay-75 duration-200 ease-in-out"
                width="18"
                height="18"
                viewBox="0 0 24 24"
                stroke-linecap="round"
                stroke-linejoin="round"
                stroke-width="2"
                stroke="currentColor"
                fill="none"
                role="graphics-symbol"
            >
                <polyline points="4 14 10 14 10 20"></polyline>
                <polyline points="20 10 14 10 14 4"></polyline>
                <line x1="14" y1="10" x2="21" y2="3"></line>
                <line x1="3" y1="21" x2="10" y2="14"></line>
            </svg>
        }
    } else {
        view! {
            <svg
                class="group-hover:text-emphasis group-hover:dark:text-emphasis-dark transition-colors delay-75 duration-200 ease-in-out"
                width="18"
                height="18"
                viewBox="0 0 24 24"
                stroke-linecap="round"
                stroke-linejoin="round"
                stroke-width="2"
                stroke="currentColor"
                fill="none"
                role="graphics-symbol"
            >
                <polyline points="15 3 21 3 21 9"></polyline>
                <polyline points="9 21 3 21 3 15"></polyline>
                <line x1="21" y1="3" x2="14" y2="10"></line>
                <line x1="3" y1="21" x2="10" y2="14"></line>
            </svg>
        }
    }
}
