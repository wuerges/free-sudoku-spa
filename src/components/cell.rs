use crate::state::AppState;
use leptos::prelude::*;

#[component]
pub fn Cell(state: AppState, row: usize, col: usize) -> impl IntoView {
    let value = move || state.0.get().get(row, col);
    let is_given = move || state.0.get().is_given(row, col);
    let is_selected = move || state.0.get().selected == Some((row, col));
    let notes_mask = move || state.0.get().notes[row * 9 + col];
    let in_conflict = move || {
        let s = state.0.get();
        let v = s.get(row, col);
        if v == 0 { return false; }
        use crate::sudoku_engine::conflicts;
        !conflicts(&s.board, row, col).is_empty()
    };
    let is_highlighted = move || {
        if let Some((sr, sc)) = state.0.get().selected {
            (row == sr || col == sc) && Some((row, col)) != state.0.get().selected
        } else {
            false
        }
    };
    let same_value = move || {
        let s = state.0.get();
        let v = s.get(row, col);
        if v == 0 || s.selected.is_none() { return false; }
        let (sr, sc) = s.selected.unwrap();
        v == s.get(sr, sc) && v != 0 && (row != sr || col != sc)
    };

    let is_primary_highlighted = move || {
        let s = state.0.get();
        if s.secondary_highlight_value.is_none() { return false; }
        if let Some((sr, sc)) = s.selected {
            (row == sr || col == sc) && Some((row, col)) != s.selected
        } else { false }
    };
    let is_secondary_highlighted = move || {
        let s = state.0.get();
        if s.secondary_highlight_rows == 0 && s.secondary_highlight_cols == 0 {
            return false;
        }
        (s.secondary_highlight_rows >> row) & 1 == 1
            || (s.secondary_highlight_cols >> col) & 1 == 1
    };

    let highlighted_number = move || {
        let s = state.0.get();
        if let Some((sr, sc)) = s.selected {
            let v = s.get(sr, sc);
            if v != 0 { v } else { 0 }
        } else {
            0
        }
    };

    let flash = move || state.0.get().just_filled == Some((row, col));
    let is_hinted = move || state.0.get().is_hinted(row, col);
    let is_wrong = move || {
        let s = state.0.get();
        let v = s.get(row, col);
        v != 0 && !s.is_given(row, col) && v != s.solution[row * 9 + col]
    };

    view! {
        <button
            class=move || {
                let mut cls = String::from(
                    "relative flex items-center justify-center w-full aspect-square text-lg sm:text-2xl font-medium select-none border-[0.5px] border-gray-300 dark:border-gray-600 transition-colors",
                );
                if is_selected() { cls.push_str(" bg-blue-200 dark:bg-blue-800"); }
                else if is_primary_highlighted() { cls.push_str(" bg-blue-100 dark:bg-blue-900/50"); }
                else if same_value() { cls.push_str(" bg-blue-100 dark:bg-blue-900/50"); }
                else if is_highlighted() { cls.push_str(" bg-blue-50 dark:bg-blue-950/50"); }
                else if is_secondary_highlighted() { cls.push_str(" bg-blue-secondary dark:bg-blue-secondary"); }
                else if is_hinted() { cls.push_str(" bg-amber-100 dark:bg-amber-900/30"); }
                else if in_conflict() { cls.push_str(" bg-red-100 dark:bg-red-900/50"); }
                else { cls.push_str(" hover:bg-gray-100 dark:hover:bg-gray-800"); }
                if flash() { cls.push_str(" cell-flash"); }
                cls
            }
            on:click=move |_| state.select_cell(row, col)
style="min-width: 0; min-height: 0;"
        >
            <Show when=move || value() != 0 fallback=move || {
                let hv = highlighted_number();
                view! {
                    <div class="grid grid-cols-3 gap-0 w-full h-full p-[2px]">
                        {(1..=9).map(|note_val| {
                            let active = notes_mask() >> (note_val - 1) & 1 == 1;
                            let color = if active {
                                if note_val == hv { "text-blue-500 dark:text-blue-400" } else { "text-gray-400 dark:text-gray-400" }
                            } else { "" };
                            view! {
                                <span class=format!("flex items-center justify-center text-[8px] sm:text-[10px] leading-none {}", color)>
                                    { if active { note_val.to_string() } else { "".to_string() } }
                                </span>
                            }
                        }).collect::<Vec<_>>()}
                    </div>
                }
            }>
                {let v = value();
                let color = if is_hinted() {
                    "text-amber-700 dark:text-amber-300"
                } else if is_given() {
                    "text-gray-900 dark:text-white"
                } else if is_wrong() {
                    "text-red-600 dark:text-red-400"
                } else {
                    "text-blue-600 dark:text-blue-400"
                };
                view! { <span class=format!("{color} font-{}", if is_given() && !is_hinted() { "bold" } else { "semibold" })>{v.to_string()}</span> }}
            </Show>
        </button>
    }
}
