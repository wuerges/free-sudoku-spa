use crate::state::AppState;
use leptos::prelude::*;

#[component]
pub fn NumberPad(state: AppState) -> impl IntoView {
    let note_mode = move || state.0.get().note_mode;

    view! {
        <div class="w-full max-w-[min(90vw,500px)] mx-auto mt-2">
            <div class="grid grid-cols-5 gap-1.5 sm:gap-2">
                { (1..=5).map(|v| number_btn(state, v)).collect::<Vec<_>>() }
                <button
                    class="flex items-center justify-center h-12 sm:h-14 rounded-lg text-sm font-medium bg-red-100 dark:bg-red-900/30 text-red-700 dark:text-red-300 active:bg-red-200 transition-colors"
                    on:click=move |_| state.place_number(0)
                >
                    "⌫"
                </button>
                { (6..=9).map(|v| number_btn(state, v)).collect::<Vec<_>>() }
                <button
                    class=move || format!(
                        "flex items-center justify-center h-12 sm:h-14 rounded-lg text-sm font-bold active:opacity-70 transition-colors {}",
                        if note_mode() {
                            "bg-blue-500 text-white"
                        } else {
                            "bg-gray-200 dark:bg-gray-700 text-gray-700 dark:text-gray-300"
                        }
                    )
                    on:click=move |_| state.toggle_note_mode()
                >
                    {move || if note_mode() { "Nota ON" } else { "Nota" }}
                </button>
            </div>
        </div>
    }
}

fn number_btn(state: AppState, v: u8) -> impl IntoView {
    let count = move || {
        let board = state.0.get().board;
        let remaining = 9 - board.iter().filter(|&&c| c == v).count();
        remaining
    };
    view! {
        <button
            class=move || {
                let rem = count();
                if rem == 0 {
                    "flex flex-col items-center justify-center h-12 sm:h-14 rounded-lg text-lg sm:text-xl font-medium bg-gray-100 dark:bg-gray-800 text-gray-300 dark:text-gray-600 transition-colors".to_string()
                } else {
                    "flex flex-col items-center justify-center h-12 sm:h-14 rounded-lg text-lg sm:text-xl font-medium bg-gray-100 dark:bg-gray-800 text-gray-800 dark:text-gray-200 active:bg-blue-100 dark:active:bg-blue-900 transition-colors".to_string()
                }
            }
            on:click=move |_| state.place_number(v)
            disabled=move || count() == 0
        >
            <span>{v.to_string()}</span>
            <span class="text-[9px] leading-none text-gray-400">{move || count()}</span>
        </button>
    }
}
