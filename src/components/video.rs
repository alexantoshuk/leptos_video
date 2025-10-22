#![allow(unused_must_use)]

use super::icon::{self, *};
use super::spinner;
use leptos::either::*;
use leptos::ev::{Event, volumechange};
use leptos::html;
use leptos::logging::log;
use leptos::prelude::*;
use leptos_use::core::{IntoElementMaybeSignal, Position};
use leptos_use::use_throttle_fn_with_arg_and_options;
use leptos_use::{
    UseDraggableCallbackArgs, UseDraggableOptions, UseDraggableReturn, use_draggable_with_options,
    use_throttle_fn, use_throttle_fn_with_arg, utils::Pausable,
};

use crate::utils::{is_element_fullscreen, use_video_frame_fn};
use std::time::Duration;
use web_sys::{self, HtmlInputElement, HtmlMediaElement, HtmlVideoElement};

const THUMB_MAX_SIZE: i32 = 200;

#[derive(Clone, Debug, Default)]
pub struct VideoMetadata {
    pub aspect: RwSignal<f64>,
    pub fps: Signal<f64>,
    pub end_frame: RwSignal<i32>,
}

#[component]
pub fn Video(
    #[prop(into)] src: Signal<String>,
    #[prop(into, optional)] proxy: Signal<String>,
    #[prop(into)] fps: Signal<f64>,
    #[prop(into, optional)] overlay_controls: Signal<bool>,
) -> impl IntoView {
    let container_ref = NodeRef::<html::Div>::new();
    let video_ref = NodeRef::<html::Video>::new();
    let proxy_ref = NodeRef::<html::Video>::new();
    let controls_ref = NodeRef::<html::Div>::new();
    let progress_ref = NodeRef::<html::Div>::new();

    let metadata = VideoMetadata {
        fps,
        ..Default::default()
    };

    let frame = RwSignal::new(0);
    // let end_frame = RwSignal::new(0);
    let preload_progress = RwSignal::new(0.0);
    // let aspect = RwSignal::new(0.0);
    let volume = RwSignal::new(0.5);
    let mute = RwSignal::new(true);

    // let display_proxy = RwSignal::new(false);
    let is_playing = RwSignal::new(false);
    let is_loop = RwSignal::new(true);
    let is_dragging = RwSignal::new(false);
    let is_fullscreen = RwSignal::new(false);
    let display_proxy = RwSignal::new(false);
    let is_waiting = RwSignal::new(false);

    let show_overlay_controls = RwSignal::new(false);

    let overlay = Signal::derive(move || overlay_controls.get() || is_fullscreen.get());
    let set_current_frame = move |video_ref: NodeRef<html::Video>, f: i32| {
        if let Some(video) = video_ref.get() {
            let fps = fps.get_untracked();
            let time = (f as f64) / fps;
            video.set_current_time(time);
            // frame.set(f);
        }
    };

    let load_metadata = move || {
        if let Some(video) = video_ref.get() {
            let d = video.duration();
            if d.is_finite() {
                let total_frames = (d * fps.get()).next_up() as i32;
                metadata.end_frame.set((total_frames - 1).max(0));
                metadata
                    .aspect
                    .set(video.video_width() as f64 / video.video_height() as f64);
            }
        }
    };

    let sync_preload_progress = move || {
        if let Some(video) = video_ref.get() {
            let vb = video.buffered();
            let time = video.current_time();
            let fps = fps.get();
            let end_frame = metadata.end_frame.get();
            let total_frames = (end_frame + 1) as f64;
            for i in (0..vb.length()).rev() {
                let start = vb.start(i).unwrap();
                let end = vb.end(i).unwrap();
                if time >= start && time <= end {
                    let f = frame_from_time(end, fps).min(end_frame) + 1;
                    let p = f as f64 / total_frames;
                    preload_progress.set(p);
                    break;
                }
            }
        }
    };

    let recover = move |video_ref: NodeRef<html::Video>| {
        if let Some(video) = video_ref.get() {
            let time = video.current_time();

            video.load();
            video.set_current_time(time);
            if is_playing.get_untracked() {
                video.play();
            }
        }
    };

    let is_ended = move || {
        let video = video_ref.get().unwrap();
        video.ended() || frame.get_untracked() >= metadata.end_frame.get_untracked()
    };

    let stop = move || {
        let video = video_ref.get().unwrap();
        video.pause();
        set_current_frame(video_ref, 0);
    };

    let play = move || {
        let video = video_ref.get().unwrap();
        if is_ended() {
            set_current_frame(video_ref, 0);
        }
        video.play();
    };

    let is_paused = move || {
        let video = video_ref.get().unwrap();
        video.paused()
    };

    let pause = move || {
        log!("press_pause");
        let video = video_ref.get().unwrap();
        video.pause();
    };

    let toggle_play = move || {
        is_playing.update(|p| *p = !*p);
    };

    let toggle_fullscreen = move || {
        is_fullscreen.update(|f| *f = !*f);
        // if let Some(el) = container_ref.get() {
        //     if is_fullscreen.get() {
        //         document().exit_fullscreen();
        //     } else {
        //         el.request_fullscreen();
        //     }
        // }
    };

    let next_frame = move || {
        if let Some(video) = video_ref.get() {
            if !video.paused() {
                video.pause();
                is_playing.set(false);
            }

            let end_frame = metadata.end_frame.get_untracked();
            let f = frame.get_untracked();
            if f >= end_frame {
                return;
            }
            let f = f + 1;
            let fps = fps.get_untracked();
            let time = (f as f64) / fps;
            video.set_current_time(time);
        }
    };

    let prev_frame = move || {
        if let Some(video) = video_ref.get() {
            if !video.paused() {
                video.pause();
                is_playing.set(false);
            }

            let f = frame.get_untracked();
            if f <= 0 {
                return;
            }
            let f = f - 1;
            let fps = fps.get_untracked();
            let time = (f as f64) / fps;
            video.set_current_time(time);
        }
    };

    let on_mouse_move = {
        let timeout = StoredValue::new(None::<TimeoutHandle>);
        let reset_timer = move || {
            if let Some(timeout) = timeout.get_value() {
                timeout.clear();
            }
            if !overlay.get_untracked() {
                return;
            }
            if !show_overlay_controls.get_untracked() {
                show_overlay_controls.set(true);
            }

            timeout.set_value(
                set_timeout_with_handle(
                    move || {
                        show_overlay_controls.set(false);
                    },
                    Duration::from_secs(2),
                )
                .ok(),
            );
        };
        move |_| reset_timer()
    };

    let handle_keydown = {
        let throttled_fn = use_throttle_fn_with_arg(
            move |key: String| {
                // ev.stop_propagation();
                // ev.prevent_default();
                match key.as_str() {
                    " " => toggle_play(),
                    "f" => toggle_fullscreen(),
                    "m" => toggle_mute(mute, volume),
                    "ArrowLeft" => {
                        prev_frame();
                    }
                    "ArrowRight" => {
                        next_frame();
                    }
                    _ => (),
                }
            },
            100.0,
        );
        move |ev: leptos::ev::KeyboardEvent| {
            if is_dragging.get_untracked() {
                return;
            }
            let key = ev.key();
            throttled_fn(key);
        }
    };

    // on create
    Effect::new(move |_| {
        load_metadata();
        if let Some(video) = video_ref.get() {
            use_video_frame_fn(video, move |time| {
                if is_dragging.get_untracked() {
                    return;
                }
                let f = frame_from_time(time, fps.get_untracked())
                    .min(metadata.end_frame.get_untracked());
                frame.set(f);
                log!("prcise time: {time}, frame: {f}");
            });
        }
    });

    // fullscreen
    Effect::new(move |_| {
        if let Some(el) = container_ref.get() {
            if is_fullscreen.get() {
                if !is_element_fullscreen(el.clone()) {
                    el.request_fullscreen();
                }
            } else {
                if is_element_fullscreen(el) {
                    document().exit_fullscreen();
                }
            }
        }
    });

    // playing
    Effect::new(move |_| {
        if is_playing.get() {
            play();
        } else {
            pause();
        }
    });

    // loop
    Effect::new(move |_| {
        if let Some(video) = video_ref.get() {
            video.set_loop(is_loop.get());
        }
    });

    // mute
    Effect::new(move |_| {
        if let Some(video) = video_ref.get() {
            video.set_muted(mute.get());
        }
    });

    // volume
    Effect::new(move |_| {
        if let Some(video) = video_ref.get() {
            video.set_volume(volume.get());
        }
    });

    view! {
        <div
            node_ref=container_ref
            autofocus
            tabindex="-1"
            class="relative size-full flex bg-black flex-col overflow-hidden touch-none group"
            on:fullscreenchange=move |_| is_fullscreen.set(is_element_fullscreen(container_ref))
            on:keydown=handle_keydown
            on:mousemove=on_mouse_move
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
                    preload="metadata"
                    class="cursor-pointer absolute size-full"
                    // style=move || { if display_proxy.get() { "display:none;" } else { "" } }

                    on:contextmenu=move |ev| ev.prevent_default()
                    on:loadedmetadata=move |_| load_metadata()
                    on:durationchange=move |_| load_metadata()
                    on:pause=move |_| { log!("paused") }
                    on:play=move |_| { log!("played") }
                    on:click=move |_| toggle_play()
                    on:progress=move |_| { sync_preload_progress() }
                    on:ended=move |_| {
                        log!("ended");
                        if !is_dragging.get_untracked() {
                            is_playing.set(false);
                        }
                    }
                    on:waiting=move |_| {
                        log!("Waiting video...");
                        set_timeout(
                            move || {
                                if video_ref.get().unwrap().ready_state()
                                    <= HtmlMediaElement::HAVE_CURRENT_DATA
                                {
                                    is_waiting.set(true);
                                }
                            },
                            Duration::from_secs(1),
                        );
                    }

                    on:stalled={
                        let timeout = StoredValue::new(None::<TimeoutHandle>);
                        let reset_timer = move || {
                            log!("Waiting 10 sec before recover stalled video...");
                            if let Some(timeout) = timeout.get_value() {
                                timeout.clear();
                            }
                            timeout
                                .set_value(
                                    set_timeout_with_handle(
                                            move || {
                                                if video_ref.get().unwrap().ready_state()
                                                    < HtmlMediaElement::HAVE_CURRENT_DATA
                                                {
                                                    log!("Attempting to recover stalled video...");
                                                    recover(video_ref);
                                                }
                                            },
                                            Duration::from_secs(10),
                                        )
                                        .ok(),
                                );
                        };
                        move |_| reset_timer()
                    }
                    // on:suspend=move |_| log!("suspend")
                    on:canplay=move |_| {
                        is_waiting.set(false);
                        sync_preload_progress()
                    }
                    on:playing=move |_| {
                        is_waiting.set(false);
                    }
                    on:seeked=move |_| {
                        log!("seeked video");
                        display_proxy.set(false);
                        if !is_dragging.get_untracked() && is_playing.get_untracked() {
                            play();
                        }
                    }
                />
                <Show when=move || proxy.get() != "">
                    <video
                        playsinline
                        disablepictureinpicture
                        controlslist="nodownload"
                        node_ref=proxy_ref
                        src=proxy
                        preload="auto"
                        class="cursor-pointer absolute size-full pointer-events-none"
                        style:opacity=move || { if display_proxy.get() { "1" } else { "0" } }
                        on:seeked=move |_| {
                            log!("seeked proxy");
                            if is_dragging.get_untracked() {
                                display_proxy.set(true);
                            }
                        }
                    />

                </Show>
                // <div
                // class="absolute bg-red-500/50"
                // style=move || {
                // let (w, h, x, y) = calc_video_transform(
                // width.get(),
                // height.get(),
                // aspect.get(),
                // );
                // format!("width:{w}px; height: {h}px; translate:{x}px {y}px;")
                // }
                // ></div>
                <Show when=move || is_waiting.get()>
                    <div class="absolute inset-0 left-1/2 top-1/2">
                        <spinner::Ring />
                    </div>
                </Show>
            </div>

            // Controls
            <div
                node_ref=controls_ref
                tabindex="-1"
                class=move || {
                    if overlay.get() {
                        let o = if is_dragging.get() || show_overlay_controls.get() {
                            "100"
                        } else {
                            "0"
                        };
                        format!(
                            "flex-none outline-none bottom-0 absolute inset-x-0 w-full px-6 pt-20 bg-gradient-to-t from-black transition-opacity duration-200 focus:opacity-100 hover:opacity-100 opacity-{o}",
                        )
                    } else {
                        "flex-none outline-none bottom-0 px-6 opacity-100".into()
                    }
                }
            >

                <div class="relative">
                    // Progress bar
                    <ProgressBar
                        frame=frame
                        metadata=metadata
                        preload=preload_progress.read_only()
                        // hover=progress_hover
                        node_ref=progress_ref
                        overlay=overlay
                        thumb_src=proxy
                        is_dragging=is_dragging
                        on_dragging_start=move |f| {
                            container_ref.get().unwrap().focus();
                            if let Some(video) = video_ref.get() {
                                video.pause();
                            }
                            if let Some(_) = proxy_ref.get() {
                                set_current_frame(proxy_ref, f);
                            } else {
                                set_current_frame(video_ref, f);
                            }
                            true
                        }
                        on_dragging_move={
                            let set_current_frame_throttled = use_throttle_fn(
                                move || {
                                    let f = frame.get_untracked();
                                    log!("throttled: {f}");
                                    if let Some(_) = proxy_ref.get() {
                                        set_current_frame(proxy_ref, f);
                                    } else {
                                        set_current_frame(video_ref, f);
                                    }
                                },
                                50.0,
                            );
                            move |_| {
                                set_current_frame_throttled();
                            }
                        }
                        on_dragging_end=move |f| {
                            set_current_frame(video_ref, f);
                        }
                    />

                    <div class="flex items-center justify-between h-4"></div>
                    // Control buttons
                    <div class="flex items-center justify-between pb-1 pt-2 bottom-0">
                        // Left side
                        <div class="flex items-center space-x-4">
                            <PlayPauseToggle is_playing=is_playing />
                            <LoopToggle is_loop=is_loop />
                            <VolumeControl volume=volume mute=mute />

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
                            <FullscreenToggle is_fullscreen=is_fullscreen />
                        </div>
                    </div>
                </div>
            </div>
        </div>
    }
}

#[component]
fn ProgressBar<S, M, E>(
    node_ref: NodeRef<html::Div>,
    metadata: VideoMetadata,
    frame: RwSignal<i32>,
    // #[prop(into)] hover: RwSignal<Hover>,
    preload: ReadSignal<f64>,
    #[prop(into, optional)] thumb_src: Signal<String>,
    #[prop(into)] overlay: Signal<bool>,
    is_dragging: RwSignal<bool>,
    on_dragging_start: S,
    on_dragging_move: M,
    on_dragging_end: E,
) -> impl IntoView
where
    S: Fn(i32) -> bool + Send + Sync + 'static,
    M: Fn(i32) + Send + Sync + 'static,
    E: Fn(i32) + Send + Sync + 'static,
{
    #[derive(Debug, Clone, PartialEq, Eq)]
    enum Hover {
        Enter(i32),
        Move(i32),
        Exit(i32),
    }
    let thumb_video_ref = NodeRef::<html::Video>::new();
    let hover = RwSignal::new(Hover::Exit(0));
    let end_frame = metadata.end_frame;
    let fps = metadata.fps;
    let aspect = metadata.aspect;

    let set_current_frame_throttled = use_throttle_fn_with_arg(
        move |f: i32| {
            if let Some(thumb_video) = thumb_video_ref.get() {
                let fps = fps.get_untracked();
                let time = (f as f64) / fps;
                thumb_video.set_current_time(time);
            }
        },
        50.0,
    );

    use_draggable_with_options(
        node_ref,
        UseDraggableOptions::default()
            .initial_value(Position { x: 0.0, y: 0.0 })
            .target_offset(move |_| (0.0, 0.0))
            .on_start(move |args| {
                is_dragging.set(true);

                if let Some(p) = node_ref.get() {
                    let pos = args.event.offset_x() as f64 / p.client_width() as f64;
                    let f = frame_from_pos(pos, end_frame.get_untracked());
                    frame.set(f);
                    let result = on_dragging_start(f);
                    if args.event.pointer_type() == "touch" {
                        let _ = p.focus();
                    }
                    result
                } else {
                    false
                }
            })
            .on_move(move |args| {
                if let Some(p) = node_ref.get() {
                    let x = args.event.client_x() - p.get_bounding_client_rect().left() as i32;
                    let pos = x as f64 / p.client_width() as f64;
                    let f = frame_from_pos(pos, end_frame.get_untracked());
                    frame.set(f);
                    on_dragging_move(f);
                }
            })
            .on_end(move |args| {
                if let Some(p) = node_ref.get() {
                    let x = args.event.client_x() - p.get_bounding_client_rect().left() as i32;
                    hover.set(Hover::Exit(x));
                    is_dragging.set(false);
                    let pos = x as f64 / p.client_width() as f64;
                    let f = frame_from_pos(pos, end_frame.get_untracked());
                    frame.set(f);
                    on_dragging_end(f);
                }
            })
            .stop_propagation(true)
            .prevent_default(true),
    );

    view! {
        <div>
            <div
                node_ref=node_ref
                tabindex="-1"
                class=move || {
                    format!(
                        "absolute outline-none group/progress origin-bottom w-full h-1 expand-clickable-area hover:scale-y-200 focus:scale-y-200 cursor-pointer transform transition-all duration-200 {}",
                        if overlay.get() { "bg-white/50" } else { "bg-neutral-700" },
                    )
                }

                on:mouseover=move |ev| {
                    log!("over");
                    hover.set(Hover::Enter(ev.offset_x()))
                }
                on:mousemove=move |ev| {
                    log!("move");
                    let x = ev.offset_x();
                    if let Some(p) = node_ref.get() {
                        let pos = x as f64 / p.client_width() as f64;
                        let f = frame_from_pos(pos, end_frame.get_untracked());
                        set_current_frame_throttled(f);
                    }
                    hover.set(Hover::Move(x));
                }
                on:mouseout=move |ev| {
                    log!("out");
                    hover.set(Hover::Exit(ev.offset_x()))
                }
            >
                <div
                    class="absolute origin-left h-full w-full bg-white/30 transition-scale duration-200 pointer-events-none"
                    style:scale=move || { format!("{} 1", preload.get()) }
                />

                // Preload
                <div
                    class="absolute origin-left h-full w-full bg-white/30 transition-scale duration-200 pointer-events-none"
                    style:scale=move || { format!("{} 1", preload.get()) }
                />

                // Progress
                <div
                    class="absolute origin-left h-full w-full bg-main-accent pointer-events-none"
                    style:scale=move || {
                        format!("{} 1", frame.get() as f64 / (end_frame.get() + 1) as f64)
                    }
                />

                // Cursor
                <div
                    class="absolute origin-left h-full w-full pointer-events-none"
                    style:translate=move || {
                        format!("{}% 0", (100 * frame.get()) as f64 / (end_frame.get() + 1) as f64)
                    }
                >
                    <div
                        class="h-full origin-left bg-white pointer-events-none"
                        style:width=move || {
                            format!("calc(max({}%,3px))", 100.0 / (end_frame.get() + 1) as f64)
                        }
                    />
                </div>

            </div>
            <Show when=move || thumb_src.get() != "">
                <div
                    class="absolute rounded-sm outline-solid outline-2 outline-neutral-300 drop-shadow-xl
                    overflow-hidden pointer-events-none transition-opacity duration-200 delay-100"
                    style=move || {
                        if let Some(p) = node_ref.get() {
                            let p_width = p.client_width();
                            let (w, h) = thumbnail_size(aspect.get());
                            if w == 0 {
                                return "opacity:0".into();
                            }
                            let (x, o) = match hover.get() {
                                Hover::Move(x) => (x, (!is_dragging.get()).into()),
                                Hover::Enter(x) => (x, 0),
                                Hover::Exit(x) => (x, 0),
                            };
                            let x = 0.max(x - w / 2).min(p_width - w);
                            let y = -(h + 32);
                            format!("width:{w}px;height:{h}px;translate:{x}px {y}px;opacity:{o};")
                        } else {
                            "opacity:0".into()
                        }
                    }
                >
                    <video
                        node_ref=thumb_video_ref
                        playsinline
                        disablepictureinpicture
                        controlslist="nodownload"
                        src=thumb_src
                        preload="auto"
                    />

                </div>
            </Show>
        </div>
    }
}

#[component]
pub fn PlayPauseToggle(is_playing: RwSignal<bool>) -> impl IntoView {
    view! {
        <button
            class="size-12 p-1 hover:text-white text-neutral-200 transition-colors rounded-xl cursor-pointer outline-none drop-shadow-(--icon-shadow)"
            type="button"
            on:click=move |_| {
                let p = !is_playing.get_untracked();
                is_playing.set(p);
            }
            on:keydown=move |ev| ev.prevent_default()
        >

            {move || {
                if is_playing.get() {
                    Either::Left(view! { <icon::Pause /> })
                } else {
                    Either::Right(view! { <icon::Play /> })
                }
            }}
        </button>
    }
}

// #[component]
// pub fn NextFrameButton(is_playing: RwSignal<bool>) -> impl IntoView {
//     view! {
//         <button
//             class="size-12 p-1 hover:text-white text-neutral-200 transition-colors rounded-xl cursor-pointer outline-none drop-shadow-(--icon-shadow)"
//             type="button"
//             on:click=move |_| {
//                 let p = !is_playing.get_untracked();
//                 is_playing.set(p);
//             }
//             on:keydown=move |ev| ev.prevent_default()
//         >

//             {move || {
//                 if is_playing.get() {
//                     Either::Left(view! { <icon::Pause /> })
//                 } else {
//                     Either::Right(view! { <icon::Play /> })
//                 }
//             }}
//         </button>
//     }
// }

#[component]
pub fn FullscreenToggle(is_fullscreen: RwSignal<bool>) -> impl IntoView {
    view! {
        <button
            class="size-10 p-1 hover:text-white text-neutral-200 transition-colors rounded-xl cursor-pointer outline-none drop-shadow-(--icon-shadow)"
            type="button"
            on:click=move |_| {
                let p = !is_fullscreen.get_untracked();
                is_fullscreen.set(p);
            }
            on:keydown=move |ev| ev.prevent_default()
        >
            {move || {
                if is_fullscreen.get() {
                    Either::Left(view! { <icon::FullScreenExit /> })
                } else {
                    Either::Right(view! { <icon::FullScreenEnter /> })
                }
            }}
        </button>
    }
}

#[component]
pub fn LoopToggle(is_loop: RwSignal<bool>) -> impl IntoView {
    view! {
        <button
            class="size-10 p-2 hover:text-white text-neutral-200 transition-colors rounded-xl cursor-pointer outline-none drop-shadow-(--icon-shadow)"
            type="button"
            style:opacity=move || { if is_loop.get() { "1" } else { "0.7" } }
            on:click=move |_| {
                let r = !is_loop.get_untracked();
                is_loop.set(r);
            }
            on:keydown=move |ev| ev.prevent_default()
        >
            <icon::Loop />
        </button>
    }
}

#[component]
fn VolumeControl(volume: RwSignal<f64>, mute: RwSignal<bool>) -> impl IntoView {
    view! {
        <div class="relative group/volume">
            <button
                class="size-10 p-2 hover:text-white text-neutral-200 transition-colors rounded-xl mr-2 cursor-pointer outline-none drop-shadow-(--icon-shadow)"
                type="button"
                on:click=move |_| toggle_mute(mute, volume)
                on:keydown=move |ev| ev.prevent_default()
            >

                {move || {
                    let vol = volume.get();
                    if mute.get() || vol <= 0.0 {
                        EitherOf3::A(view! { <icon::Volume0 /> })
                    } else if vol < 0.5 {
                        EitherOf3::B(view! { <icon::Volume1 /> })
                    } else {
                        EitherOf3::C(view! { <icon::Volume2 /> })
                    }
                }}
            </button>

            <div class="absolute opacity-0 group-hover/volume:opacity-100 group-hover/volume:delay-0 delay-500 transition-opacity duration-300 h-full w-16 top-0 left-11 flex items-center outline-none">
                <input
                    type="range"
                    min="0.0"
                    max="1.0"
                    step="0.01"
                    prop:value=move || if mute.get() { 0.0 } else { volume.get() }
                    on:input=move |ev| {
                        ev.stop_propagation();
                        let target = event_target::<HtmlInputElement>(&ev);
                        let vol = target.value_as_number();
                        let m = vol == 0.0;
                        if mute.get_untracked() != m {
                            mute.set(m);
                        }
                        volume.set(vol);
                    }
                    on:keydown=move |ev| ev.prevent_default()
                    class="appearance-none hover:text-white text-neutral-200 outline-none rounded-full"
                />
            </div>
        </div>
    }
}

fn toggle_mute(mute: RwSignal<bool>, volume: RwSignal<f64>) {
    let m = !mute.get_untracked();
    if !m && volume.get_untracked() == 0.0 {
        volume.set(1.0);
    }
    mute.set(m);
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

fn frame_from_pos(pos: f64, end_frame: i32) -> i32 {
    let pos = pos.max(0.0);
    let total_frames = end_frame + 1;
    ((pos * total_frames as f64).next_up() as i32).min(end_frame)
}

fn frame_from_time(time: f64, fps: f64) -> i32 {
    (time * fps).next_up() as i32
}

fn thumbnail_size(aspect: f64) -> (i32, i32) {
    if aspect > 1.0 {
        (
            THUMB_MAX_SIZE,
            (THUMB_MAX_SIZE as f64 / aspect).round() as i32,
        )
    } else {
        (
            (THUMB_MAX_SIZE as f64 * aspect).round() as i32,
            THUMB_MAX_SIZE,
        )
    }
}

fn discretize_time(time: f64, fps: f64) -> f64 {
    (time * fps).next_up().floor() / fps
}

fn frame_from_time_rounded(time: f64, fps: f64) -> i32 {
    (time * fps).round() as i32
}

fn calc_video_transform(el_width: f64, el_height: f64, video_aspect: f64) -> (f64, f64, f64, f64) {
    let el_aspect = el_width / el_height;
    if video_aspect < el_aspect {
        let w = video_aspect * el_height;
        let h = el_height;
        let x = (el_width - w) / 2.0;
        let y = 0.0;
        (w, h, x, y)
    } else {
        let w = el_width;
        let h = el_width / video_aspect;
        let x = 0.0;
        let y = (el_height - h) / 2.0;
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
