use super::icon;
use super::state::{AudioState, PlayingState, TimeFormat, VideoInfo};
use crate::utils::*;
use leptos::either::*;
use leptos::html;
use leptos::logging::log;
use leptos::prelude::*;
use leptos_use::core::Position;
use leptos_use::{UseDraggableOptions, use_draggable_with_options, use_throttle_fn_with_arg};
use web_sys::{CanvasRenderingContext2d, HtmlElement, HtmlInputElement};

const THUMB_MAX_SIZE: i32 = 200;

#[component]
pub fn Controls(
    proxy_ref: NodeRef<html::Video>,
    #[prop(into)] videoinfo: Signal<VideoInfo>,
    #[prop(into)] progress: Signal<f64>,
    #[prop(into)] overlay: Signal<bool>,
    frame: RwSignal<u32>,
    playback_rate: RwSignal<f64>,
    is_dragging: RwSignal<bool>,
    playing: RwSignal<PlayingState>,
    is_loop: RwSignal<bool>,
    audio_state: RwSignal<AudioState>,
    is_fullscreen: RwSignal<bool>,
    time_format: RwSignal<TimeFormat>,
) -> impl IntoView {
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

    view! {
        <div class="flex-col space-y-1">
            <ProgressBar proxy_ref videoinfo frame overlay is_dragging progress time_format />

            // <div class="flex items-center justify-between h-4"></div>
            // Control buttons
            <div class="flex items-center justify-between">
                // Left side
                <div class="shrink-0 flex items-center">
                    <PlayPauseToggle playing />
                    <PlybackRateControl playback_rate />
                    // <PrevNextFrameButtonsGrp on_prev_click=prev_frame on_next_click=next_frame />
                    <LoopToggle is_loop />
                    <VolumeControl audio_state />
                </div>

                // Center
                <div class="shrink flex items-center min-w-0">
                    <TimecodeControl videoinfo frame time_format />
                </div>

                // Right side
                <div class="shrink-0 flex items-center space-x-0">
                    <span class="mr-2 text-base font-medium text-gray-400 text-nowrap drop-shadow-ico hidden @3xl:block">
                        {move || format!("{}fps", videoinfo.read().fps)}
                    </span>
                    <Settings />
                    <FullscreenToggle is_fullscreen />
                </div>
            </div>
        </div>
    }
}

#[component]
fn ProgressBar(
    proxy_ref: NodeRef<html::Video>,
    #[prop(into)] videoinfo: Signal<VideoInfo>,
    #[prop(into)] progress: Signal<f64>,
    #[prop(into)] overlay: Signal<bool>,
    #[prop(into)] time_format: Signal<TimeFormat>,
    frame: RwSignal<u32>,
    is_dragging: RwSignal<bool>,
) -> impl IntoView {
    #[derive(Debug, Clone, PartialEq, Eq)]
    enum Hover {
        Enter(i32),
        Move(i32),
        Exit(i32),
    }
    let node_ref = NodeRef::<html::Div>::new();
    let thumb_canvas_ref = NodeRef::<html::Canvas>::new();
    let hover = RwSignal::new(Hover::Exit(0));

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

    #[cfg(not(feature = "ssr"))]
    Effect::new(move |_| {
        use wasm_bindgen::{JsCast, JsValue};
        if let Some(proxy_video) = proxy_ref.get_untracked()
            && let Some(canvas) = thumb_canvas_ref.get_untracked()
        {
            let thumb_frame = thumb_frame.get();
            let time = videoinfo.read_untracked().time_from_frame(thumb_frame);
            proxy_video.set_current_time(time);

            if let Ok(Some(ctx)) = canvas.get_context("2d") {
                let ctx: CanvasRenderingContext2d = ctx.unchecked_into();
                ctx.draw_image_with_html_video_element_and_dw_and_dh(
                    &proxy_video,
                    0.0,
                    0.0,
                    canvas.width() as f64,
                    canvas.height() as f64,
                );
            }
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
                    let f = videoinfo.read_untracked().frame_from_pos(pos);
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
                    let f = videoinfo.read_untracked().frame_from_pos(pos);
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
                    let f = videoinfo.read_untracked().frame_from_pos(pos);
                    frame.set(f);
                    // on_dragging_end(f);
                }
            })
            .stop_propagation(true)
            .prevent_default(true),
    );

    view! {
        <div class="relative h-1">
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
                        let f = videoinfo.read_untracked().frame_from_pos(pos);
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
                <div class="absolute size-full bg-neutral-600 pointer-events-none" />
                // class=("bg-white/25", move || overlay.get())
                // Preload progress
                <div
                    class="absolute origin-left size-full bg-white/20 transition-scale duration-200 pointer-events-none"
                    style:scale=move || { format!("{} 1", progress.get()) }
                />

                // Progress
                <div
                    class="absolute origin-left size-full bg-primary pointer-events-none"
                    style:scale=move || {
                        let p = videoinfo.read().progress(frame.get());
                        format!("{} 1", p)
                    }
                />

                // Cursor
                <div
                    class="absolute origin-left size-full pointer-events-none"
                    style:translate=move || {
                        let end_frame = videoinfo.read().end_frame;
                        format!("{}% 0", (100 * frame.get()) as f64 / (end_frame + 1) as f64)
                    }
                >
                    <div
                        class="h-full origin-left bg-white pointer-events-none"
                        style:width=move || {
                            let end_frame = videoinfo.read().end_frame;
                            if end_frame == 0 {
                                "0".into()
                            } else {
                                format!("calc(max({}%,3px))", 100.0 / (end_frame + 1) as f64)
                            }
                        }
                    />
                </div>

            </div>
            <div
                class="absolute transition-opacity duration-200 delay-100 flex flex-col items-center gap-2 pointer-events-none"
                style=move || {
                    if let Some(p) = node_ref.get_untracked() {
                        let p_width = p.client_width();
                        let aspect_ratio = videoinfo.read().aspect_ratio;
                        let (w, h) = thumbnail_size(aspect_ratio);
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
                        format!("translate:{x}px {y}px;opacity:{o};")
                    } else {
                        "opacity:0".into()
                    }
                }
            >
                <div class="rounded-sm outline-solid outline-2 outline-neutral-300 drop-shadow-xl/50
                overflow-hidden ">
                    {move || {
                        let aspect_ratio = videoinfo.read().aspect_ratio;
                        let (w, h) = thumbnail_size(aspect_ratio);
                        view! { <canvas node_ref=thumb_canvas_ref width=w height=h></canvas> }
                    }}

                </div>
                <div class="text-base font-bold drop-shadow-ico text-neutral-300 text-center">
                    {move || { videoinfo.read().time_string(thumb_frame.get(), time_format.get()) }}
                </div>
            </div>
        </div>
    }
}

#[component]
fn PlayPauseToggle(playing: RwSignal<PlayingState>) -> impl IntoView {
    view! {
        <button
            class="btn-player size-10 p-1"
            type="button"
            on:click=move |_| playing.write().toggle_play()
            on:keydown=move |ev| ev.prevent_default()
        >
            {move || {
                if let PlayingState::Play = playing.get() {
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
        <div class="*:btn-player *:w-8 *:h-10 hidden @3xl:flex items-center space-x-1">
            <button
                class="p-0 pl-2"
                on:click=move |_| on_prev_click()
                on:keydown=move |ev| ev.prevent_default()
            >
                <icon::NextFrame class="-scale-x-100" />
            </button>
            <button
                class="p-0 pr-2"
                on:click=move |_| on_next_click()
                on:keydown=move |ev| ev.prevent_default()
            >
                <icon::NextFrame />
            </button>
        </div>
    }
}

#[component]
fn PlybackRateControl(playback_rate: RwSignal<f64>) -> impl IntoView {
    let step = 0.25;
    let start = 1;
    let end = 7;
    let ticks_num = end - start - 1;
    let start_speed = start as f64 * step;
    let end_speed = end as f64 * step;

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
                        {(start..=end)
                            .into_iter()
                            .map(|i| {
                                let ratio = i as f64 * step;
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
                        min=start_speed.to_string()
                        max=end_speed.to_string()
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
                        {(0..ticks_num).into_iter().map(|_| view! { <span></span> }).collect_view()}
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
fn VolumeControl(audio_state: RwSignal<AudioState>) -> impl IntoView {
    let volume = move || audio_state.read().volume();

    view! {
        <div class="relative group/volume h-10 mobile:hidden">
            <button
                class="btn-player size-10 p-2"
                on:click=move |_| audio_state.write().toggle_mute()
                on:keydown=move |ev| ev.prevent_default()
            >
                {move || {
                    let vol = volume();
                    if vol <= 0.0 {
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
                    style:--slider-value=move || format!("{}%", volume() * 100.0)
                    prop:value=volume
                    on:input=move |ev| {
                        ev.stop_propagation();
                        let target = event_target::<HtmlInputElement>(&ev);
                        let vol = target.value_as_number();
                        audio_state.write().set_volume(vol);
                    }
                    on:keydown=move |ev| ev.prevent_default()
                />
            </div>
        </div>
    }
}

#[component]
fn TimecodeControl(
    time_format: RwSignal<TimeFormat>,
    #[prop(into)] frame: Signal<u32>,
    #[prop(into)] videoinfo: Signal<VideoInfo>,
) -> impl IntoView {
    view! {
        // Time display
        <div class="dropdown dropdown-top dropdown-center">
            <button tabindex="0" class="btn-player text-sm @lg:text-base font-medium">
                <span class="text-gray-300">
                    {move || videoinfo.read().time_string(frame.get(), time_format.get())}
                </span>
                <span class="text-gray-400">/</span>
                <span class="text-gray-400">
                    {move || videoinfo.read().end_time_string(time_format.get())}
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
