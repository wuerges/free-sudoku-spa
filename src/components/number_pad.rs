use crate::state::AppState;
use leptos::prelude::*;

#[component]
pub fn NumberPad(state: AppState) -> impl IntoView {
    let note_mode = move || state.0.get().note_mode;
    let drop_mode = move || state.0.get().drop_mode;
    let drop_number = move || state.0.get().drop_number;

    view! {
        <div class="w-full max-w-[min(90vw,500px)] mx-auto mt-2 space-y-1.5">
            // Numbers 1-9 in a single row
            <div class="grid grid-cols-9 gap-1 sm:gap-1.5">
                {(1..=9).map(|v| view! { <NumberBtn state=state v=v /> }).collect::<Vec<_>>()}
            </div>
            // Delete + Notas + Drop
            <div class="grid grid-cols-3 gap-1.5">
                <button
                    class="flex items-center justify-center h-10 sm:h-12 rounded-lg text-sm font-medium bg-error text-error-text active:bg-error transition-colors"
                    on:click=move |_| state.place_number(0)
                >
                    "⌫ Apagar"
                </button>
                <button
                    class=move || format!(
                        "flex items-center justify-center h-10 sm:h-12 rounded-lg text-sm font-bold transition-colors {}",
                        if note_mode() {
                            "bg-primary text-on-primary active:bg-primary-hover"
                        } else {
                            "bg-control text-text active:bg-control-hover"
                        }
                    )
                    on:click=move |_| state.toggle_note_mode()
                >
                    {move || if note_mode() { "📝 Nota ON" } else { "📝 Nota" }}
                </button>
                <button
                    class=move || format!(
                        "flex items-center justify-center h-10 sm:h-12 rounded-lg text-sm font-bold transition-colors {}",
                        if drop_mode() {
                            "bg-primary text-on-primary active:bg-primary-hover"
                        } else {
                            "bg-control text-text active:bg-control-hover"
                        }
                    )
                    on:click=move |_| state.toggle_drop_mode()
                >
                    {move || if drop_mode() {
                        if let Some(n) = drop_number() {
                            format!("🎯 Drop {}", n)
                        } else {
                            "🎯 Drop ON".to_string()
                        }
                    } else {
                        "🎯 Drop".to_string()
                    }}
                </button>
            </div>
        </div>
    }
}

#[component]
fn NumberBtn(state: AppState, v: u8) -> impl IntoView {
    let remaining = move || 9 - state.0.get().board.iter().filter(|&&c| c == v).count();
    let drop_active = move || state.0.get().drop_mode && state.0.get().drop_number == Some(v);

    view! {
        <button
            class=move || {
                if drop_active() {
                    "flex items-center justify-center aspect-square rounded text-lg sm:text-xl font-medium bg-primary text-on-primary active:bg-primary-hover transition-colors"
                } else if remaining() == 0 && !state.0.get().drop_mode {
                    "flex items-center justify-center aspect-square rounded text-lg sm:text-xl font-medium bg-disabled text-muted transition-colors"
                } else {
                    "flex items-center justify-center aspect-square rounded text-lg sm:text-xl font-medium bg-control text-text active:bg-selected transition-colors"
                }
            }
            on:click=move |_| {
                if state.0.get_untracked().drop_mode {
                    state.select_drop_number(v);
                } else {
                    state.place_number(v);
                }
            }
            disabled=move || remaining() == 0 && !state.0.get().drop_mode
        >
            {v.to_string()}
        </button>
    }
}
