use crate::components::cell::Cell;
use crate::state::AppState;
use leptos::prelude::*;

#[component]
pub fn SudokuGrid(state: AppState) -> impl IntoView {
    view! {
        <div class="w-full max-w-[min(90vw,90vh-280px,500px)] mx-auto mt-2">
            <div class="grid grid-cols-3 gap-0.5 border-2 border-grid-strong rounded-sm overflow-hidden bg-grid-strong">
                {(0..3).flat_map(|br| {
                    (0..3).map(move |bc| {
                        view! {
                            <div class="grid grid-cols-3 bg-cell">
                                {(0..3).flat_map(move |r| {
                                    (0..3).map(move |c| {
                                        let row = br * 3 + r;
                                        let col = bc * 3 + c;
                                        view! { <Cell state=state row=row col=col /> }
                                    }).collect::<Vec<_>>()
                                }).collect::<Vec<_>>()}
                            </div>
                        }
                    }).collect::<Vec<_>>()
                }).collect::<Vec<_>>()}
            </div>
            <Show when=move || state.0.get().active_number().is_some()>
                <p class="text-xs text-muted text-center mt-2 leading-relaxed" aria-live="polite">
                    {move || format!("Para {}: • disponível · sombreada bloqueada", state.0.get().active_number().unwrap_or(0))}
                </p>
            </Show>
        </div>
    }
}
