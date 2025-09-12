use leptos::either::*;
use leptos::prelude::*;

#[component]
pub fn Play() -> impl IntoView {
    view! {
        <svg
            class="group-hover:text-emphasis group-hover:dark:text-emphasis-dark transition-colors delay-75 duration-200 ease-in-out"
            width="22"
            height="22"
            viewBox="0 0 24 24"
            stroke-linecap="round"
            stroke-linejoin="round"
            stroke-width="2"
            stroke="currentColor"
            fill="none"
            role="graphics-symbol"
        >
            <path stroke="none" d="M0 0h24v24H0z" fill="none"></path>
            <path
                d="M6 4v16a1 1 0 0 0 1.524 .852l13 -8a1 1 0 0 0 0 -1.704l-13 -8a1 1 0 0 0 -1.524 .852z"
                stroke-width="0"
                fill="currentColor"
            ></path>
        </svg>
    }
}

#[component]
pub fn Pause() -> impl IntoView {
    view! {
        <svg
            class="group-hover:text-emphasis group-hover:dark:text-emphasis-dark transition-colors delay-75 duration-200 ease-in-out"
            width="22"
            height="22"
            viewBox="0 0 24 24"
            stroke-linecap="round"
            stroke-linejoin="round"
            stroke-width="2"
            stroke="currentColor"
            fill="none"
            role="graphics-symbol"
        >
            <path stroke="none" d="M0 0h24v24H0z" fill="none"></path>
            <path
                d="M9 4h-2a2 2 0 0 0 -2 2v12a2 2 0 0 0 2 2h2a2 2 0 0 0 2 -2v-12a2 2 0 0 0 -2 -2z"
                stroke-width="0"
                fill="currentColor"
            ></path>
            <path
                d="M17 4h-2a2 2 0 0 0 -2 2v12a2 2 0 0 0 2 2h2a2 2 0 0 0 2 -2v-12a2 2 0 0 0 -2 -2z"
                stroke-width="0"
                fill="currentColor"
            ></path>
        </svg>
    }
}

#[component]
pub fn PlayPause(#[prop(into)] play: Signal<bool>) -> impl IntoView {
    move || {
        if play.get() {
            Either::Left(Pause())
        } else {
            Either::Right(Play())
        }
    }
}

#[component]
pub fn Volume0() -> impl IntoView {
    view! {
        <svg
            class="group-hover:text-emphasis group-hover:dark:text-emphasis-dark transition-colors delay-75 duration-200 ease-in-out"
            width="24"
            height="24"
            viewBox="0 0 24 24"
            stroke-linecap="round"
            stroke-linejoin="round"
            stroke-width="1.5"
            stroke="currentColor"
            fill="none"
            role="graphics-symbol"
        >
            <polygon points="11 5 6 9 2 9 2 15 6 15 11 19 11 5"></polygon>
            <line x1="23" y1="9" x2="17" y2="15"></line>
            <line x1="17" y1="9" x2="23" y2="15"></line>
        </svg>
    }
}

#[component]
pub fn Volume1() -> impl IntoView {
    view! {
        <svg
            class="group-hover:text-emphasis group-hover:dark:text-emphasis-dark transition-colors delay-75 duration-200 ease-in-out"
            width="24"
            height="24"
            viewBox="0 0 24 24"
            stroke-linecap="round"
            stroke-linejoin="round"
            stroke-width="1.5"
            stroke="currentColor"
            fill="none"
            role="graphics-symbol"
        >
            <polygon points="11 5 6 9 2 9 2 15 6 15 11 19 11 5"></polygon>
            <path d="M15.54 8.46a5 5 0 0 1 0 7.07"></path>
        </svg>
    }
}

#[component]
pub fn Volume2() -> impl IntoView {
    view! {
        <svg
            class="group-hover:text-emphasis group-hover:dark:text-emphasis-dark transition-colors delay-75 duration-200 ease-in-out"
            width="24"
            height="24"
            viewBox="0 0 24 24"
            stroke-linecap="round"
            stroke-linejoin="round"
            stroke-width="1.5"
            stroke="currentColor"
            fill="none"
            role="graphics-symbol"
        >
            <polygon points="11 5 6 9 2 9 2 15 6 15 11 19 11 5"></polygon>
            <path d="M19.07 4.93a10 10 0 0 1 0 14.14M15.54 8.46a5 5 0 0 1 0 7.07"></path>
        </svg>
    }
}

#[component]
pub fn Volume(#[prop(into)] volume: Signal<f64>) -> impl IntoView {
    move || {
        let vol = volume.get();
        if vol <= 0.0 {
            EitherOf3::A(Volume0())
        } else if vol < 0.5 {
            EitherOf3::B(Volume1())
        } else {
            EitherOf3::C(Volume2())
        }
    }
}

#[component]
pub fn FullScreenEnter() -> impl IntoView {
    view! {
        <svg
            class="group-hover:text-emphasis group-hover:dark:text-emphasis-dark transition-colors delay-75 duration-200 ease-in-out"
            width="18"
            height="18"
            viewBox="0 0 24 24"
            stroke-linecap="round"
            stroke-linejoin="round"
            stroke-width="2"
            stroke="currentColor"
            fill="none"
            role="graphics-symbol"
        >
            <polyline points="4 14 10 14 10 20"></polyline>
            <polyline points="20 10 14 10 14 4"></polyline>
            <line x1="14" y1="10" x2="21" y2="3"></line>
            <line x1="3" y1="21" x2="10" y2="14"></line>
        </svg>
    }
}

#[component]
pub fn FullScreenExit() -> impl IntoView {
    view! {
        <svg
            class="group-hover:text-emphasis group-hover:dark:text-emphasis-dark transition-colors delay-75 duration-200 ease-in-out"
            width="18"
            height="18"
            viewBox="0 0 24 24"
            stroke-linecap="round"
            stroke-linejoin="round"
            stroke-width="2"
            stroke="currentColor"
            fill="none"
            role="graphics-symbol"
        >
            <polyline points="15 3 21 3 21 9"></polyline>
            <polyline points="9 21 3 21 3 15"></polyline>
            <line x1="21" y1="3" x2="14" y2="10"></line>
            <line x1="3" y1="21" x2="10" y2="14"></line>
        </svg>
    }
}

#[component]
pub fn Fullscreen(#[prop(into)] fullscreen: Signal<bool>) -> impl IntoView {
    move || {
        if fullscreen.get() {
            Either::Left(FullScreenEnter())
        } else {
            Either::Right(FullScreenExit())
        }
    }
}
