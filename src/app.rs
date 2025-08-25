// use icondata as ico;
use leptos::ev::Event;
use leptos::prelude::*;
// use leptos_icons::Icon;
use leptos_meta::{provide_meta_context, MetaTags, Stylesheet, Title};
use leptos_router::{
    components::{Route, Router, Routes},
    StaticSegment,
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
        <Title text="Welcome to Leptos" />

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
use web_sys::MouseEvent;

#[component]
pub fn VideoPlayer(
    src: String,
    #[prop(default = 800)] width: u32,
    #[prop(default = 450)] height: u32,
) -> impl IntoView {
    let video_ref: NodeRef<html::Video> = NodeRef::new();
    let (is_playing, set_is_playing) = signal(false);
    let (progress, set_progress) = signal(0.0);
    let (duration, set_duration) = signal(0.0);
    let (current_time, set_current_time) = signal(0.0);
    let (is_muted, set_is_muted) = signal(false);
    let (volume, set_volume) = signal(0.7);
    let (is_fullscreen, set_is_fullscreen) = signal(false);

    // Event handlers as closures
    let loaded_metadata = move |_| {
        if let Some(video) = video_ref.get() {
            set_duration.set(video.duration());
        }
    };

    let time_update = move |_| {
        if let Some(video) = video_ref.get() {
            let time = video.current_time();
            set_current_time.set(time);
            if video.duration() > 0.0 {
                set_progress.set(time / video.duration());
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

    let seek = move |ev: MouseEvent| {
        ev.stop_propagation();
        if let Some(video) = video_ref.get() {
            let target = event_target::<web_sys::HtmlDivElement>(&ev);
            let rect = target.get_bounding_client_rect();
            let pos = (ev.client_x() as f64 - rect.left()) / rect.width();
            let seek_time = pos * video.duration();
            video.set_current_time(seek_time);
            set_progress.set(pos);
            set_current_time.set(seek_time);
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
            video.set_muted(muted);

            if muted {
                set_volume.set(0.0);
            } else {
                set_volume.set(video.volume());
            }
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
        format!("{:02}:{:02}", minutes, seconds)
    };

    view! {
        <div class="relative bg-black overflow-hidden shadow-xl">
            // Video element
            <video
                node_ref=video_ref
                src=src
                width=width.to_string()
                height=height.to_string()
                on:loadedmetadata=loaded_metadata
                on:timeupdate=time_update
                on:click=toggle_play
                preload="metadata"
                class="w-full cursor-pointer"
            />

            // Controls
            <div class="bg-gray-900">
                // Progress bar
                <div
                    class="w-full h-2 bg-gray-800 cursor-pointer hover:bg-gray-700 transition-colors"
                    on:click=seek
                >
                    <div
                        class="h-full bg-blue-500 transition-all duration-100"
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
                                min="0"
                                max="1"
                                step="0.01"
                                value=move || volume.get()
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
            <svg class="w-6 h-6" fill="currentColor" viewBox="0 0 20 20">
                <path
                    fill-rule="evenodd"
                    d="M10 18a8 8 0 100-16 8 8 0 000 16zM9.555 7.168A1 1 0 008 8v4a1 1 0 001.555.832l3-2a1 1 0 000-1.664l-3-2z"
                    clip-rule="evenodd"
                />
            </svg>
        }
    } else {
        view! {
            <svg class="w-6 h-6" fill="currentColor" viewBox="0 0 20 20">
                <path
                    fill-rule="evenodd"
                    d="M18 10a8 8 0 11-16 0 8 8 0 0116 0zM7 8a1 1 0 012 0v4a1 1 0 11-2 0V8zm5-1a1 1 0 00-1 1v4a1 1 0 102 0V8a1 1 0 00-1-1z"
                    clip-rule="evenodd"
                />
            </svg>
        }
    }
}
fn volume_icon(volume: f64) -> impl IntoView {
    if volume <= 0.0 {
        view! {
            <svg
                class="w-5 h-5"
                fill="currentColor"
                viewBox="0 0 20 20"
                xmlns="http://www.w3.org/2000/svg"
            >
                <path d="M9.383 3.076A1 1 0 0110 4v12a1 1 0 01-1.707.707L4.586 13H2a1 1 0 01-1-1V8a1 1 0 011-1h2.586l3.707-3.707a1 1 0 011.09-.217zM12.293 7.293a1 1 0 011.414 0L15 8.586l1.293-1.293a1 1 0 111.414 1.414L16.414 10l1.293 1.293a1 1 0 01-1.414 1.414L15 11.414l-1.293 1.293a1 1 0 01-1.414-1.414L13.586 10l-1.293-1.293a1 1 0 010-1.414z" />
            </svg>
        }
    } else if volume < 0.5 {
        view! {
            <svg
                class="w-5 h-5"
                fill="currentColor"
                viewBox="0 0 20 20"
                xmlns="http://www.w3.org/2000/svg"
            >
                <path d="M9.383 3.076A1 1 0 0110 4v12a1 1 0 01-1.707.707L4.586 13H2a1 1 0 01-1-1V8a1 1 0 011-1h2.586l3.707-3.707a1 1 0 011.09-.217zM12.828 4.757a1 1 0 011.415 0A5.983 5.983 0 0115 10a5.984 5.984 0 01-1.757 4.243 1 1 0 01-1.415-1.415A3.984 3.984 0 0013 10a3.983 3.983 0 00-1.172-2.828 1 1 0 010-1.415z" />
            </svg>
        }
    } else {
        view! {
            <svg
                class="w-5 h-5"
                fill="currentColor"
                viewBox="0 0 20 20"
                xmlns="http://www.w3.org/2000/svg"
            >
                <path d="M9.383 3.076A1 1 0 0110 4v12a1 1 0 01-1.707.707L4.586 13H2a1 1 0 01-1-1V8a1 1 0 011-1h2.586l3.707-3.707a1 1 0 011.09-.217zM14.657 2.929a1 1 0 011.414 0A9.972 9.972 0 0119 10a9.972 9.972 0 01-2.929 7.071 1 1 0 01-1.414-1.414A7.971 7.971 0 0017 10c0-2.21-.894-4.208-2.343-5.657a1 1 0 010-1.414zm-2.829 2.828a1 1 0 011.415 0A5.983 5.983 0 0115 10a5.984 5.984 0 01-1.757 4.243 1 1 0 01-1.415-1.415A3.984 3.984 0 0013 10a3.983 3.983 0 00-1.172-2.828 1 1 0 010-1.415z" />
            </svg>
        }
    }
}

fn fullscreen_icon(fullscreen: bool) -> impl IntoView {
    if fullscreen {
        view! {
            <svg
                class="w-5 h-5"
                fill="currentColor"
                viewBox="0 0 20 20"
                xmlns="http://www.w3.org/2000/svg"
            >
                <path
                    fill-rule="evenodd"
                    d="M3 4a1 1 0 011-1h4a1 1 0 010 2H6.414l2.293 2.293a1 1 0 01-1.414 1.414L5 6.414V8a1 1 0 01-2 0V4zm9 1a1 1 0 010-2h4a1 1 0 011 1v4a1 1 0 01-2 0V6.414l-2.293 2.293a1 1 0 11-1.414-1.414L13.586 5H12zm-9 7a1 1 0 012 0v1.586l2.293-2.293a1 1 0 111.414 1.414L6.414 15H8a1 1 0 010 2H4a1 1 0 01-1-1v-4zm13-1a1 1 0 011 1v4a1 1 0 01-1 1h-4a1 1 0 010-2h1.586l-2.293-2.293a1 1 0 111.414-1.414L15 13.586V12a1 1 0 011-1z"
                    clip-rule="evenodd"
                />
            </svg>
        }
    } else {
        view! {
            <svg
                class="w-5 h-5"
                fill="currentColor"
                viewBox="0 0 20 20"
                xmlns="http://www.w3.org/2000/svg"
            >
                <path
                    fill-rule="evenodd"
                    d="M3 7a1 1 0 011-1h3a1 1 0 010 2H6.414l1.293 1.293a1 1 0 01-1.414 1.414L5 9.414V10a1 1 0 01-2 0V7zm13 6a1 1 0 011 1v3a1 1 0 01-1 1h-3a1 1 0 010-2h1.586l-1.293-1.293a1 1 0 111.414-1.414L15 14.586V14a1 1 0 011-1zm-9-4a1 1 0 011-1h4a1 1 0 110 2h-1.586l1.293 1.293a1 1 0 11-1.414 1.414L10 11.414V12a1 1 0 11-2 0V9zM7 3a1 1 0 00-1 1v3a1 1 0 102 0V5.414l1.293 1.293a1 1 0 101.414-1.414L8.414 4H10a1 1 0 100-2H7z"
                    clip-rule="evenodd"
                />
            </svg>
        }
    }
}
