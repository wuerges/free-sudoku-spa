use crate::components::{
    game_controls::GameControls, header::Header, help_page::HelpPage, number_pad::NumberPad,
    sudoku_grid::SudokuGrid,
};
use crate::state::{save_state, AppState};
use leptos::prelude::*;

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

    let show_help = RwSignal::new(false);

    view! {
        <div class=move || format!(
            "min-h-screen flex flex-col items-center p-2 sm:p-4 font-sans transition-colors {}",
            if dark_mode.get() { "bg-gray-900 text-white" } else { "bg-white text-gray-900" },
        )>
            <Header state=state dark_mode=dark_mode show_help=show_help />
            {move || if show_help.get() {
                view! { <HelpPage show_help=show_help /> }.into_any()
            } else {
                view! {
                    <SudokuGrid state=state />
                    <NumberPad state=state />
                    <GameControls state=state />
                }.into_any()
            }}
        </div>
    }
}
