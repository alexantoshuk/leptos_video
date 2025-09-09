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
    use_debounce_fn, use_draggable_with_options, use_element_size, use_mouse_in_element,
    use_timeout_fn, UseDraggableOptions,
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
        <script>0</script>
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

#[derive(Debug, Clone, PartialEq, Eq)]
enum Dragging {
    Start,
    Move,
    None,
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

        <div style="  width:400px; height:400px;">
            // <VideoPlayer src="https://download.blender.org/peach/bigbuckbunny_movies/BigBuckBunny_640x360.m4v"
            // .to_string()  fps=25.0 />

            <VideoPlayer src="Metallborne2.mp4".to_string() fps=25.0 />
        </div>
    }
}
use leptos::*;
use web_sys::js_sys::Object;
use web_sys::wasm_bindgen::convert::OptionIntoWasmAbi;
use web_sys::MouseEvent;

#[component]
pub fn VideoPlayer(
    #[prop(into)] src: Signal<String>,
    #[prop(into)] fps: Signal<f64>,
) -> impl IntoView {
    let container_ref = NodeRef::<html::Div>::new();
    let video_ref = NodeRef::<html::Video>::new();
    let progress_ref = NodeRef::<html::Div>::new();
    let (is_playing, set_is_playing) = signal(false);
    let (is_dragging, set_is_dragging) = signal(Dragging::None);
    let (progress, set_progress) = signal(0.0);
    let (frame, set_frame) = signal(0.0);
    let (preload_progress, set_preload_progress) = signal(0.0);
    let (duration, set_duration) = signal(0.0);
    let (controls_visible, set_controls_visible) = signal(false);
    let (current_time, set_current_time) = signal(0.0);
    let (is_muted, set_is_muted) = signal(false);
    let (volume, set_volume) = signal(1.0);
    let (is_fullscreen, set_is_fullscreen) = signal(false);
    let (drag_offset, set_drag_offset) = signal(0.0);

    let container_mouse = use_mouse_in_element(container_ref);
    let total_frames = move || duration.get() * fps.get();

    let controls_hide_after_delay = use_debounce_fn(
        move || {
            set_controls_visible.set(false);
        },
        2000.0, // 2 seconds
    );

    let seek_to_position = move |pos: f64| {
        if let Some(video) = video_ref.get() {
            let d = duration.get();
            if d <= 0.0 {
                return;
            }

            let time = pos * d;
            video.set_current_time(time);

            set_current_time.set(time);
            let pos = discretize_progress(pos, d * fps.get());
            set_progress.set(pos);
        }
    };

    let time_update = move |_| {
        if is_dragging.get() == Dragging::None {
            if let Some(video) = video_ref.get() {
                let d = duration.get();
                if d <= 0.0 {
                    return;
                }

                let time = video.current_time();
                let pos = time / d;

                set_current_time.set(time);
                let pos = discretize_progress(pos, d * fps.get());
                set_progress.set(pos);
            }
        }
    };

    let stop = move || {
        if let Some(video) = video_ref.get() {
            video.set_current_time(0.0);
        }
        set_current_time.set(0.0);
        set_progress.set(0.0);
    };

    let load_metadata = move || {
        if let Some(video) = video_ref.get() {
            let d = video.duration();
            if d.is_finite() {
                set_duration.set(d);
            }
        }
    };

    let preload_update = move || {
        // log!("preload");
        if let Some(video) = video_ref.get() {
            let vb = video.buffered();
            let t = current_time.get();
            for i in (0..vb.length()).rev() {
                let start = vb.start(i).unwrap();
                let end = vb.end(i).unwrap();
                // log!("buffered {i} {start}-{end} {t}");
                if t >= start && t <= end {
                    set_preload_progress.set(end / duration.get());
                    break;
                }
            }
        }
    };

    let is_ended = move || {
        let n = total_frames();
        let f = cur_frame(progress.get(), n);
        f == n - 1.0
    };

    use_draggable_with_options(
        progress_ref,
        UseDraggableOptions::default()
            .initial_value(Position { x: 0.0, y: 0.0 })
            .target_offset(move |ev| (0.0, 0.0))
            .on_start(move |ev| {
                if let Some(p) = progress_ref.get() {
                    set_is_dragging.set(Dragging::Start);

                    if ev.event.pointer_type() == "touch" {
                        let _ = p.focus();
                    }

                    let x = ev.event.offset_x() as f64;
                    set_drag_offset.set(x);
                    let pos = (x / p.client_width() as f64).max(0.0).min(1.0);
                    seek_to_position(pos);
                    true
                } else {
                    set_is_dragging.set(Dragging::None);
                    false
                }
            })
            .on_end(move |_| {
                set_is_dragging.set(Dragging::None);
            })
            .on_move(move |ev| {
                if let Some(p) = progress_ref.get() {
                    set_is_dragging.set(Dragging::Move);
                    let x = ev.position.x + drag_offset.get();
                    let pos = (x / p.client_width() as f64).max(0.0).min(1.0);
                    set_controls_visible.set(true);
                    seek_to_position(pos);
                }
            })
            // .stop_propagation(true)
            .prevent_default(true),
    );

    let toggle_play = move |_| {
        if let Some(video) = video_ref.get() {
            if is_playing.get() {
                let _ = video.pause();
                set_is_playing.set(false);
            } else {
                if is_ended() {
                    stop();
                }
                let _ = video.play();
                set_is_playing.set(true);
            }
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
        if let Some(el) = container_ref.get() {
            if is_fullscreen.get() {
                document().exit_fullscreen();
            } else {
                el.request_fullscreen();
            }
        }
    };

    let fullscreenchange = move |_| {
        if let Some(el) = container_ref.get() {
            set_is_fullscreen.set(document().fullscreen_element() == Some(el.into()));
        }
    };

    Effect::new(move |_| {
        load_metadata();
    });

    // Show button on mouse movement and reset hide timer
    Effect::new(move |_| {
        let _ = container_mouse.x.get(); // Track mouse movement
        let _ = container_mouse.y.get();
        set_controls_visible.set(true);
        controls_hide_after_delay();
    });

    view! {
        <div
            node_ref=container_ref
            class="size-full flex bg-black flex-col overflow-hidden shadow-xl touch-none group"
            on:fullscreenchange=fullscreenchange
        >
            // Video element
            <div class="relative flex-auto m-[1px] group-fullscreen:m-0">
                <video
                    // controls
                    playsinline
                    disablepictureinpicture
                    controlslist="nodownload"
                    node_ref=video_ref
                    src=src
                    // preload="none"
                    class="cursor-pointer absolute size-full object-contain"
                    on:contextmenu=move |ev| ev.prevent_default()
                    on:loadedmetadata=move |m| {
                        log!("{:?}",m);
                        load_metadata()
                    }
                    on:durationchange=move |_| load_metadata()
                    on:timeupdate=time_update
                    on:click=toggle_play
                    on:progress=move |_| preload_update()
                    on:canplaythrough=move |_| preload_update()
                    on:ratechange=move |_| log!("ratechange")
                    on:ended=move |_| {
                        // log!("ended");
                        set_is_playing.set(false)
                    }
                >
                    "Your browser doesn't support HTML video."
                </video>
            </div>

            // Controls
            <div
                tabindex="-1"
                class=move || {
                    format!(
                        "flex-none not-group-fullscreen:bg-gray-900 bottom-0 px-2 group-fullscreen:absolute group-fullscreen:bg-gradient-to-t group-fullscreen:from-black group-fullscreen:inset-x-0 group-fullscreen:w-full group-fullscreen:pt-2 group-fullscreen:px-6 transition-opacity duration-200 {} focus:opacity-100 hover:opacity-100",
                        if is_fullscreen.get() && !controls_visible.get() {
                            "opacity-0"
                        } else {
                            "opacity-100"
                        },
                    )
                }
            >
                <div class="relative">
                    // Progress bar
                    <div
                        node_ref=progress_ref
                        tabindex="-1"
                        class="absolute outline-none group/progress origin-bottom w-full h-1 expand-clickable-area hover:scale-y-200 focus:scale-y-200 bg-gray-600 group-fullscreen:bg-white/20 cursor-pointer transform transition-all duration-200"
                    >
                        <div
                            class="absolute origin-left h-full w-full bg-white/20 transition-scale duration-200 pointer-events-none"
                            style:transform=move || {
                                format!("scaleX({})", preload_progress.get())
                            }
                        />

                        <div
                            class="absolute origin-left h-full w-full bg-blue-500 pointer-events-none"
                            style:transform=move || { format!("scaleX({})", progress.get()) }
                        />

                        <div
                            class="absolute origin-left h-full w-full bg-blue-300 pointer-events-none"

                            style:transform=move || {
                                format!("translateX({}%) scaleX({})", progress.get()*100.0, 1.0 / total_frames())
                            }

                        />
                    </div>

                    // Control buttons
                    <div class="flex items-center justify-between px-1 pb-2 pt-4 bottom-0">
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
                                <span>
                                    {move || timecode(
                                        cur_frame(progress.get(), duration.get() * fps.get()),
                                        fps.get(),
                                    )}
                                </span>
                                <span class="mx-1 text-gray-400">/</span>
                                <span class="text-gray-400">
                                    {move || timecode(
                                        cur_frame(progress.get(), duration.get() * fps.get()),
                                        fps.get(),
                                    )}
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
        }.into_any()
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
                stroke-width="1.5"
                stroke="currentColor"
                fill="none"
                role="graphics-symbol"
            >
                <polygon points="11 5 6 9 2 9 2 15 6 15 11 19 11 5"></polygon>
                <line x1="23" y1="9" x2="17" y2="15"></line>
                <line x1="17" y1="9" x2="23" y2="15"></line>
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
                stroke-width="1.5"
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
                stroke-width="1.5"
                stroke="currentColor"
                fill="none"
                role="graphics-symbol"
            >
                <polygon points="11 5 6 9 2 9 2 15 6 15 11 19 11 5"></polygon>
                <path d="M19.07 4.93a10 10 0 0 1 0 14.14M15.54 8.46a5 5 0 0 1 0 7.07"></path>
            </svg>
        }.into_any()
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
        }.into_any()
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
        }.into_any()
    }
}

fn cur_frame(progress: f64, total_frames: f64) -> f64 {
    f64::floor(progress * total_frames)
}

fn discretize_progress(progress: f64, total_frames: f64) -> f64 {
    if total_frames < 1.0 {
        0.0
    } else {
        let last_frame = total_frames - 1.0;
        let f = cur_frame(progress, total_frames);
        f.min(last_frame) / total_frames
    }
}

fn timecode(frame: f64, fps: f64) -> String {
    let time = frame / fps;
    let hours = (time / 360.0).floor() as i32;
    let minutes = (time / 60.0).floor() as i32;
    let seconds = (time % 60.0).floor() as i32;
    let frame = (frame % fps) as i32;
    let pad = (fps as i32).to_string().len();
    format!("{hours:02}:{minutes:02}:{seconds:02}:{frame:0>pad$}")
}

fn format_time(time: f64) -> String {
    let minutes = (time / 60.0).floor() as i32;
    let seconds = (time % 60.0).floor() as i32;
    format!("{minutes:02}:{seconds:02}")
}
