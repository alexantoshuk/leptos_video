#![allow(unused_must_use)]
mod controls;
pub mod state;
mod video;
use super::icon;
use crate::app::utils::*;
use controls::Controls;
use leptos::html;
use leptos::prelude::*;
use leptos_use::{UseElementSizeReturn, use_element_size, use_throttle_fn_with_arg};
use state::{AudioState, TimeFormat, VideoMetadata, calc_video_box};
use video::Video;
use web_time::Duration;

#[component]
pub fn VideoPlayer(
    metadata: RwSignal<VideoMetadata>,
    #[prop(into, optional)] overlay_controls: Signal<bool>,
) -> impl IntoView {
    let container_ref = NodeRef::<html::Div>::new();
    let video_container_ref = NodeRef::<html::Div>::new();
    let video_ref = NodeRef::<html::Video>::new();
    let proxy_ref = NodeRef::<html::Video>::new();
    let controls_ref = NodeRef::<html::Div>::new();

    let frame = RwSignal::new(0);
    let playback_rate = RwSignal::new(1.0);
    let time_format = RwSignal::new(TimeFormat::default());
    let progress = RwSignal::new(0.0);
    let audio_state = RwSignal::new(AudioState::default());
    let is_playing = RwSignal::new(false);
    let is_loop = RwSignal::new(false);
    let is_dragging = RwSignal::new(false);
    let is_fullscreen = RwSignal::new(false);
    let is_waiting = RwSignal::new(false);
    let hide_overlay_controls = RwSignal::new(false);
    let overlay = Signal::derive(move || overlay_controls.get() || is_fullscreen.get());

    let UseElementSizeReturn {
        width: video_container_width,
        height: video_container_height,
    } = use_element_size(video_container_ref);

    let toggle_play = move || {
        is_playing.update(|p| *p = !*p);
    };

    let toggle_fullscreen = move || {
        is_fullscreen.update(|f| *f = !*f);
    };

    let next_frame = move || {
        is_playing.maybe_set(false);
        let end_frame = metadata.read_untracked().end_frame;
        frame.maybe_update(|f| {
            if *f >= end_frame {
                false
            } else {
                *f += 1;
                true
            }
        });
    };

    let prev_frame = move || {
        is_playing.maybe_set(false);
        frame.maybe_update(|f| {
            if *f == 0 {
                false
            } else {
                *f -= 1;
                true
            }
        });
    };

    let on_click = move |_| toggle_play();
    let on_dblclick = move |_| toggle_fullscreen();

    let reset_overlay_controls_timeout = {
        let timeout = StoredValue::new(None::<TimeoutHandle>);
        move || {
            if let Some(timeout) = timeout.get_value() {
                timeout.clear();
            }
            if !overlay.get_untracked() {
                return;
            }
            if hide_overlay_controls.get_untracked() {
                hide_overlay_controls.set(false);
            }

            timeout.set_value(
                set_timeout_with_handle(
                    move || {
                        let hover_or_focus = if let Some(controls) = controls_ref.get() {
                            controls.matches(":hover, :focus").unwrap()
                        } else {
                            false
                        };
                        let show = hover_or_focus || is_dragging.get_untracked();
                        hide_overlay_controls.set(!show);
                    },
                    Duration::from_secs(2),
                )
                .ok(),
            );
        }
    };

    let handle_keydown = {
        let throttled_fn = use_throttle_fn_with_arg(
            move |key: String| {
                // ev.stop_propagation();
                // ev.prevent_default();
                match key.as_str() {
                    " " => toggle_play(),
                    "f" => toggle_fullscreen(),
                    "m" => audio_state.write().toggle_mute(),
                    "ArrowLeft" => {
                        prev_frame();
                    }
                    "ArrowRight" => {
                        next_frame();
                    }
                    _ => (),
                }
            },
            150.0,
        );
        move |ev: leptos::ev::KeyboardEvent| {
            if is_dragging.get_untracked() {
                return;
            }
            let key = ev.key();
            throttled_fn(key);
        }
    };

    // fullscreen
    Effect::new(move |_| {
        if is_fullscreen.get() {
            request_fullscreen(container_ref.get_untracked());
        } else if document().fullscreen() {
            document().exit_fullscreen();
        }
    });

    Effect::new(move |_| {
        if !is_dragging.get() {
            reset_overlay_controls_timeout();
        }
    });

    view! {
        <div
            node_ref=container_ref
            autofocus
            tabindex="-1"
            class="@container size-full relative flex bg-black flex-col overflow-hidden touch-none group outline-none select-none"
            on:fullscreenchange=move |_| {
                let fscr = is_element_fullscreen(container_ref);
                is_fullscreen.set(fscr);
            }
            on:keydown=handle_keydown
            on:mousemove=move |_| reset_overlay_controls_timeout()
            on:touchmove=move |_| reset_overlay_controls_timeout()
        >
            // Video container
            <div
                node_ref=video_container_ref
                class="relative flex-auto cursor-pointer select-none"
                class=("cursor-none!", move || overlay.get() && hide_overlay_controls.get())
                on:contextmenu=move |ev| ev.prevent_default()
                on:click=onclick_handler(on_click, on_dblclick)
                on:dblclick=move |ev| ev.prevent_default()
            >
                <div
                    class="absolute"
                    style=move || {
                        let aspect_ratio = metadata.read().aspect_ratio;
                        let (w, h, x, y) = calc_video_box(
                            video_container_width.get(),
                            video_container_height.get(),
                            aspect_ratio,
                        );
                        let s = 1;
                        format!("width:{w}px;height:{h}px;scale:{s};translate:{x}px {y}px")
                    }
                >
                    <Video
                        video_ref
                        proxy_ref
                        metadata
                        frame
                        is_playing
                        is_loop
                        is_dragging
                        is_waiting
                        progress
                        audio_state
                        playback_rate
                    />

                    // Canvas container
                    <div class="absolute size-full pointer-events-none"></div>
                </div>

                // Overlay Controls container
                <div
                    class="absolute size-full flex justify-center pointer-events-none"
                    class:hidden=move || !is_waiting.get()
                >
                    // Spinner
                    <div class="loading loading-spinner scale-200 loading-xl"></div>
                </div>

            </div>

            // Controls
            <div
                class="transition-opacity duration-200"
                class:opacity-0=move || overlay.get() && hide_overlay_controls.get()
            >
                // Overlay container with gradient
                <div
                    class="absolute top-0 left-0 overlay-gradient pointer-events-none z-100"
                    class:hidden=move || !overlay.get()
                    style=move || {
                        format!(
                            "width:{}px;height:{}px;",
                            video_container_width.get(),
                            video_container_height.get(),
                        )
                    }
                >
                    <span>Video name and other information</span>
                </div>

                // Control bar
                <div
                    tabindex="-1"
                    node_ref=controls_ref
                    class="flex-none outline-none bottom-0 px-3 @3xl:px-4 @4xl:px-6 z-200"
                    class=(["absolute", "inset-x-0", "w-full"], overlay)
                >
                    <Controls
                        proxy_ref
                        metadata
                        progress
                        overlay
                        frame
                        playback_rate
                        is_dragging
                        is_playing
                        is_loop
                        audio_state
                        is_fullscreen
                        time_format
                    />
                </div>
            </div>
        </div>
    }
}
