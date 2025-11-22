mod components;
mod pages;
mod timecode;
mod utils;
use pages::VideoDetail;

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

                // <script>
                //     r#"
                //     if ('serviceWorker' in navigator) {
                //         window.addEventListener('load', () => {
                //             navigator.serviceWorker.register('/service-worker.js')
                //             .then(registration => {
                //                 console.log('SW registered: ', registration);
                //             })
                //             .catch(registrationError => {
                //                 console.log('SW registration failed: ', registrationError);
                //             });
                //         });
                //     }
                //     "#
                // </script>

                <Stylesheet id="leptos" href="/pkg/app.css" />
                <link rel="icon" href="/favicon.svg" type="image/svg+xml" sizes="any" />
                <link rel="manifest" href="/manifest.json" />
                <MetaTags />
            </head>
            <body>
            </body>
        </html>
    }
}

#[component]
pub fn App() -> impl IntoView {
    // Provides context that manages stylesheets, titles, meta tags, etc.
    provide_meta_context();

    view! {
        // sets the document title
        <Title text="Welcome  to Leptos" />

        // content for this welcome page

        <Router>
            <main>
                <Routes fallback=|| "Page not found.".into_view()>
                    <Route path=StaticSegment("") view=VideoDetail />
                </Routes>
            </main>
        </Router>
    }
}
