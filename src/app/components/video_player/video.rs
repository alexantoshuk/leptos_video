use super::{frame_from_time, time_from_frame};
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
    #[prop(into)] src: Signal<String>,
    #[prop(into, optional)] proxy: Signal<String>,
    #[prop(into)] fps: Signal<f64>,
    #[prop(into)] is_dragging: Signal<bool>,
    #[prop(into)] is_loop: Signal<bool>,
    frame: RwSignal<u32>,
    end_frame: RwSignal<u32>,
    is_playing: RwSignal<bool>,
    is_waiting: RwSignal<bool>,
    progress: RwSignal<f64>,
    playback_rate: RwSignal<f64>,
    volume: RwSignal<f64>,
    is_mute: RwSignal<bool>,
) -> impl IntoView {
    let display_proxy = RwSignal::new(false);

    let load_metadata = move || {
        if let Some(video) = video_ref.get_untracked() {
            let d = video.duration();
            if d.is_finite() {
                let total_frames = (d * fps.get()).next_up() as u32;
                end_frame.set((total_frames - 1).max(0));
            }
        }
    };

    let set_current_frame = move |video_ref: NodeRef<html::Video>, f: u32| {
        if let Some(video) = video_ref.get_untracked() {
            let fps = fps.get_untracked();
            let time = time_from_frame(f, fps);
            video.set_current_time(time);
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
                    progress.set(p);
                    break;
                }
            }
        }
    };

    let is_ended = move || {
        let video = video_ref.get_untracked().unwrap();
        video.ended() || frame.get_untracked() >= end_frame.get_untracked()
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
            video.pause();

            let fps = fps.get_untracked();
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
            video.set_current_time(t);
        }
    };

    let play = move || {
        if let Some(video) = video_ref.get_untracked() {
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
                let fps = fps.get_untracked();
                let end_frame = end_frame.get_untracked();
                let f = frame_from_time(time, fps).min(end_frame);
                frame.set(f);

                // log!("prcise time: {time}, frame: {f}");
            });
        }
    });

    // is_mute
    Effect::new(move |_| {
        if let Some(video) = video_ref.get_untracked() {
            video.set_muted(is_mute.get());
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
                (f + 1).min(end_frame.get_untracked())
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
                src=src
                preload="metadata"
                class="absolute size-full object-fill pointer-events-none"
                // style=move || { if display_proxy.get() { "display:none;" } else { "" } }

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
                    log!("Waiting video...");
                    set_timeout(
                        move || {
                            if let Some(video) = video_ref.get_untracked()
                                && video.ready_state() <= HtmlMediaElement::HAVE_CURRENT_DATA
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
                    is_waiting.set_changed(false);
                    update_preload_progress()
                }
                on:playing=move |_| {
                    is_waiting.set_changed(false);
                }
                on:seeked=move |_| {
                    display_proxy.set_changed(false);
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
            <Show when=move || proxy.get() != "">
                <video
                    playsinline
                    disablepictureinpicture
                    node_ref=proxy_ref
                    src=proxy
                    preload="auto"
                    class="absolute size-full object-fill pointer-events-none"
                    class:opacity-0=move || !display_proxy.get()
                    on:seeked=move |_| {
                        if is_dragging.get_untracked() {
                            display_proxy.set_changed(true);
                        }
                    }
                />
            </Show>
        </>
    }
}

// fn frame_from_time(time: f64, fps: f64) -> u32 {
//     (time * fps + 0.01) as u32
// }
