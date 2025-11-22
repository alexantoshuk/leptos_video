#![allow(unused_must_use)]
mod controls;
mod video;
use super::icon;
use crate::app::timecode::*;
use crate::app::utils::*;
use controls::Controls;
use leptos::either::*;
use leptos::html;
use leptos::logging::log;
use leptos::prelude::*;
use leptos_use::core::Position;
use leptos_use::{
    UseDraggableCallbackArgs, UseDraggableOptions, UseElementSizeReturn,
    use_draggable_with_options, use_element_size, use_throttle_fn, use_throttle_fn_with_arg,
    utils::Pausable,
};
use video::Video;
use web_sys::{
    self, CanvasRenderingContext2d, HtmlCanvasElement, HtmlElement, HtmlInputElement,
    HtmlMediaElement, HtmlVideoElement, MouseEvent,
};
use web_time::{Duration, Instant};

const THUMB_MAX_SIZE: i32 = 200;

#[derive(Clone, Debug, Default)]
pub struct VideoMetadata {
    pub fps: Signal<f64>,
    pub end_frame: Signal<u32>,
    pub aspect_ratio: Signal<f64>,
}

#[component]
pub fn VideoPlayer(
    #[prop(into)] src: Signal<String>,
    #[prop(into, optional)] proxy: Signal<String>,
    #[prop(into)] fps: Signal<f64>,
    #[prop(into)] aspect_ratio: Signal<f64>,
    #[prop(into, optional)] overlay_controls: Signal<bool>,
) -> impl IntoView {
    let container_ref = NodeRef::<html::Div>::new();
    let video_container_ref = NodeRef::<html::Div>::new();
    let video_ref = NodeRef::<html::Video>::new();
    let proxy_ref = NodeRef::<html::Video>::new();
    let controls_ref = NodeRef::<html::Div>::new();

    let frame = RwSignal::new(0);
    let end_frame = RwSignal::new(0);
    let playback_rate = RwSignal::new(1.0);
    let time_format = RwSignal::new(TimeFormat::default());
    let progress = RwSignal::new(0.0);
    let volume = RwSignal::new(0.5);
    let is_mute = RwSignal::new(false);
    let is_playing = RwSignal::new(false);
    let is_loop = RwSignal::new(false);
    let is_dragging = RwSignal::new(false);
    let is_fullscreen = RwSignal::new(false);
    let is_waiting = RwSignal::new(false);
    let hide_overlay_controls = RwSignal::new(false);
    let overlay = Signal::derive(move || overlay_controls.get() || is_fullscreen.get());
    let metadata = VideoMetadata {
        fps,
        end_frame: end_frame.into(),
        aspect_ratio,
    };

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
        if let Some(video) = video_ref.get_untracked() {
            video.pause();
            is_playing.set_changed(false);

            let end_frame = end_frame.get_untracked();
            let f = frame.get_untracked();
            if f >= end_frame {
                return;
            }

            let f = f + 1;
            frame.set_changed(f);
            // let fps = fps.get_untracked();
            // let time = time_from_frame(f, fps);
            // video.set_current_time(time);
        }
    };

    let prev_frame = move || {
        if let Some(video) = video_ref.get_untracked() {
            video.pause();
            is_playing.set_changed(false);

            let f = frame.get_untracked();
            if f == 0 {
                return;
            }
            let f = f - 1;
            frame.set_changed(f);
            // let fps = fps.get_untracked();
            // let time = time_from_frame(f, fps);
            // video.set_current_time(time);
        }
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
                    "m" => toggle_mute(is_mute, volume),
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
                on:contextmenu=move |ev| ev.prevent_default()
                on:click=onclick_handler(on_click, on_dblclick)
                on:dblclick=move |ev| ev.prevent_default()
            >

                <div
                    class="absolute"
                    style=move || {
                        let (w, h, x, y) = calc_video_box(
                            video_container_width.get(),
                            video_container_height.get(),
                            aspect_ratio.get(),
                        );
                        let s = 1;
                        format!("width:{w}px;height:{h}px;scale:{s};translate:{x}px {y}px")
                    }
                >
                    <Video
                        video_ref
                        proxy_ref
                        src
                        proxy
                        fps
                        frame
                        end_frame
                        is_playing
                        is_loop
                        is_dragging
                        is_waiting
                        progress
                        volume
                        is_mute
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
                    class="absolute top-0 left-0 overlay-gradient pointer-events-none"
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
                    class="flex-none outline-none bottom-0 px-3 @3xl:px-4 @4xl:px-6"
                    class=(["absolute", "inset-x-0", "w-full"], overlay)
                >
                    <Controls
                        proxy_ref
                        fps
                        aspect_ratio
                        end_frame
                        progress
                        overlay
                        frame
                        playback_rate
                        is_dragging
                        is_playing
                        is_loop
                        is_mute
                        volume
                        is_fullscreen
                        time_format
                    />
                </div>
            </div>
        </div>
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub enum TimeFormat {
    #[default]
    Frames,
    Timecode,
}

fn timecode_str(
    frame: u32,
    fps: f64,
    end_frame: u32,
    time_format: TimeFormat,
    show_frames: bool,
) -> String {
    match time_format {
        TimeFormat::Frames => frame.to_string(),
        TimeFormat::Timecode => {
            let show_hours = Timecode::hours(end_frame, fps) != 0;
            let t = Timecode::from_frame(frame, fps);
            t.to_string_opt(show_hours, show_frames)
        }
    }
}

fn toggle_mute(is_mute: RwSignal<bool>, volume: RwSignal<f64>) {
    let m = !is_mute.get_untracked();
    if !m && volume.get_untracked() == 0.0 {
        volume.set(1.0);
    }
    is_mute.set(m);
}

fn frame_from_pos(pos: f64, end_frame: u32) -> u32 {
    let pos = pos.max(0.0);
    let total_frames = end_frame + 1;
    ((pos * total_frames as f64 + 0.01) as u32).min(end_frame)
}

fn frame_from_time(time: f64, fps: f64) -> u32 {
    (time * fps + 0.01) as u32
}

fn time_from_frame(frame: u32, fps: f64) -> f64 {
    (frame as f64 + 0.01) / fps
}

fn set_current_frame(video_ref: NodeRef<html::Video>, f: u32, fps: f64) {
    if let Some(video) = video_ref.get_untracked() {
        let time = time_from_frame(f, fps);
        video.set_current_time(time);
    }
}

fn thumbnail_size(aspect_ratio: f64) -> (i32, i32) {
    if aspect_ratio > 1.0 {
        (
            THUMB_MAX_SIZE,
            (THUMB_MAX_SIZE as f64 / aspect_ratio).round() as i32,
        )
    } else {
        (
            (THUMB_MAX_SIZE as f64 * aspect_ratio).round() as i32,
            THUMB_MAX_SIZE,
        )
    }
}

fn calc_video_box(
    container_width: f64,
    container_height: f64,
    video_aspect: f64,
) -> (f64, f64, f64, f64) {
    let container_aspect = container_width / container_height;
    if video_aspect < container_aspect {
        let w = video_aspect * container_height;
        let h = container_height;
        let x = (container_width - w) / 2.0;
        let y = 0.0;
        (w, h, x, y)
    } else {
        let w = container_width;
        let h = container_width / video_aspect;
        let x = 0.0;
        let y = (container_height - h) / 2.0;
        (w, h, x, y)
    }
}

fn is_seekable(noderef: NodeRef<html::Video>, time: f64) -> bool {
    if let Some(video) = noderef.get()
        && video.ready_state() >= HtmlMediaElement::HAVE_METADATA
    {
        let range = video.seekable();
        time >= range.start(0).unwrap() && time <= range.end(0).unwrap()
    } else {
        false
    }
}
