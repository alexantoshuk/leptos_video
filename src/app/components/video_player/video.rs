use super::state::{AudioState, VideoMetadata};
use crate::app::utils::*;
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
    metadata: RwSignal<VideoMetadata>,
    frame: RwSignal<u32>,
    is_playing: RwSignal<bool>,
    is_waiting: RwSignal<bool>,
    progress: RwSignal<f64>,
    playback_rate: RwSignal<f64>,
    audio_state: RwSignal<AudioState>,
) -> impl IntoView {
    let display_proxy = RwSignal::new(false);
    let meta = metadata.read_only();

    let load_metadata = move || {
        if let Some(video) = video_ref.get_untracked() {
            let d = video.duration();
            if d.is_finite() {
                metadata.write().set_duration(d);
            }
        }
    };

    let set_current_frame = move |video_ref: NodeRef<html::Video>, f: u32| {
        if let Some(video) = video_ref.get_untracked() {
            let time = meta.read_untracked().time_from_frame(f);
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

    let is_ended = move || {
        let video = video_ref.get_untracked().unwrap();
        video.ended() || frame.get_untracked() >= meta.read_untracked().end_frame
    };

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

            let fps = meta.read_untracked().fps;
            let d = video.duration();
            let timestep = 1.0 / fps;
            let t = video.current_time() + timestep;
            let t = if t > d {
                if is_loop.get_untracked() {
                    t % d
                } else {
                    t.min(d)
                }
            } else {
                t
            };
            // video.set_current_time(t);
            let f = meta.read_untracked().frame_from_time(t);
            frame.set(f);
        }
    };

    let play = move || {
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

    // on create
    Effect::new(move |_| {
        load_metadata();
        if let Some(video) = video_ref.get_untracked() {
            use_video_frame_fn(video, move |time| {
                // log!("prcise time begin");
                if is_dragging.get_untracked() || !is_playing.get_untracked() {
                    return;
                }
                let f = meta.read_untracked().frame_from_time(time);
                frame.set(f);
                // log!("prcise time: {time}, frame: {f}");
            });
        }
    });

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

    // playing
    Effect::new(move |_| {
        if is_playing.get() {
            play();
        } else {
            // pause();
            precise_pause();
        }
    });

    // loop
    Effect::new(move |_| {
        if let Some(video) = video_ref.get_untracked() {
            video.set_loop(is_loop.get());
        }
    });

    // start/end seek
    Effect::new(move |_| {
        if is_dragging.get() {
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
            let f = if is_playing.get_untracked() {
                meta.read_untracked().end_frame.min(f + 1)
            } else {
                f
            };
            set_current_frame(video_ref, f);
        }
    });

    Effect::new(move |_| {
        let f = frame.get();
        if is_dragging.get_untracked() {
            // dragging seek process
            if let Some(_) = proxy_ref.get_untracked() {
                set_current_frame(proxy_ref, f);
            } else {
                set_current_frame(video_ref, f);
            }
        } else if !is_playing.get_untracked() {
            // user set frame seek
            // pause();
            set_current_frame(video_ref, f);
        }
    });
    view! {
        <>
            <video
                // controls
                playsinline
                disablepictureinpicture
                node_ref=video_ref
                src=move || meta.read().src.clone()
                preload="metadata"
                class="absolute size-full object-fill pointer-events-none"
                class:hidden=move || display_proxy.get()

                on:loadedmetadata=move |_| load_metadata()
                on:durationchange=move |_| load_metadata()
                // on:timeupdate=move |_| {}
                // on:pause=move |_| {}
                // on:play=move |_| {}

                on:progress=move |_| { update_preload_progress() }
                on:ended=move |_| {
                    if !is_dragging.get_untracked() {
                        is_playing.set(false);
                    }
                }
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
                    is_waiting.maybe_set(false);
                    update_preload_progress()
                }
                on:playing=move |_| {
                    is_waiting.maybe_set(false);
                }
                on:seeked=move |_| {
                    display_proxy.maybe_set(false);
                    if let Some(video) = video_ref.get_untracked() {
                        if !video.paused() {
                            return;
                        }
                    }
                    if !is_dragging.get_untracked() && is_playing.get_untracked() {
                        play();
                    }
                }
            />
            <Show when=move || meta.read().proxy.is_some()>
                <video
                    playsinline
                    disablepictureinpicture
                    node_ref=proxy_ref
                    src=move || meta.read().proxy.clone().unwrap()
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
