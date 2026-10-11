use crate::components::{header::Header, number_pad::NumberPad, sudoku_grid::SudokuGrid};
use crate::state::AppState;
use leptos::prelude::*;

#[component]
pub fn GamePage() -> impl IntoView {
    let state: AppState = use_context().unwrap();
    view! {
        <main class="game-page" aria-label="Jogo de Sudoku">
        <Header />
        <div class="game-play-area">
            <div class="board-panel">
                <SudokuGrid state=state />
            </div>
            <div class="control-panel">
                <NumberPad state=state />
                <Show when=move || state.0.get().won>
                    <div class="text-center text-success-text font-bold" role="status">"Parabéns! Puzzle resolvido!"</div>
                </Show>
            </div>
        </div>
        </main>
    }
}
