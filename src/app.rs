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
    use_draggable, use_draggable_with_options, use_window, UseDraggableOptions, UseDraggableReturn,
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
        <VideoPlayer src="Metallborne.mp4".to_string() width=720 height=300 />
    }
}
use leptos::*;
use web_sys::js_sys::Object;
use web_sys::wasm_bindgen::convert::OptionIntoWasmAbi;
use web_sys::MouseEvent;

#[component]
pub fn VideoPlayer(
    src: String,
    #[prop(default = 800)] width: u32,
    #[prop(default = 450)] height: u32,
) -> impl IntoView {
    let video_ref = NodeRef::<html::Video>::new();
    let progress_ref = NodeRef::<html::Div>::new();
    let (is_playing, set_is_playing) = signal(false);
    let (progress, set_progress) = signal(0.0);
    let (progress_hover_pos, set_progress_hover_pos) = signal(0);
    let (duration, set_duration) = signal(0.0);
    let (current_time, set_current_time) = signal(0.0);
    let (is_muted, set_is_muted) = signal(false);
    let (volume, set_volume) = signal(1.0);
    let (is_fullscreen, set_is_fullscreen) = signal(false);

    // Calculate seek position from mouse event
    let norm_seek_position = move |client_x: f64| -> f64 {
        if let Some(target) = progress_ref.get() {
            let rect = target.get_bounding_client_rect();
            let pos = (client_x - rect.left()) / rect.width();
            pos.max(0.0).min(1.0)
        } else {
            0.0
        }
    };

    // Handle seek (click or drag)
    let seek_to_position = move |pos: f64| {
        if let Some(video) = video_ref.get() {
            let seek_time = pos * video.duration();
            video.set_current_time(seek_time);
            set_progress.set(pos);
            set_current_time.set(seek_time);
        }
    };

    let UseDraggableReturn {
        x: drag_x,
        is_dragging,
        ..
    } = use_draggable_with_options(
        progress_ref,
        UseDraggableOptions::default()
            .initial_value(Position { x: 0.0, y: 0.0 })
            .target_offset(move |_| {
                let x = progress_hover_pos.get();
                (x as f64, 0.0)
            })
            .on_start(move |ev| {
                let x = progress_hover_pos.get() as f64;
                let pos = norm_seek_position(x);
                seek_to_position(pos);
                true
            })
            .on_move(move |ev| {
                let pos = norm_seek_position(ev.position.x);
                seek_to_position(pos);
            })
            .stop_propagation(true)
            .prevent_default(true),
    );

    let load_metadata = move || {
        if let Some(video) = video_ref.get() {
            let d = video.duration();
            set_duration.set(d);
        }
    };

    let mousemove = move |ev: MouseEvent| {
        set_progress_hover_pos.set(ev.client_x());
    };

    let time_update = move |_| {
        if !is_dragging.get() {
            if let Some(video) = video_ref.get() {
                let time = video.current_time();
                set_current_time.set(time);
                if video.duration() > 0.0 {
                    set_progress.set(time / video.duration());
                }
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
        if let Some(video) = video_ref.get() {
            if !is_fullscreen.get() {
                if let Ok(_) = video.request_fullscreen() {
                    set_is_fullscreen.set(true);
                }
            } else {
                document().exit_fullscreen();
                set_is_fullscreen.set(false);
            }
        }
    };

    // Helper functions
    let format_time = move |time: f64| {
        let minutes = (time / 60.0).floor() as i32;
        let seconds = (time % 60.0).floor() as i32;
        format!("{minutes:02}:{seconds:02}")
    };

    Effect::new(move |_| {
        load_metadata();
        logging::log!("LOAD METADATA");
    });

    view! {
        <div class="relative bg-black overflow-hidden shadow-xl">
            // Video element
            <video
                node_ref=video_ref
                src=src
                width=width.to_string()
                height=height.to_string()
                on:loadedmetadata=move |_| load_metadata()
                on:timeupdate=time_update
                on:click=toggle_play
                preload="metadata"
                class="w-full cursor-pointer"
            />

            // Controls
            <div class="relative bg-gray-900">
                // Progress bar
                <div
                    node_ref=progress_ref
                    class="absolute w-full h-1 expand-clickable-area hover:h-2 hover:-translate-y-1 bg-gray-800 cursor-pointer hover:bg-gray-700 transition-all duration-300"
                    on:mousemove=mousemove
                >
                    <div
                        class="h-full bg-blue-500"
                        style:width=move || format!("{}%", progress.get() * 100.0)
                    />
                </div>

                // Control buttons
                <div class="flex items-center justify-between px-4 py-3">
                    // Left side
                    <div class="flex items-center space-x-4">
                        // Play/Pause button
                        <button
                            on:click=toggle_play
                            class="text-white hover:text-blue-400 transition-colors p-1 rounded"
                        >
                            {move || play_pause_icon(!is_playing.get())}
                        </button>

                        // Time display
                        <div class="flex items-center text-white text-sm font-mono">
                            <span>{move || format_time(current_time.get())}</span>
                            <span class="mx-1 text-gray-400">/</span>
                            <span class="text-gray-400">{move || format_time(duration.get())}</span>
                        </div>
                    </div>

                    // Right side
                    <div class="flex items-center space-x-4">
                        // Volume control
                        <div class="flex items-center">
                            <button
                                on:click=toggle_mute
                                class="text-white hover:text-blue-400 transition-colors p-1 rounded mr-2"
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
                                class="w-16 accent-blue-500 hover:accent-blue-400 transition-colors"
                            />
                        </div>

                        // Fullscreen button
                        <button
                            on:click=toggle_fullscreen
                            class="text-white hover:text-blue-400 transition-colors p-1 rounded"
                        >
                            {move || fullscreen_icon(is_fullscreen.get())}
                        </button>
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
                width="24"
                height="24"
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
        }.into_any()
    } else {
        view! {
            <svg
                class="group-hover:text-emphasis group-hover:dark:text-emphasis-dark transition-colors delay-75 duration-200 ease-in-out"
                width="24"
                height="24"
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
        }.into_any()
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
                stroke-width="1"
                stroke="currentColor"
                fill="none"
                role="graphics-symbol"
            >
                <polygon points="11 5 6 9 2 9 2 15 6 15 11 19 11 5"></polygon>
                <line x1="22" x2="16" y1="9" y2="15"></line>
                <line x1="16" x2="22" y1="9" y2="15"></line>
            </svg>
        }.into_any()
    } else if volume < 0.5 {
        view! {
            <svg
                class="group-hover:text-emphasis group-hover:dark:text-emphasis-dark transition-colors delay-75 duration-200 ease-in-out"
                width="24"
                height="24"
                viewBox="0 0 24 24"
                stroke-linecap="round"
                stroke-linejoin="round"
                stroke-width="1"
                stroke="currentColor"
                fill="none"
                role="graphics-symbol"
            >
                <polygon points="11 5 6 9 2 9 2 15 6 15 11 19 11 5"></polygon>
                <path d="M15.54 8.46a5 5 0 0 1 0 7.07"></path>
            </svg>
        }.into_any()
    } else {
        view! {
            <svg
                class="group-hover:text-emphasis group-hover:dark:text-emphasis-dark transition-colors delay-75 duration-200 ease-in-out"
                width="24"
                height="24"
                viewBox="0 0 24 24"
                stroke-linecap="round"
                stroke-linejoin="round"
                stroke-width="1"
                stroke="currentColor"
                fill="none"
                role="graphics-symbol"
            >
                <polygon points="11 5 6 9 2 9 2 15 6 15 11 19 11 5"></polygon>
                <path d="M15.54 8.46a5 5 0 0 1 0 7.07"></path>
                <path d="M19.07 4.93a10 10 0 0 1 0 14.14"></path>
            </svg>
        }.into_any()
    }
}

fn fullscreen_icon(fullscreen: bool) -> impl IntoView {
    if fullscreen {
        view! {
            <svg
                width="24"
                height="24"
                viewBox="0 0 24 24"
                fill="none"
                focusable="false"
                aria-hidden="true"
            >
                <path
                    fill-rule="evenodd"
                    clip-rule="evenodd"
                    d="M19 5.75a.75.75 0 0 0-.75-.75h-5.833a.75.75 0 0 0 0 1.5H17.5v5.083a.75.75 0 0 0 1.5 0V5.75ZM5 18.25c0 .414.336.75.75.75h5.833a.75.75 0 0 0 0-1.5H6.5v-5.083a.75.75 0 0 0-1.5 0v5.833Z"
                    fill="currentColor"
                ></path>
            </svg>
        }
    } else {
        view! {
            <svg
                width="24"
                height="24"
                viewBox="0 0 24 24"
                fill="none"
                focusable="false"
                aria-hidden="true"
            >
                <path
                    fill-rule="evenodd"
                    clip-rule="evenodd"
                    d="M19 5.75a.75.75 0 0 0-.75-.75h-5.833a.75.75 0 0 0 0 1.5H17.5v5.083a.75.75 0 0 0 1.5 0V5.75ZM5 18.25c0 .414.336.75.75.75h5.833a.75.75 0 0 0 0-1.5H6.5v-5.083a.75.75 0 0 0-1.5 0v5.833Z"
                    fill="currentColor"
                ></path>
            </svg>
        }
    }
}
