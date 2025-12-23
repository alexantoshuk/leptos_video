use super::state::{AudioState, PlayingState, VideoInfo};
use crate::utils::*;
use leptos::html;
use leptos::logging::log;
use leptos::prelude::*;
use web_sys::HtmlMediaElement;
use web_time::Duration;

#[component]
pub fn Video(
    video_ref: NodeRef<html::Video>,
    proxy_ref: NodeRef<html::Video>,
    #[prop(into)] is_dragging: Signal<bool>,
    #[prop(into)] is_loop: Signal<bool>,
    videoinfo: RwSignal<VideoInfo>,
    frame: RwSignal<u32>,
    playing: RwSignal<PlayingState>,
    is_waiting: RwSignal<bool>,
    progress: RwSignal<f64>,
    playback_rate: RwSignal<f64>,
    audio_state: RwSignal<AudioState>,
) -> impl IntoView {
    let display_proxy = RwSignal::new(false);
    let videoinfo_ = videoinfo.read_only();

    let load_metadata = move || {
        if let Some(video) = video_ref.get_untracked() {
            let d = video.duration();
            if d.is_finite() {
                videoinfo.write().set_duration(d);
            }
        }
    };

    let set_current_frame = move |video_ref: NodeRef<html::Video>, f: u32| {
        if let Some(video) = video_ref.get_untracked() {
            let time = videoinfo_.read_untracked().time_from_frame(f);
            video.set_current_time(time);
        }
    };

    let update_preload_progress = move || {
        if let Some(video) = video_ref.get_untracked() {
            let vb = video.buffered();
            let time = video.current_time();
            let d = video.duration();

            for i in (0..vb.length()).rev() {
                let start = vb.start(i).unwrap();
                let end = vb.end(i).unwrap();
                if time >= start && time <= end {
                    progress.set(end / d);
                    break;
                }
            }
        }
    };

    let is_ended = move || frame.get_untracked() >= videoinfo_.read_untracked().end_frame;

    let pause = move || {
        log!("press_pause");
        if let Some(video) = video_ref.get_untracked() {
            video.pause();
        }
    };

    let precise_pause = move || {
        log!("precise_pause");
        if let Some(video) = video_ref.get_untracked() {
            if video.paused() {
                return;
            }
            video.pause();
            let end_frame = videoinfo.read_untracked().end_frame;
            frame.maybe_update(|f| {
                if *f >= end_frame {
                    false
                } else {
                    *f += 1;
                    true
                }
            });
        }
    };

    let play = move || {
        log!("play");
        if let Some(video) = video_ref.get_untracked() {
            if !video.paused() {
                return;
            }
            if !is_loop.get_untracked() && is_ended() {
                video.set_current_time(0.0);
            }
            video.play();
        }
    };

    // audio state change
    Effect::new(move |_| {
        if let Some(video) = video_ref.get_untracked() {
            let audio_state = audio_state.get();
            video.set_volume(audio_state.volume);
            video.set_muted(audio_state.is_muted);
        }
    });

    // playback rate
    Effect::new(move |_| {
        if let Some(video) = video_ref.get_untracked() {
            video.set_playback_rate(playback_rate.get());
        }
    });

    // loop
    Effect::new(move |_| {
        if let Some(video) = video_ref.get_untracked() {
            video.set_loop(is_loop.get());
        }
    });

    // playing
    Effect::watch(
        move || playing.get(),
        move |playing, _, _| {
            use PlayingState::*;
            match playing {
                Play => {
                    play();
                }
                PrecisePause => {
                    precise_pause();
                }
                _ => {
                    pause();
                }
            }
        },
        false,
    );

    // start/end seek
    Effect::watch(
        move || is_dragging.get(),
        move |&is_dragging, _, _| {
            if is_dragging {
                // start dragging seek
                pause();
                let f = frame.get_untracked();
                // container_ref.get_untracked().unwrap().focus();
                if let Some(_) = proxy_ref.get_untracked() {
                    set_current_frame(proxy_ref, f);
                } else {
                    set_current_frame(video_ref, f);
                }
            } else {
                // reset_overlay_controls_timeout();
                // end dragging seek
                let f = frame.get_untracked();
                let f = if playing.get_untracked() == PlayingState::Play {
                    videoinfo_.read_untracked().end_frame.min(f + 1)
                } else {
                    f
                };
                set_current_frame(video_ref, f);
            }
        },
        false,
    );

    Effect::watch(
        move || frame.get(),
        move |&frame, _, _| {
            if is_dragging.get_untracked() {
                // dragging seek process
                if let Some(_) = proxy_ref.get_untracked() {
                    set_current_frame(proxy_ref, frame);
                } else {
                    set_current_frame(video_ref, frame);
                }
            } else if playing.get_untracked() != PlayingState::Play {
                // user set frame seek
                // pause();
                set_current_frame(video_ref, frame);
            }
        },
        false,
    );

    // on create
    Effect::new(move |_| {
        load_metadata();
        if let Some(video) = video_ref.get_untracked() {
            use_video_frame_fn(video, move |time| {
                // log!("prcise time begin");
                if is_dragging.get_untracked() || playing.get_untracked() != PlayingState::Play {
                    return;
                }
                let f = videoinfo_.read_untracked().frame_from_time(time);

                if !is_loop.get_untracked() && f >= videoinfo_.read_untracked().end_frame {
                    playing.set(PlayingState::EndPause);
                }

                frame.set(f);
                // log!("prcise time: {time}, frame: {f}");
            });
        }
    });

    view! {
        <>
            <video
                // controls
                playsinline
                disablepictureinpicture
                node_ref=video_ref
                poster=move || videoinfo_.read().poster.clone()
                src=move || videoinfo_.read().src.clone()
                preload="metadata"
                class="absolute size-full object-fill pointer-events-none"
                class:hidden=move || display_proxy.get()
                on:loadedmetadata=move |_| load_metadata()
                on:durationchange=move |_| load_metadata()
                // on:timeupdate=move |_| {}
                // on:pause=move |_| {}
                // on:play=move |_| {}

                on:progress=move |_| { update_preload_progress() }
                // on:ended=move |_| {
                // if !is_dragging.get_untracked() {
                // playing.maybe_set(PlayingState::EndPause);
                // }
                // }
                on:waiting=move |_| {
                    set_timeout(
                        move || {
                            if let Some(video) = video_ref.get_untracked()
                                && video.ready_state() <= HtmlMediaElement::HAVE_CURRENT_DATA
                            {
                                log!("Waiting video...");
                                is_waiting.set(true);
                            }
                        },
                        Duration::from_secs(1),
                    );
                }
                on:stalled=move |_| {
                    is_waiting.set(true);
                }
                // on:suspend=move |_| log!("suspend")
                on:canplay=move |_| {
                    is_waiting.set(false);
                    update_preload_progress()
                }
                on:playing=move |_| {
                    is_waiting.set(false);
                }
                on:seeked=move |_| {
                    display_proxy.maybe_set(false);
                    if let Some(video) = video_ref.get_untracked() {
                        if !video.paused() {
                            return;
                        }
                    }
                    if !is_dragging.get_untracked() && playing.get_untracked() == PlayingState::Play
                    {
                        play();
                    }
                }
            />
            <Show when=move || !videoinfo_.read().proxy.is_empty()>
                <video
                    playsinline
                    disablepictureinpicture
                    node_ref=proxy_ref
                    src=move || videoinfo_.read().proxy.clone()
                    preload="auto"
                    class="absolute size-full object-fill pointer-events-none"
                    class:hidden=move || !display_proxy.get()
                    on:seeked=move |_| {
                        if is_dragging.get_untracked() {
                            display_proxy.maybe_set(true);
                        }
                    }
                />
            </Show>
        </>
    }
}
