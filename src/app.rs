use crate::components::video::Video;
use leptos::logging::log;
use leptos::prelude::*;
use leptos_meta::{MetaTags, Stylesheet, Title, provide_meta_context};
use leptos_router::{
    StaticSegment,
    components::{Route, Router, Routes},
};

pub fn shell(options: LeptosOptions) -> impl IntoView {
    view! {
        <!DOCTYPE html>
        <html lang="en">
            <head>
                <meta charset="utf-8" />
                <meta name="viewport" content="width=device-width, initial-scale=1.0" />

                <AutoReload options=options.clone() />
                <HydrationScripts options />
                <MetaTags />
            </head>
            <body>
                <App />
            </body>
        </html>
    }
}

#[component]
pub fn App() -> impl IntoView {
    // Provides context that manages stylesheets, titles, meta tags, etc.
    provide_meta_context();

    view! {
        <script>
            r#"
            if ('serviceWorker' in navigator) {
                window.addEventListener('load', () => {
                    navigator.serviceWorker.register('/service-worker.js')
                    .then(registration => {
                        console.log('SW registered: ', registration);
                    })
                    .catch(registrationError => {
                        console.log('SW registration failed: ', registrationError);
                    });
                });
            }
            "#
        </script>
        // injects a stylesheet into the document <head>
        // id=leptos means cargo-leptos will hot-reload this stylesheet
        <Stylesheet id="leptos" href="/pkg/app.css" />
        <link rel="icon" href="/favicon.svg" type="image/svg+xml" sizes="any" />
        <link rel="manifest" href="/manifest.json" />
        // sets the document title
        <Title text="Welcome  to Leptos" />

        // content for this welcome page

        <Router>
            <main>
                <Routes fallback=|| "Page not found.".into_view()>
                    <Route path=StaticSegment("") view=HomePage />
                </Routes>
            </main>
        </Router>
    }
}

/// Renders the home page of your application.
#[component]
fn HomePage() -> impl IntoView {
    // Creates a reactive value to update the button
    let count = RwSignal::new(0);
    // let on_click = move |_| *count.write() += 1;
    view! {
        // <h1>"Welcome to Leptos!"</h1>
        // <button on:click=on_click class="font-sans">
        // "Click Me: "
        // {count}
        // </button>

        // <Show when=move || {
        // let c = count.get();
        // c > 1 && c < 5
        // }>
        <div class="p-0 w-full h-dvh">
            // <Video
            // src="https://download.blender.org/peach/bigbuckbunny_movies/big_buck_bunny_1080p_h264.mov"
            // proxy="BigBuckBunny_640x360_proxy.mp4"
            // fps=24.0
            // />

            <Video
                src="Metallborne3.mp4"
                proxy="Metallborne3_proxy.mp4"
                fps=25.0
                overlay_controls=false
            />
        </div>
    }
}
