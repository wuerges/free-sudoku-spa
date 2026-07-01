use crate::components::cell::Cell;
use crate::state::AppState;
use leptos::prelude::*;

#[component]
pub fn SudokuGrid(state: AppState) -> impl IntoView {
    view! {
        <div class="w-full max-w-[min(90vw,90vh-280px,500px)] mx-auto mt-2">
            <div class="grid grid-cols-3 gap-0.5 border-2 border-gray-800 dark:border-gray-200 rounded-sm overflow-hidden bg-gray-300 dark:bg-gray-700">
                {(0..3).map(|br| {
                    (0..3).map(move |bc| {
                        view! {
                            <div class="grid grid-cols-3 bg-white dark:bg-gray-900">
                                {(0..3).map(move |r| {
                                    (0..3).map(move |c| {
                                        let row = br * 3 + r;
                                        let col = bc * 3 + c;
                                        view! { <Cell state=state row=row col=col /> }
                                    }).collect::<Vec<_>>()
                                }).flatten().collect::<Vec<_>>()}
                            </div>
                        }
                    }).collect::<Vec<_>>()
                }).flatten().collect::<Vec<_>>()}
            </div>
        </div>
    }
}
