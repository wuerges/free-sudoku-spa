use crate::components::{
    game_controls::{GameControls, GameStatus},
    header::Header,
    number_pad::NumberPad,
    sudoku_grid::SudokuGrid,
};
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
                <GameStatus state=state />
                <SudokuGrid state=state />
            </div>
            <div class="control-panel">
                <NumberPad state=state />
                <GameControls state=state />
            </div>
        </div>
        </main>
    }
}
