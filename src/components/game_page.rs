use crate::components::{header::Header, number_pad::NumberPad, sudoku_grid::SudokuGrid};
use crate::i18n::{use_i18n, Message};
use crate::state::AppState;
use leptos::prelude::*;

#[component]
pub fn GamePage() -> impl IntoView {
    let i18n = use_i18n();
    let state: AppState = use_context().unwrap();
    view! {
        <main class="game-page" aria-label=move || i18n.text(Message::SudokuGame)>
        <Header />
        <div class="game-play-area">
            <div class="board-panel">
                <SudokuGrid state=state />
            </div>
            <div class="control-panel">
                <NumberPad state=state />
                <Show when=move || state.0.get().won>
                    <div class="text-center text-success-text font-bold" role="status">{move || i18n.text(Message::Victory)}</div>
                </Show>
            </div>
        </div>
        </main>
    }
}
