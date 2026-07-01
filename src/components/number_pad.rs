use crate::state::AppState;
use leptos::prelude::*;

#[component]
pub fn NumberPad(state: AppState) -> impl IntoView {
    let note_mode = move || state.0.get().note_mode;

    view! {
        <div class="w-full max-w-[min(90vw,500px)] mx-auto mt-2 space-y-1.5">
            // Numbers 1-9 in a single row
            <div class="grid grid-cols-9 gap-1 sm:gap-1.5">
                {(1..=9).map(|v| view! { <NumberBtn state=state v=v /> }).collect::<Vec<_>>()}
            </div>
            // Delete + Note below
            <div class="grid grid-cols-2 gap-1.5">
                <button
                    class="flex items-center justify-center h-10 sm:h-12 rounded-lg text-sm font-medium bg-red-100 dark:bg-red-900/30 text-red-700 dark:text-red-300 active:bg-red-200 transition-colors"
                    on:click=move |_| state.place_number(0)
                >
                    "⌫ Apagar"
                </button>
                <button
                    class=move || format!(
                        "flex items-center justify-center h-10 sm:h-12 rounded-lg text-sm font-bold active:opacity-70 transition-colors {}",
                        if note_mode() {
                            "bg-blue-500 text-white"
                        } else {
                            "bg-gray-200 dark:bg-gray-700 text-gray-700 dark:text-gray-300"
                        }
                    )
                    on:click=move |_| state.toggle_note_mode()
                >
                    {move || if note_mode() { "📝 Nota ON" } else { "📝 Nota" }}
                </button>
            </div>
        </div>
    }
}

#[component]
fn NumberBtn(state: AppState, v: u8) -> impl IntoView {
    let remaining = move || 9 - state.0.get().board.iter().filter(|&&c| c == v).count();
    view! {
        <button
            class=move || {
                if remaining() == 0 {
                    "flex items-center justify-center aspect-square rounded text-lg sm:text-xl font-medium bg-gray-100 dark:bg-gray-800 text-gray-300 dark:text-gray-600 transition-colors"
                } else {
                    "flex items-center justify-center aspect-square rounded text-lg sm:text-xl font-medium bg-gray-100 dark:bg-gray-800 text-gray-800 dark:text-gray-200 active:bg-blue-100 dark:active:bg-blue-900 transition-colors"
                }
            }
            on:click=move |_| state.place_number(v)
            disabled=move || remaining() == 0
        >
            {v.to_string()}
        </button>
    }
}
