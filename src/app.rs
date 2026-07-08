use crate::components::{
    config_page::ConfigPage, game_page::GamePage, help_page::HelpPage,
};
use crate::state::{save_state, AppState};
use leptos::prelude::*;
use leptos_router::components::{Route, Router, Routes};
use leptos_router::path;

fn set_html_dark(dark: bool) {
    if let Some(doc) = web_sys::window().and_then(|w| w.document()) {
        if let Some(html) = doc.document_element() {
            let _ = if dark {
                html.class_list().add_1("dark")
            } else {
                html.class_list().remove_1("dark")
            };
        }
    }
}

#[component]
pub fn App() -> impl IntoView {
    let state = AppState::new();

    // Timer
    let timer_state = state;
    set_interval(
        move || timer_state.tick_timer(),
        std::time::Duration::from_secs(1),
    );

    // Persist
    let persist_state = state;
    let _effect = Effect::new(move || {
        let s = persist_state.0.get();
        save_state(&s);
    });

    // Remove loading div after 2s
    set_timeout(
        || {
            if let Some(doc) = web_sys::window().and_then(|w| w.document()) {
                if let Some(el) = doc.get_element_by_id("loading") {
                    if let Some(parent) = el.parent_node() {
                        let _ = parent.remove_child(&el);
                    }
                }
            }
        },
        std::time::Duration::from_secs(2),
    );

    // Dark mode — class on <html> so Tailwind dark: variants work
    let dark_mode = RwSignal::new(
        js_sys::eval("window.matchMedia&&window.matchMedia('(prefers-color-scheme:dark)').matches")
            .map(|v| v.as_bool().unwrap_or(false))
            .unwrap_or(false),
    );
    set_html_dark(dark_mode.get());

    let _sync = Effect::new(move || {
        set_html_dark(dark_mode.get());
    });

    provide_context(state);
    provide_context(dark_mode);

    view! {
        <style>
            "@keyframes cell-pop { 0%{transform:scale(1)}20%{transform:scale(1.12)}100%{transform:scale(1)} }
            .cell-flash{animation:cell-pop .35s ease-out;z-index:1;position:relative}"
        </style>
        <Router>
            <div class=move || format!(
                "min-h-screen flex flex-col items-center p-2 sm:p-4 font-sans transition-colors {}",
                if dark_mode.get() { "bg-gray-900 text-white" } else { "bg-white text-gray-900" },
            )>
                <Routes fallback=|| "Not found">
                    <Route path=path!("") view=GamePage />
                    <Route path=path!("/help") view=HelpPage />
                    <Route path=path!("/config") view=ConfigPage />
                </Routes>
            </div>
        </Router>
    }
}
