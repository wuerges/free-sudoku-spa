use crate::components::{game_controls::GameControls, header::Header, number_pad::NumberPad, sudoku_grid::SudokuGrid};
use crate::state::AppState;
use leptos::prelude::*;

#[component]
pub fn GamePage() -> impl IntoView {
    let state: AppState = use_context().unwrap();
    view! {
        <Header />
        <SudokuGrid state=state />
        <NumberPad state=state />
        <GameControls state=state />
    }
}
