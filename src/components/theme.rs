use leptos::logging::log;
use leptos::prelude::*;
use serde::{Deserialize, Serialize};
use serde_json;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ColorScheme {
    Light,
    Dark,
    System,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ThemeName {
    Blue,
    Red,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Theme {
    pub name: ThemeName,
    pub scheme: ColorScheme,
    pub colors: ThemeColors,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThemeColors {
    // pub primary: String,
    // pub secondary: String,
    pub accent: String,
    pub background: String,
    // pub surface: String,
    // pub text: String,
}

impl Theme {
    pub fn light(theme: ThemeName) -> Self {
        let colors = match theme {
            ThemeName::Blue => ThemeColors {
                // primary: "#3b82f6".to_string(),   // blue-500
                // secondary: "#64748b".to_string(), // slate-500
                accent: "#06b6d4".to_string(),
                // cyan-500
                background: "#ffffff".to_string(),
                // surface: "#f8fafc".to_string(), // slate-50
                // text: "#0f172a".to_string(),    // slate-900
            },
            ThemeName::Red => ThemeColors {
                // primary: "#3b82f6".to_string(),   // blue-500
                // secondary: "#64748b".to_string(), // slate-500
                accent: "#ff0000".to_string(),
                // cyan-500
                background: "#ffffff".to_string(),
                // surface: "#f8fafc".to_string(), // slate-50
                // text: "#0f172a".to_string(),    // slate-900
            },
        };

        Self {
            name: theme,
            scheme: ColorScheme::Light,
            colors,
        }
    }

    pub fn dark(theme: ThemeName) -> Self {
        let colors = match theme {
            ThemeName::Blue => ThemeColors {
                // primary: "#3b82f6".to_string(),   // blue-500
                // secondary: "#64748b".to_string(), // slate-500
                accent: "#06b6d4".to_string(),
                // cyan-500
                background: "#0f172a".to_string(),
                // surface: "#f8fafc".to_string(), // slate-50
                // text: "#0f172a".to_string(),    // slate-900
            },
            ThemeName::Red => ThemeColors {
                // primary: "#3b82f6".to_string(),   // blue-500
                // secondary: "#64748b".to_string(), // slate-500
                accent: "#ff0000".to_string(),
                // cyan-500
                background: "#0f172a".to_string(),
                // surface: "#f8fafc".to_string(), // slate-50
                // text: "#0f172a".to_string(),    // slate-900
            },
        };

        Self {
            name: theme,
            scheme: ColorScheme::Dark,
            colors,
        }
    }

    pub fn update_name(self, name: ThemeName) -> Self {
        let new_theme = match self.scheme {
            ColorScheme::Light => Theme::light(name),
            ColorScheme::Dark => Theme::dark(name),
            ColorScheme::System => {
                // Detect system preference
                if is_dark_mode_preferred() {
                    Theme::dark(name)
                } else {
                    Theme::light(name)
                }
            }
        };
        new_theme
    }

    pub fn update_scheme(self, scheme: ColorScheme) -> Self {
        let new_theme = match scheme {
            ColorScheme::Light => Theme::light(self.name),
            ColorScheme::Dark => Theme::dark(self.name),
            ColorScheme::System => {
                if is_dark_mode_preferred() {
                    Theme::dark(self.name)
                } else {
                    Theme::light(self.name)
                }
            }
        };
        new_theme
    }
}

#[component]
pub fn ThemeProvider(children: Children) -> impl IntoView {
    let theme = RwSignal::new(Theme::dark(ThemeName::Red));

    // Provide theme to context
    provide_context(theme);

    // Apply theme to CSS custom properties
    Effect::new(move |_| {
        let theme = theme.get();
        apply_theme_css(&theme);
        apply_dark_mode_class(theme.scheme);
    });

    // Persist theme to localStorage
    Effect::new(move |_| {
        let theme = theme.get();
        if let Ok(Some(storage)) = window().local_storage() {
            let _ = storage.set_item("theme", &serde_json::to_string(&theme).unwrap_or_default());
        }
    });

    children()
}

fn apply_theme_css(theme: &Theme) {
    let style = format!(
        "
        --color-main-accent:{}; \
        --color-background:{}; \
        ",
        // theme.colors.primary,
        // theme.colors.secondary,
        theme.colors.accent,
        theme.colors.background,
        // theme.colors.surface,
        // theme.colors.text,
    );
    let document = document();

    // Get the root element (html)
    let root = document.document_element().expect("no html element");
    root.style(style);
}

fn apply_dark_mode_class(scheme: ColorScheme) {
    let document = document();
    let html = document.document_element().expect("no html element");
    let class_list = html.class_list();

    match scheme {
        ColorScheme::Dark => {
            class_list.add_1("dark").ok();
        }
        ColorScheme::Light => {
            class_list.remove_1("dark").ok();
        }
        ColorScheme::System => {
            // System will be handled by media queries
            class_list.remove_1("dark").ok();
        }
    }
}

fn is_dark_mode_preferred() -> bool {
    #[cfg(feature = "hydrate")]
    {
        let window = web_sys::window().expect("no window");
        let media_query = window
            .match_media("(prefers-color-scheme: dark)")
            .ok()
            .flatten();

        media_query.map(|mq| mq.matches()).unwrap_or(false)
    }
    #[cfg(not(feature = "hydrate"))]
    {
        false
    }
}
