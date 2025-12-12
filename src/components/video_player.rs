#![allow(unused_must_use)]
mod controls;
pub mod state;
mod video;
use super::icon;
use crate::utils::*;
use controls::Controls;
use leptos::either::*;
use leptos::html;
use leptos::logging::log;
use leptos::prelude::*;
use leptos_use::{UseElementSizeReturn, use_element_size, use_throttle_fn_with_arg};
use state::{AudioState, PlayingState, TimeFormat, VideoInfo, calc_video_box};
use video::Video;
use web_time::Duration;

#[derive(Clone, Copy, Default, Debug, PartialEq)]
pub enum OverlayBtn {
    #[default]
    None,
    Play,
    Pause,
    Mute,
    UnMute,
}

#[component]
pub fn VideoPlayer(
    videoinfo: RwSignal<VideoInfo>,
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
    let playing = RwSignal::new(PlayingState::Pause);
    let overlay_btn = RwSignal::new(OverlayBtn::default());
    let is_loop = RwSignal::new(false);
    let is_dragging = RwSignal::new(false);
    let is_fullscreen = RwSignal::new(false);
    let is_waiting = RwSignal::new(false);
    let hide_overlay_controls = RwSignal::new(false);
    let overlay = Signal::derive(move || overlay_controls.get() || is_fullscreen.get());
    let prevent_on_click_once = StoredValue::new(false);
    let UseElementSizeReturn {
        width: video_container_width,
        height: video_container_height,
    } = use_element_size(video_container_ref);

    let toggle_play = move || {
        playing.write().toggle_play();
    };

    let toggle_fullscreen = move || {
        is_fullscreen.update(|f| *f = !*f);
    };

    let next_frame = move || {
        playing.maybe_set(PlayingState::Pause);
        let end_frame = videoinfo.read_untracked().end_frame;
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
        playing.maybe_set(PlayingState::Pause);
        frame.maybe_update(|f| {
            if *f == 0 {
                false
            } else {
                *f -= 1;
                true
            }
        });
    };

    let on_imm_click = move |_| {
        overlay_btn.set(match playing.get_untracked() {
            PlayingState::Play => OverlayBtn::Pause,
            _ => OverlayBtn::Play,
        })
    };
    let on_click = move |_| {
        toggle_play();
    };
    let on_dblclick = move |_| {
        overlay_btn.set(match playing.get_untracked() {
            PlayingState::Play => OverlayBtn::Play,
            _ => OverlayBtn::Pause,
        });

        toggle_fullscreen();
    };

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
                    "m" => {
                        audio_state.write().toggle_mute();
                        let ob = if audio_state.read_untracked().is_muted {
                            OverlayBtn::Mute
                        } else {
                            OverlayBtn::UnMute
                        };
                        overlay_btn.set(ob);
                    }
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

    Effect::new(move |_| {
        if playing.get() == PlayingState::EndPause {
            if overlay_btn.get_untracked() == OverlayBtn::Pause {
                prevent_on_click_once.set_value(true);
            }
        }
    });

    view! {
        <div
            node_ref=container_ref
            autofocus
            tabindex="-1"
            class="@container size-full relative flex bg-neutral-900 flex-col overflow-hidden touch-none group outline-none select-none"
            on:fullscreenchange=move |_| {
                let fscr = is_element_fullscreen(container_ref);
                is_fullscreen.maybe_set(fscr);
            }
            on:keydown=handle_keydown
            on:mousemove=move |_| reset_overlay_controls_timeout()
            on:touchmove=move |_| reset_overlay_controls_timeout()
        >
            // Video container
            <div
                node_ref=video_container_ref
                class="relative flex-auto cursor-pointer select-none"
                class=("overlay:cursor-none!", hide_overlay_controls)
                on:contextmenu=move |ev| ev.prevent_default()
                on:click=onclick_handler(
                    on_imm_click,
                    on_click,
                    on_dblclick,
                    Some(prevent_on_click_once),
                )
                on:dblclick=move |ev| {
                    ev.prevent_default();
                    ev.stop_propagation();
                }
            >
                <div
                    class="absolute"
                    style=move || {
                        let aspect_ratio = videoinfo.read().aspect_ratio;
                        let width = video_container_width.get();
                        let height = video_container_height.get();
                        let (w, h, x, y) = calc_video_box(width, height, aspect_ratio);
                        let s = 1;
                        format!("width:{w}px;height:{h}px;scale:{s};translate:{x}px {y}px")
                    }
                >
                    <Video
                        video_ref
                        proxy_ref
                        videoinfo
                        frame
                        playing
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
                <div class="absolute size-full flex justify-center items-center pointer-events-none">
                    <OverlayFeedback overlay_btn />
                </div>
                <Show when=move || is_waiting.get()>
                    <div class="absolute size-full flex justify-center items-center pointer-events-none">
                        <Spinner />
                    </div>
                </Show>
            </div>

            // Controls
            <div
                class="transition-opacity duration-200"
                class=("overlay:opacity-0", hide_overlay_controls)
            >
                // Overlay container with gradient
                <div
                    class="absolute top-0 left-0 overlay-gradient pointer-events-none z-100 invisible overlay:visible"
                    class:overlay=overlay_controls
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
                    class="flex-none outline-none bottom-0 px-3 @3xl:px-4 @4xl:px-6 z-200 overlay:absolute overlay:inset-x-0 overlay:w-full"
                    class:overlay=overlay_controls
                >
                    <Controls
                        proxy_ref
                        videoinfo
                        progress
                        frame
                        playback_rate
                        is_dragging
                        playing
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

#[component]
fn OverlayFeedback(#[prop(into)] overlay_btn: Signal<OverlayBtn>) -> impl IntoView {
    let toggle = RwSignal::new(false);

    Effect::new(move |_| {
        let _ = overlay_btn.get();
        toggle.set(false);
        set_timeout(
            move || {
                toggle.set(true);
            },
            Duration::from_millis(0),
        );
    });

    view! {
        {move || {
            match overlay_btn.get() {
                OverlayBtn::Play => {
                    EitherOf5::A(
                        view! {
                            <div
                                class="size-24 rounded-full bg-black/40 flex justify-center items-center pl-1 opacity-0"
                                class:animate-zoomfade=toggle
                            >
                                <icon::Play class="size-16 text-white/80" />
                            </div>
                        },
                    )
                }
                OverlayBtn::Pause => {
                    EitherOf5::B(
                        view! {
                            <div
                                class="size-24 rounded-full bg-black/40 flex justify-center items-center opacity-0"
                                class:animate-zoomfade=toggle
                            >
                                <icon::Pause class="size-16 text-white/80" />
                            </div>
                        },
                    )
                }
                OverlayBtn::Mute => {
                    EitherOf5::C(
                        view! {
                            <div
                                class="size-24 rounded-full bg-black/40 flex justify-center items-center opacity-0"
                                class:animate-zoomfade=toggle
                            >
                                <icon::Volume0 class="size-16 text-white/80" />
                            </div>
                        },
                    )
                }
                OverlayBtn::UnMute => {
                    EitherOf5::D(
                        view! {
                            <div
                                class="size-24 rounded-full bg-black/40 flex justify-center items-center opacity-0"
                                class:animate-zoomfade=toggle
                            >
                                <icon::Volume2 class="size-16 text-white/80" />
                            </div>
                        },
                    )
                }
                _ => EitherOf5::E(()),
            }
        }}
    }
}

#[component]
fn Spinner() -> impl IntoView {
    view! { <div class="loading loading-spinner size-25 bg-white/70"></div> }
}
