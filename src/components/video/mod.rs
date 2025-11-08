#![allow(unused_must_use)]
use super::icon::{self, *};
use crate::timecode::*;
use crate::utils::*;
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
use web_sys::{
    self, HtmlElement, HtmlInputElement, HtmlMediaElement, HtmlVideoElement, MouseEvent,
};
use web_time::{Duration, Instant};

const THUMB_MAX_SIZE: i32 = 200;

#[derive(Clone, Debug, Default)]
pub struct VideoMetadata {
    pub aspect: RwSignal<f64>,
    pub fps: Signal<f64>,
    pub end_frame: RwSignal<u32>,
    pub time_format: RwSignal<TimeFormat>,
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

    let frame = RwSignal::new(0);
    let end_frame = RwSignal::new(0);
    let aspect = RwSignal::new(0.0);
    let playback_rate = RwSignal::new(1.0);
    let time_format = RwSignal::new(TimeFormat::default());
    let preload_progress = RwSignal::new(0.0);
    let volume = RwSignal::new(0.5);
    let mute = RwSignal::new(false);
    let is_playing = RwSignal::new(false);
    let is_loop = RwSignal::new(true);
    let is_dragging = RwSignal::new(false);
    let is_fullscreen = RwSignal::new(false);
    let display_proxy = RwSignal::new(false);
    let is_waiting = RwSignal::new(false);
    let hide_overlay_controls = RwSignal::new(false);
    let overlay = Signal::derive(move || overlay_controls.get() || is_fullscreen.get());
    let metadata = VideoMetadata {
        fps,
        end_frame,
        aspect,
        time_format,
    };

    let UseElementSizeReturn { width, height } = use_element_size(video_ref);

    let set_current_frame = move |video_ref: NodeRef<html::Video>, f: u32| {
        if let Some(video) = video_ref.get_untracked() {
            let fps = fps.get_untracked();
            let time = time_from_frame(f, fps);
            video.set_current_time(time);
        }
    };

    let load_metadata = move || {
        if let Some(video) = video_ref.get_untracked() {
            let d = video.duration();
            if d.is_finite() {
                let total_frames = (d * fps.get()).next_up() as u32;
                end_frame.set((total_frames - 1).max(0));
                aspect.set(video.video_width() as f64 / video.video_height() as f64);
            }
        }
    };

    let update_preload_progress = move || {
        if let Some(video) = video_ref.get_untracked() {
            let vb = video.buffered();
            let time = video.current_time();
            let fps = fps.get_untracked();
            let end_frame = end_frame.get_untracked();
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

    let reload = move |video_ref: NodeRef<html::Video>| {
        if let Some(video) = video_ref.get_untracked() {
            let time = video.current_time();
            video.load();
            video.set_current_time(time);
            if is_playing.get_untracked() {
                video.play();
            }
        }
    };

    let is_ended = move || {
        let video = video_ref.get_untracked().unwrap();
        video.ended() || frame.get_untracked() >= end_frame.get_untracked()
    };

    let stop = move || {
        let video = video_ref.get_untracked().unwrap();
        video.pause();
        set_current_frame(video_ref, 0);
    };

    let play = move || {
        if let Some(video) = video_ref.get_untracked() {
            if !is_loop.get_untracked() && is_ended() {
                set_current_frame(video_ref, 0);
            }
            video.play();
        }
    };

    let pause = move || {
        log!("press_pause");
        if let Some(video) = video_ref.get_untracked() {
            video.pause();
        }
    };

    let toggle_play = move || {
        is_playing.update(|p| *p = !*p);
    };

    let toggle_fullscreen = move || {
        is_fullscreen.update(|f| *f = !*f);
    };

    let precised_pause = move || {
        log!("precised_pause");
        if let Some(video) = video_ref.get_untracked() {
            video.pause();

            let fps = fps.get_untracked();
            let t = video.current_time();
            let timestep = 0.51 / fps;
            let new_t = t + timestep;
            let t = if new_t < video.duration() { new_t } else { t };
            // set_current_frame(video_ref, f.max(frame.get_untracked()));

            video.set_current_time(t);
        }
    };

    let next_frame = move || {
        if let Some(video) = video_ref.get_untracked() {
            video.pause();
            if is_playing.get_untracked() {
                is_playing.set(false);
            }

            let end_frame = end_frame.get_untracked();
            let f = frame.get_untracked();
            if f >= end_frame {
                return;
            }

            let f = f + 1;
            let fps = fps.get_untracked();
            let time = time_from_frame(f, fps);
            video.set_current_time(time);
        }
    };

    let prev_frame = move || {
        if let Some(video) = video_ref.get_untracked() {
            video.pause();
            if is_playing.get_untracked() {
                is_playing.set(false);
            }

            let f = frame.get_untracked();
            if f == 0 {
                return;
            }
            let f = f - 1;
            let fps = fps.get_untracked();
            let time = time_from_frame(f, fps);
            video.set_current_time(time);
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

    // on create
    Effect::new(move |_| {
        load_metadata();
        if let Some(video) = video_ref.get_untracked() {
            use_video_frame_fn(video, move |time| {
                log!("prcise time begin");
                if is_dragging.get_untracked() {
                    return;
                }
                let fps = fps.get_untracked();
                let end_frame = end_frame.get_untracked();
                let f = frame_from_time(time, fps).min(end_frame);
                frame.set(f);

                log!("prcise time: {time}, frame: {f}");
            });
        }
    });

    // fullscreen
    Effect::new(move |_| {
        if is_fullscreen.get() {
            request_fullscreen(container_ref.get_untracked());
        } else if document().fullscreen() {
            document().exit_fullscreen();
        }
    });

    // playing
    Effect::new(move |_| {
        if is_playing.get() {
            play();
        } else {
            // pause();
            precised_pause();
        }
    });

    // loop
    Effect::new(move |_| {
        if let Some(video) = video_ref.get_untracked() {
            video.set_loop(is_loop.get());
        }
    });

    // mute
    Effect::new(move |_| {
        if let Some(video) = video_ref.get_untracked() {
            video.set_muted(mute.get());
        }
    });

    // volume
    Effect::new(move |_| {
        if let Some(video) = video_ref.get_untracked() {
            video.set_volume(volume.get());
        }
    });

    // playback rate
    Effect::new(move |_| {
        if let Some(video) = video_ref.get_untracked() {
            video.set_playback_rate(playback_rate.get());
        }
    });

    // start/end seek
    Effect::new(move |_| {
        if is_dragging.get() {
            // start seek
            pause();
            let f = frame.get_untracked();
            // container_ref.get_untracked().unwrap().focus();
            if let Some(_) = proxy_ref.get_untracked() {
                set_current_frame(proxy_ref, f);
            } else {
                set_current_frame(video_ref, f);
            }
        } else {
            reset_overlay_controls_timeout();
            // end seek
            let f = frame.get_untracked();
            let f = if is_playing.get_untracked() {
                (f + 1).min(end_frame.get_untracked())
            } else {
                f
            };
            set_current_frame(video_ref, f);
        }
    });

    // seek
    Effect::new(move |_| {
        let f = frame.get();
        if is_dragging.get_untracked() {
            if let Some(_) = proxy_ref.get_untracked() {
                set_current_frame(proxy_ref, f);
            } else {
                set_current_frame(video_ref, f);
            }
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
            // Video element
            <div
                class="relative flex-auto cursor-pointer select-none"
                on:contextmenu=move |ev| ev.prevent_default()
                on:click=onclick_handler(on_click, on_dblclick)
                on:dblclick=move |ev| ev.prevent_default()
            >
                <video
                    // controls
                    playsinline
                    disablepictureinpicture
                    node_ref=video_ref
                    src=src
                    preload="metadata"
                    class="absolute size-full pointer-events-none"
                    // style=move || { if display_proxy.get() { "display:none;" } else { "" } }

                    on:loadedmetadata=move |_| load_metadata()
                    on:durationchange=move |_| load_metadata()
                    on:timeupdate=move |_| {
                        log!("timeupdate at: {}", video_ref.get_untracked().unwrap().current_time())
                    }
                    on:pause=move |_| {
                        log!("paused at: {}", video_ref.get_untracked().unwrap().current_time())
                    }
                    on:play=move |_| { log!("played") }

                    on:progress=move |_| { update_preload_progress() }
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
                                if video_ref.get_untracked().unwrap().ready_state()
                                    <= HtmlMediaElement::HAVE_CURRENT_DATA
                                {
                                    is_waiting.set(true);
                                }
                            },
                            Duration::from_secs(1),
                        );
                    }

                    on:stalled=move |_| {}
                    // on:suspend=move |_| log!("suspend")
                    on:canplay=move |_| {
                        log!("canplay at: {}", video_ref.get_untracked().unwrap().current_time());
                        is_waiting.set_changed(false);
                        update_preload_progress()
                    }
                    on:playing=move |_| {
                        is_waiting.set_changed(false);
                    }
                    on:seeked=move |_| {
                        log!("seeked video");
                        display_proxy.set_changed(false);
                        if let Some(video) = video_ref.get_untracked() {
                            log!("seeked video time: {}", video.current_time());
                            if !video.paused() {
                                return;
                            }
                        }
                        if !is_dragging.get_untracked() && is_playing.get_untracked() {
                            play();
                        }
                    }
                />
                <Show when=move || proxy.get() != "">
                    <video
                        playsinline
                        disablepictureinpicture
                        node_ref=proxy_ref
                        src=proxy
                        preload="auto"
                        class="absolute size-full pointer-events-none"
                        class:opacity-0=move || !display_proxy.get()
                        on:seeked=move |_| {
                            log!(
                                "seeked proxy: {}", proxy_ref.get_untracked().unwrap().current_time()
                            );
                            if is_dragging.get_untracked() {
                                display_proxy.set_changed(true);
                            }
                        }
                    />
                </Show>

                // Canvas container
                <div
                    class="absolute pointer-events-none"
                    style=move || {
                        let (w, h, x, y) = calc_video_box(width.get(), height.get(), aspect.get());
                        format!("width:{w}px;height:{h}px;translate:{x}px {y}px")
                    }
                ></div>

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
                    style=move || { format!("width:{}px;height:{}px;", width.get(), height.get()) }
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

                    <div class="relative">
                        <ProgressBar
                            node_ref=progress_ref
                            frame
                            metadata
                            preload=preload_progress.read_only()
                            overlay
                            thumb_src=proxy
                            is_dragging
                        />

                        <div class="flex items-center justify-between h-4"></div>
                        // Control buttons
                        <div class="flex items-center justify-between">
                            // Left side
                            <div class="shrink-0 flex items-center space-x-1">
                                <PlayPauseToggle is_playing />

                                <PlybackRateControl playback_rate />
                                <PrevNextFrameButtonsGrp
                                    on_prev_click=prev_frame
                                    on_next_click=next_frame
                                />
                                <LoopToggle is_loop />
                                <VolumeControl volume mute />
                            </div>

                            // Center
                            <div class="shrink flex items-center min-w-0">
                                <TimecodeControl frame end_frame fps time_format />
                            </div>

                            // Right side
                            <div class="shrink-0 flex items-center space-x-0">
                                <span class="mr-2 text-base font-medium text-gray-500 text-nowrap drop-shadow-ico hidden @3xl:block">
                                    {move || format!("{}fps", fps.get())}
                                </span>
                                <Settings />
                                <FullscreenToggle is_fullscreen />
                            </div>
                        </div>
                    </div>
                </div>
            </div>
        </div>
    }
}

#[component]
fn ProgressBar(
    node_ref: NodeRef<html::Div>,
    metadata: VideoMetadata,
    frame: RwSignal<u32>,
    preload: ReadSignal<f64>,
    #[prop(into, optional)] thumb_src: Signal<String>,
    #[prop(into)] overlay: Signal<bool>,
    is_dragging: RwSignal<bool>,
) -> impl IntoView {
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
    let thumb_frame = RwSignal::new(0);

    let set_frame_throttled = use_throttle_fn_with_arg(
        move |f: u32| {
            frame.set(f);
        },
        100.0,
    );

    let set_thumb_frame_throttled = use_throttle_fn_with_arg(
        move |f: u32| {
            thumb_frame.set(f);
        },
        100.0,
    );

    Effect::new(move |_| {
        let fps = fps.get_untracked();
        let time = time_from_frame(thumb_frame.get(), fps);
        if let Some(thumb_video) = thumb_video_ref.get() {
            thumb_video.set_current_time(time);
        }
    });

    use_draggable_with_options(
        node_ref,
        UseDraggableOptions::default()
            .initial_value(Position { x: 0.0, y: 0.0 })
            .target_offset(move |_| (0.0, 0.0))
            .on_start(move |args| {
                is_dragging.set(true);

                if let Some(p) = node_ref.get_untracked() {
                    let pos = args.event.offset_x() as f64 / p.client_width() as f64;
                    let f = frame_from_pos(pos, end_frame.get_untracked());
                    frame.set(f);
                    // let result = on_dragging_start(f);
                    if args.event.pointer_type() == "touch" {
                        let _ = p.focus();
                    }
                    // result
                    true
                } else {
                    false
                }
            })
            .on_move(move |args| {
                if let Some(p) = node_ref.get_untracked() {
                    let x = args.event.client_x() - p.get_bounding_client_rect().left() as i32;
                    let pos = x as f64 / p.client_width() as f64;
                    let f = frame_from_pos(pos, end_frame.get_untracked());
                    set_frame_throttled(f);
                    // on_dragging_move(f);
                }
            })
            .on_end(move |args| {
                if let Some(p) = node_ref.get_untracked() {
                    let x = args.event.client_x() - p.get_bounding_client_rect().left() as i32;
                    hover.set(Hover::Exit(x));
                    is_dragging.set(false);
                    let pos = x as f64 / p.client_width() as f64;
                    let f = frame_from_pos(pos, end_frame.get_untracked());
                    frame.set(f);
                    // on_dragging_end(f);
                }
            })
            .stop_propagation(true)
            .prevent_default(true),
    );

    view! {
        <div class="relative">
            <div
                node_ref=node_ref
                tabindex="-1"
                class="absolute outline-none group/progress origin-bottom w-full h-1 expand-clickable-area hover:scale-y-200 focus:scale-y-200 cursor-pointer transition-scale duration-200"

                on:mouseover=move |ev| {
                    log!("over");
                    hover.set(Hover::Enter(ev.offset_x()))
                }
                on:mousemove=move |ev| {
                    log!("move");
                    let x = ev.offset_x();
                    if let Some(p) = node_ref.get_untracked() {
                        let pos = x as f64 / p.client_width() as f64;
                        let f = frame_from_pos(pos, end_frame.get_untracked());
                        set_thumb_frame_throttled(f);
                    }
                    hover.set(Hover::Move(x));
                }
                on:mouseout=move |ev| {
                    log!("out");
                    hover.set(Hover::Exit(ev.offset_x()))
                }
            >
                // Track
                <div
                    class="absolute size-full bg-neutral-600 pointer-events-none"
                    class=("bg-white/25", move || overlay.get())
                />
                // Preload
                <div
                    class="absolute origin-left size-full bg-white/25 transition-scale duration-200 pointer-events-none"
                    style:scale=move || { format!("{} 1", preload.get()) }
                />

                // Progress
                <div
                    class="absolute origin-left size-full bg-primary pointer-events-none"
                    style:scale=move || {
                        format!("{} 1", frame.get() as f64 / (end_frame.get() + 1) as f64)
                    }
                />

                // Cursor
                <div
                    class="absolute origin-left size-full pointer-events-none"
                    style:translate=move || {
                        format!("{}% 0", (100 * frame.get()) as f64 / (end_frame.get() + 1) as f64)
                    }
                >
                    <div
                        class="h-full origin-left bg-white pointer-events-none"
                        style:width=move || {
                            let end_frame = end_frame.get();
                            if end_frame == 0 {
                                "0".into()
                            } else {
                                format!("calc(max({}%,3px))", 100.0 / (end_frame + 1) as f64)
                            }
                        }
                    />
                </div>

            </div>
            <Show when=move || thumb_src.get() != "">
                <div
                    class="absolute transition-opacity duration-200 delay-100 flex flex-col items-center gap-2"
                    style=move || {
                        if let Some(p) = node_ref.get_untracked() {
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
                            let y = -(h + 40);
                            format!("width:{w}px;translate:{x}px {y}px;opacity:{o};")
                        } else {
                            "opacity:0".into()
                        }
                    }
                >
                    <div
                        class="rounded-sm outline-solid outline-2 outline-neutral-300 drop-shadow-xl/50
                        overflow-hidden pointer-events-none"
                        style=move || {
                            if let Some(p) = node_ref.get_untracked() {
                                let (w, h) = thumbnail_size(aspect.get());
                                format!("height:{h}px;")
                            } else {
                                "".into()
                            }
                        }
                    >
                        <video
                            node_ref=thumb_video_ref
                            playsinline
                            disablepictureinpicture
                            src=thumb_src
                            preload="auto"
                        />
                    </div>
                    <div class="text-base font-bold drop-shadow-ico text-neutral-300 text-center">
                        {
                            let time_format = metadata.time_format;
                            move || {
                                timecode_str(
                                    thumb_frame.get(),
                                    fps.get(),
                                    end_frame.get(),
                                    time_format.get(),
                                    true,
                                )
                            }
                        }
                    </div>
                </div>
            </Show>
        </div>
    }
}

#[component]
fn PlayPauseToggle(is_playing: RwSignal<bool>) -> impl IntoView {
    view! {
        <button
            class="btn-player size-10 p-1"
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

#[component]
fn PrevNextFrameButtonsGrp<FP, FN>(on_prev_click: FP, on_next_click: FN) -> impl IntoView
where
    FP: Fn() + 'static,
    FN: Fn() + 'static,
{
    view! {
        <div class="*:btn-player *:w-6 *:h-10 *:p-0 hidden @3xl:flex items-center space-x-1">
            <button on:click=move |_| on_prev_click() on:keydown=move |ev| ev.prevent_default()>
                <icon::NextFrame class="-scale-x-100" />
            </button>
            <button on:click=move |_| on_next_click() on:keydown=move |ev| ev.prevent_default()>
                <icon::NextFrame />
            </button>
        </div>
    }
}

#[component]
fn PlybackRateControl(playback_rate: RwSignal<f64>) -> impl IntoView {
    view! {
        <div class="dropdown dropdown-top dropdown-start hidden @2xl:block">
            <button
                tabindex="0"
                class="btn-player w-16 h-10 p-1 text-base font-medium  text-gray-400"
            >
                <span>{move || format!("{:?}x", playback_rate.get())}</span>
            </button>

            <div
                tabindex="-1"
                class="dropdown-content menu bg-base-100 rounded-box z-1 min-w-90 p-2 drop-shadow-xl/50"
            >
                <div class="menu-title">Playback speed</div>
                <div class="flex flex-col p-2 size-full">
                    <div class="flex justify-between *:h-7 *:text-sm *:font-normal *:basis-0 text-current/70 ">
                        {(1..8)
                            .into_iter()
                            .map(|i| {
                                let ratio = i as f64 * 0.25;
                                view! {
                                    <button
                                        class="btn btn-ghost p-1"
                                        on:click=move |_| playback_rate.set(ratio)
                                    >
                                        {format!("{ratio:?}")}
                                    </button>
                                }
                            })
                            .collect_view()}
                    </div>
                    <input
                        type="range"
                        min="0.25"
                        max="1.75"
                        value="1.0"
                        class="[--track-height:0.5rem] px-2"
                        step="0.05"
                        prop:value=move || playback_rate.get()
                        on:input=move |ev| {
                            ev.stop_propagation();
                            let target = event_target::<HtmlInputElement>(&ev);
                            let value = target.value_as_number();
                            playback_rate.set(value);
                        }
                        on:keydown=move |ev| ev.prevent_default()
                    />
                    <div class="absolute bottom-5 -translate-y-px w-auto inset-x-0 left-6 right-6 px-1.5 flex justify-evenly *:w-0.5 h-2 *:bg-current/20 pointer-events-none">
                        {(0..5).into_iter().map(|_| view! { <span></span> }).collect_view()}
                    </div>
                </div>
            </div>
        </div>
    }
}

#[component]
fn LoopToggle(is_loop: RwSignal<bool>) -> impl IntoView {
    view! {
        <button
            class="btn-player size-10 p-1.5 hidden @xl:block"
            on:click=move |_| {
                let r = !is_loop.get_untracked();
                is_loop.set(r);
            }
            on:keydown=move |ev| ev.prevent_default()
        >
            <icon::Loop enable=is_loop />
        </button>
    }
}

#[component]
fn VolumeControl(volume: RwSignal<f64>, mute: RwSignal<bool>) -> impl IntoView {
    let val = move || if mute.get() { 0.0 } else { volume.get() };
    view! {
        <div class="relative group/volume h-10 mobile:hidden">
            <button
                class="btn-player size-10 p-2"
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

            <div class="absolute h-full opacity-0 delay-500 rounded-md transition-opacity duration-200 top-0 flex items-center group-hover/volume:opacity-100 group-hover/volume:delay-100 compact-volume group-hover/volume:visible @4xl:base-volume"
            >
                <input
                    type="range"
                    min="0.0"
                    max="1.0"
                    step="0.025"
                    class="text-neutral-300 hover:brightness-130 active:brightness-130"
                    style:--slider-value=move || format!("{}%", val() * 100.0)
                    prop:value=move || val()
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
                />
            </div>
        </div>
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
enum TimeFormat {
    #[default]
    Frames,
    Timecode,
}

#[component]
fn TimecodeControl(
    time_format: RwSignal<TimeFormat>,
    #[prop(into)] frame: Signal<u32>,
    #[prop(into)] end_frame: Signal<u32>,
    #[prop(into)] fps: Signal<f64>,
) -> impl IntoView {
    view! {
        // Time display
        <div class="dropdown dropdown-top dropdown-center">
            <button tabindex="0" class="btn-player text-sm @lg:text-base font-medium">
                <span class="text-gray-300">
                    {move || timecode_str(
                        frame.get(),
                        fps.get(),
                        end_frame.get(),
                        time_format.get(),
                        true,
                    )}
                </span>
                <span class="text-gray-500">/</span>
                <span class="text-gray-500">
                    {move || {
                        let end_frame = end_frame.get();
                        timecode_str(end_frame, fps.get(), end_frame, time_format.get(), true)
                    }}
                </span>
            </button>

            <ul
                tabindex="-1"
                class="dropdown-content menu bg-base-100 rounded-box z-1 w-52 p-2 drop-shadow-xl/50"
            >
                <li class="menu-title">Time format</li>
                <li>
                    <button
                        class="flex justify-between *:pointer-events-none"
                        on:click=move |ev| {
                            time_format.set(TimeFormat::Frames);
                            event_target::<HtmlElement>(&ev).blur();
                        }
                    >
                        <span class="flex-start">Frames</span>
                        <span
                            class="size-5 flex-end {}"
                            class:hidden=move || time_format.get() != TimeFormat::Frames
                        >
                            <icon::Checkmark />
                        </span>
                    </button>

                </li>
                <li>
                    <button
                        class="flex justify-between *:pointer-events-none"
                        on:click=move |ev| {
                            time_format.set(TimeFormat::Timecode);
                            event_target::<HtmlElement>(&ev).blur();
                        }
                    >
                        <span class="flex-start">Timecode</span>
                        <span
                            class="size-5 flex-end"
                            class:hidden=move || time_format.get() != TimeFormat::Timecode
                        >
                            <icon::Checkmark />
                        </span>
                    </button>
                </li>
            </ul>
        </div>
    }
}

#[component]
fn Settings() -> impl IntoView {
    view! {
        <button
            class="btn-player size-10 p-1.25"
            on:click=move |_| {}
            on:keydown=move |ev| ev.prevent_default()
        >
            <icon::Settings />
        </button>
    }
}

#[component]
fn FullscreenToggle(is_fullscreen: RwSignal<bool>) -> impl IntoView {
    view! {
        <button
            class="btn-player size-10 p-1.5"
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

fn toggle_mute(mute: RwSignal<bool>, volume: RwSignal<f64>) {
    let m = !mute.get_untracked();
    if !m && volume.get_untracked() == 0.0 {
        volume.set(1.0);
    }
    mute.set(m);
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
