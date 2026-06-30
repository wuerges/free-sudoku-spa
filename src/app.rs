use crate::components::{game_controls::GameControls, header::Header, number_pad::NumberPad, sudoku_grid::SudokuGrid};
use crate::state::{save_state, AppState};
use leptos::prelude::*;

#[component]
pub fn App() -> impl IntoView {
    let state = AppState::new();

    // Timer effect
    let timer_state = state.clone();
    let timer_handle = set_interval(
        move || timer_state.tick_timer(),
        std::time::Duration::from_secs(1),
    );
    // ponytail: timer handle dropped on component unmount. SPA never unmounts.
    let _timer_handle = timer_handle;

    // Persist on change
    let persist_state = state.clone();
    // ponytail: effect persists for component lifetime via reactive owner.
    let _effect = Effect::new(move || {
        let s = persist_state.0.get();
        save_state(&s);
    });

    // ponytail: remove loading div on mount
    if let Some(window) = web_sys::window() {
        if let Some(doc) = window.document() {
            if let Some(el) = doc.get_element_by_id("loading") {
                el.remove();
            }
        }
    }

    let dark_mode = RwSignal::new(false);
    // ponytail: system preference via eval. Add MediaQueryList web-sys feature if native API needed.
    dark_mode.set(
        js_sys::eval("window.matchMedia&&window.matchMedia('(prefers-color-scheme:dark)').matches")
            .map(|v| v.as_bool().unwrap_or(false))
            .unwrap_or(false),
    );

    let theme_class = move || {
        if dark_mode.get() {
            "dark bg-gray-900 text-white"
        } else {
            "bg-white text-gray-900"
        }
    };

    view! {
        <div
            class=move || format!(
                "min-h-screen flex flex-col items-center p-2 sm:p-4 font-sans transition-colors {}",
                theme_class(),
            )
        >
            <Header state=state.clone() dark_mode=dark_mode />
            <SudokuGrid state=state.clone() />
            <NumberPad state=state.clone() />
            <GameControls state=state.clone() />
        </div>
    }
}
