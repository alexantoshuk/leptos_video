use leptos::logging::log;
use leptos::prelude::*;
use leptos_use::core::IntoElementMaybeSignal;
use leptos_use::utils::Pausable;
use web_sys::{Element, HtmlVideoElement, MouseEvent};
use web_time::{Duration, Instant};

pub trait WriteSignalEx<T: PartialEq>: Update<Value = T> {
    fn maybe_set(&self, value: T) {
        self.maybe_update(|old| {
            if *old != value {
                *old = value;
                true
            } else {
                false
            }
        });
    }
}

impl<T> WriteSignalEx<T> for RwSignal<T> where T: Send + Sync + PartialEq + 'static {}
impl<T> WriteSignalEx<T> for WriteSignal<T> where T: Send + Sync + PartialEq + 'static {}

pub fn onclick_handler(
    on_click: impl FnOnce(MouseEvent) + 'static + Copy,
    on_dblclick: impl FnOnce(MouseEvent) + 'static + Copy,
) -> impl FnMut(MouseEvent) + 'static + Copy {
    const DURATION: Duration = Duration::from_millis(200);
    let click_timeout_handle = StoredValue::new(None::<TimeoutHandle>);
    let last_click_time = StoredValue::new(None::<Instant>);
    move |ev| {
        let now = Instant::now();
        let prev = last_click_time.get_value();
        if let Some(prev) = prev
            && (now - prev) < DURATION
        {
            on_dblclick(ev);
            // Clear any pending single-click timeout
            if let Some(handle) = click_timeout_handle.get_value() {
                handle.clear();
                click_timeout_handle.set_value(None);
            }
            last_click_time.set_value(None); // Reset for next potential single click
            return;
        }
        last_click_time.set_value(Some(now));

        click_timeout_handle.set_value(
            set_timeout_with_handle(
                move || {
                    if last_click_time.get_value().is_some() {
                        on_click(ev);
                        last_click_time.set_value(None);
                    }
                },
                DURATION,
            )
            .ok(),
        )
    }
}

pub fn is_element_fullscreen<El, M: ?Sized>(target: El) -> bool
where
    El: IntoElementMaybeSignal<Element, M>,
{
    if let Some(target) = target.into_element_maybe_signal().get_untracked()
        && let Some(el) = document().fullscreen_element()
    {
        el == target
    } else {
        false
    }
}

pub fn request_fullscreen<El, M: ?Sized>(target: El)
where
    El: IntoElementMaybeSignal<Element, M>,
{
    if let Some(target) = target.into_element_maybe_signal().get_untracked() {
        if let Some(el) = document().fullscreen_element()
            && el == target
        {
            return;
        }
        let _ = target.request_fullscreen();
    }
}

pub fn use_video_frame_fn(
    video: HtmlVideoElement,
    callback: impl Fn(f64) + 'static,
) -> Pausable<impl Fn() + Clone + Send + Sync, impl Fn() + Clone + Send + Sync> {
    #[cfg(feature = "ssr")]
    {
        let (is_active, _) = signal(false);

        Pausable {
            resume: || {},
            pause: || {},
            is_active: is_active.into(),
        }
    }
    #[cfg(not(feature = "ssr"))]
    {
        use js_sys::{Function, Reflect};
        use leptos_use::{js, sendwrap_fn};
        use send_wrapper::SendWrapper;
        use std::cell::{Cell, RefCell};
        use std::rc::Rc;
        use wasm_bindgen::closure::Closure;
        use wasm_bindgen::{JsCast, JsValue};

        let (is_active, set_active) = signal(false);
        let video_clone = video.clone();
        let rvf_handle = Rc::new(Cell::new(None::<f64>));

        let loop_ref = Rc::new(RefCell::new(
            Box::new(|_: f64, _: JsValue| {}) as Box<dyn Fn(f64, JsValue)>
        ));

        let cancel_video_frame_callback =
            Reflect::get(&video_clone, &JsValue::from_str("cancelVideoFrameCallback"))
                .expect("no cancelVideoFrameCallback");
        let cancel_video_frame_callback: Function = cancel_video_frame_callback
            .dyn_into()
            .expect("not a function");

        let request_next_frame = {
            let loop_ref = Rc::clone(&loop_ref);
            let rvf_handle = Rc::clone(&rvf_handle);

            let request_video_frame_callback = Reflect::get(
                &video_clone,
                &JsValue::from_str("requestVideoFrameCallback"),
            )
            .expect("no requestVideoFrameCallback");
            let request_video_frame_callback: Function = request_video_frame_callback
                .dyn_into()
                .expect("not a function");

            move || {
                let loop_ref = Rc::clone(&loop_ref);

                rvf_handle.set(
                    request_video_frame_callback
                        .call1(
                            &video_clone,
                            Closure::once_into_js(move |now: f64, metadata: JsValue| {
                                loop_ref.borrow()(now, metadata);
                            })
                            .as_ref()
                            .unchecked_ref(),
                        )
                        .and_then(|jsv| jsv.try_into())
                        .ok(),
                );
            }
        };

        let loop_fn = {
            #[allow(clippy::clone_on_copy)]
            let request_next_frame = request_next_frame.clone();
            let video_clone = video.clone();
            move |_: f64, metadata: JsValue| {
                if !is_active.try_get_untracked().unwrap_or_default() {
                    return;
                }
                // log!("{metadata:?}");
                if let Ok(time) = js!(metadata["mediaTime"]) {
                    let duration = video_clone.duration();
                    let time = time.as_f64().unwrap();
                    let time = if time == duration {
                        time
                    } else {
                        time % duration
                    };
                    callback(time);
                }

                request_next_frame();
            }
        };

        let _ = loop_ref.replace(Box::new(loop_fn));

        let resume = sendwrap_fn!(move || {
            if !is_active.get_untracked() {
                set_active.set(true);
                request_next_frame();
            }
        });

        let video_clone = video.clone();
        let pause = sendwrap_fn!(move || {
            set_active.set(false);

            let handle = rvf_handle.get();
            if let Some(handle) = handle {
                let _ = cancel_video_frame_callback.call1(&video_clone, &JsValue::from(handle));
            }
            rvf_handle.set(None);
        });

        resume();

        on_cleanup({
            let pause = pause.clone();
            #[allow(clippy::redundant_closure)]
            move || pause()
        });

        Pausable {
            resume,
            pause,
            is_active: is_active.into(),
        }
    }
}
