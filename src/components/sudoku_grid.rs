use crate::components::cell::Cell;
use crate::state::AppState;
use leptos::prelude::*;

#[component]
pub fn SudokuGrid(state: AppState) -> impl IntoView {
    view! {
        <div class="w-full max-w-[min(90vw,90vh-280px,500px)] mx-auto mt-2">
            <div class="grid grid-cols-9 border-2 border-gray-800 dark:border-gray-200 rounded-sm overflow-hidden">
                {(0..9).map(|row| {
                    (0..9).map(move |col| {
                        view! { <Cell state=state.clone() row=row col=col /> }
                    }).collect::<Vec<_>>()
                }).flatten().collect::<Vec<_>>()}
            </div>
        </div>
    }
}
