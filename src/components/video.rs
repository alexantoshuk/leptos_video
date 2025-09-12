use super::icon::*;
use leptos::ev::Event;
use leptos::logging::log;
use leptos::prelude::*;
use leptos::reactive::owner::StoredValue;
use leptos::*;
use leptos_use::core::Position;
use leptos_use::{
    use_debounce_fn, use_draggable_with_options, use_mouse_in_element, UseDraggableOptions,
};
use web_sys;
use web_sys::MouseEvent;

#[derive(Debug, Clone, PartialEq, Eq)]
enum Dragging {
    Start,
    Move,
    None,
}

#[component]
pub fn Video(
    #[prop(into)] src: Signal<String>,
    #[prop(into, optional)] proxy: Signal<String>,
    #[prop(into)] fps: Signal<f64>,
) -> impl IntoView {
    let container_ref = NodeRef::<html::Div>::new();
    let video_ref = NodeRef::<html::Video>::new();
    let proxy_ref = NodeRef::<html::Video>::new();
    let progress_ref = NodeRef::<html::Div>::new();
    let (is_playing, set_is_playing) = signal(false);
    let (dragging, set_dragging) = signal(Dragging::None);
    let (frame, set_frame) = signal(0);
    let (end_frame, set_end_frame) = signal(0);
    let (preload_progress, set_preload_progress) = signal(0.0);
    let (controls_visible, set_controls_visible) = signal(false);
    let (is_muted, set_is_muted) = signal(false);
    let (volume, set_volume) = signal(1.0);
    let (is_fullscreen, set_is_fullscreen) = signal(false);

    let container_mouse = use_mouse_in_element(container_ref);

    let controls_hide_after_delay = use_debounce_fn(
        move || {
            set_controls_visible.set(false);
        },
        2000.0, // 2 seconds
    );

    let time_update = move |_| {
        if is_playing.get() {
            if let Some(video) = video_ref.get() {
                let time = video.current_time();
                let frame = f64::floor(time * fps.get()) as i32;
                let frame = frame.min(end_frame.get());
                set_frame.set(frame);
            }
        }
    };

    let load_metadata = move || {
        if let Some(video) = video_ref.get() {
            let d = video.duration();
            if d.is_finite() {
                let total_frames = f64::floor(d * fps.get()) as i32;
                set_end_frame.set((total_frames - 1).max(0));
            }
        }
    };

    let preload_update = move || {
        if let Some(video) = video_ref.get() {
            let d = video.duration();
            let vb = video.buffered();
            let time = video.current_time();
            for i in (0..vb.length()).rev() {
                let start = vb.start(i).unwrap();
                let end = vb.end(i).unwrap();
                if time >= start && time <= end {
                    set_preload_progress.set(end / d);
                    break;
                }
            }
        }
    };

    let is_ended = move || frame.get() == end_frame.get();

    let seek = move |frame: i32| {
        if let Some(video) = video_ref.get() {
            let frame = frame.clamp(0, end_frame.get());
            set_frame.set(frame);
            let time = frame as f64 / fps.get();
            video.set_current_time(time);
            if proxy.get() == "" {
                return;
            }
            if let Some(video) = proxy_ref.get() {
                video.set_current_time(time);
            }
        }
    };

    let stop = move || {
        if let Some(video) = video_ref.get() {
            video.set_current_time(0.0);
            set_frame.set(0);
        }
    };

    let play = move || {
        if let Some(video) = video_ref.get() {
            if is_ended() {
                stop();
            }
            set_is_playing.set(true);
            video.play();
        }
    };

    let pause = move || {
        if let Some(video) = video_ref.get() {
            set_is_playing.set(false);
            video.pause();
        }
    };

    let next_frame = move || {
        seek(frame.get() + 1);
    };

    let prev_frame = move || {
        seek(frame.get() - 1);
    };

    let toggle_play = move || {
        if is_playing.get() {
            pause();
        } else {
            play();
        }
    };

    let drag_offset = StoredValue::new(0.0);
    let is_played_before_drag = StoredValue::new(false);
    use_draggable_with_options(
        progress_ref,
        UseDraggableOptions::default()
            .initial_value(Position { x: 0.0, y: 0.0 })
            .target_offset(move |ev| (0.0, 0.0))
            .on_start(move |ev| {
                if let Some(p) = progress_ref.get() {
                    if ev.event.pointer_type() == "touch" {
                        let _ = p.focus();
                    }
                    if is_playing.get() {
                        is_played_before_drag.set_value(true);
                        pause();
                    } else {
                        is_played_before_drag.set_value(false);
                    }

                    set_dragging.set(Dragging::Start);

                    let x = ev.event.offset_x() as f64;
                    drag_offset.set_value(x);

                    let pos = x / p.client_width() as f64;
                    let total_frames = end_frame.get() + 1;
                    let frame = (pos * total_frames as f64).floor() as i32;
                    seek(frame);
                    true
                } else {
                    false
                }
            })
            .on_move(move |ev| {
                if let Some(p) = progress_ref.get() {
                    set_dragging.set(Dragging::Move);
                    let x = ev.position.x + drag_offset.get_value();

                    let pos = x / p.client_width() as f64;
                    let total_frames = end_frame.get() + 1;
                    let frame = (pos * total_frames as f64).floor() as i32;
                    seek(frame);
                    set_controls_visible.set(true);
                }
            })
            .on_end(move |_| {
                set_dragging.set(Dragging::None);
                if is_played_before_drag.get_value() {
                    play();
                }
            })
            // .stop_propagation(true)
            .prevent_default(true),
    );

    let handle_keydown = move |ev: leptos::ev::KeyboardEvent| {
        ev.stop_propagation();
        // ev.prevent_default();
        match ev.key().as_str() {
            " " => toggle_play(),
            "ArrowLeft" => prev_frame(),
            "ArrowRight" => next_frame(),
            _ => (),
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
            autofocus
            tabindex="-1"
            class="size-full flex bg-black flex-col overflow-hidden shadow-xl touch-none group"
            on:fullscreenchange=fullscreenchange
            on:keydown=handle_keydown
        >
            // Video element
            <div class="relative flex-auto m-[1px] group-fullscreen:m-0">
                <video
                    playsinline
                    disablepictureinpicture
                    controlslist="nodownload"
                    node_ref=proxy_ref
                    src=proxy
                    preload="auto"
                    class="cursor-pointer absolute size-full object-contain"
                    style:visibility=move || {
                        if proxy.get() == "" { "hidden" } else { "visible" }
                    }
                    on:contextmenu=move |ev| ev.prevent_default()
                />

                <video
                    // controls
                    playsinline
                    disablepictureinpicture
                    controlslist="nodownload"
                    node_ref=video_ref
                    src=src
                    // preload="auto"
                    class="cursor-pointer absolute size-full object-contain"
                    style:visibility=move || {
                        if proxy.get() != "" && dragging.get() == Dragging::Move {
                            "hidden"
                        } else {
                            "visible"
                        }
                    }
                    on:contextmenu=move |ev| ev.prevent_default()
                    on:loadedmetadata=move |m| {
                        log!("{:?}",m);
                        load_metadata()
                    }
                    on:durationchange=move |_| load_metadata()
                    on:timeupdate=time_update
                    on:click=move |_| toggle_play()
                    on:progress=move |_| preload_update()
                    on:canplaythrough=move |_| preload_update()
                    on:ratechange=move |_| log!("ratechange")
                    on:ended=move |_| { set_is_playing.set(false) }
                />

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
                        // Preload
                        <div
                            class="absolute origin-left h-full w-full bg-white/20 transition-scale duration-200 pointer-events-none"
                            style:transform=move || {
                                format!("scaleX({})", preload_progress.get())
                            }
                        />

                        // Progress
                        <div
                            class="absolute origin-left h-full w-full bg-blue-500 pointer-events-none"
                            style:transform=move || {
                                format!(
                                    "scaleX({})",
                                    frame.get() as f64 / (end_frame.get() + 1) as f64,
                                )
                            }
                        />

                        // Cursor
                        <div
                            class="absolute origin-left h-full w-full bg-blue-300 pointer-events-none"
                            style:transform=move || {
                                let total_frames = (end_frame.get() + 1) as f64;
                                format!(
                                    "translateX({}%) scaleX({})",
                                    (100 * frame.get()) as f64 / total_frames,
                                    total_frames.recip(),
                                )
                            }
                        />

                    </div>

                    // Control buttons
                    <div class="flex items-center justify-between px-1 pb-2 pt-4 bottom-0">
                        // Left side
                        <div class="flex items-center space-x-4">
                            // Play/Pause button
                            <button
                                on:click=move |_| toggle_play()
                                on:keydown=move |ev| ev.prevent_default()
                                class="text-white hover:text-blue-400  hover:bg-white/10 0transition-colors p-1 rounded cursor-pointer"
                            >
                                <PlayPause play=is_playing />
                            </button>
                        </div>

                        // Center
                        <div class="flex items-center space-x-4">
                            // Time display
                            <div class="flex items-center text-white text-sm font-mono">
                                <span>{move || timecode(frame.get(), fps.get())}</span>
                                <span class="mx-1 text-gray-400">/</span>
                                <span class="text-gray-400">{frame}</span>
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
                                    <Volume volume=volume />
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
                                <Fullscreen fullscreen=is_fullscreen />
                            </button>
                        </div>
                    </div>
                </div>
            </div>
        </div>
    }
}

fn timecode(frame: i32, fps: f64) -> String {
    let fframe = frame as f64;
    let time = fframe / fps;
    let hours = (time / 360.0).floor() as i32;
    let minutes = (time / 60.0).floor() as i32;
    let seconds = (time % 60.0).floor() as i32;
    let frame = (fframe % fps) as i32;
    let pad = (fps as i32).to_string().len();
    format!("{hours:02}:{minutes:02}:{seconds:02}:{frame:0>pad$}")
}
