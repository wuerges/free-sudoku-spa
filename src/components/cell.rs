use crate::state::AppState;
use leptos::prelude::*;

#[component]
pub fn Cell(state: AppState, row: usize, col: usize) -> impl IntoView {
    let value = move || state.0.get().get(row, col);
    let selected = move || state.0.get().selected == Some((row, col));
    let given = move || state.0.get().is_given(row, col);
    let hinted = move || state.0.get().is_hinted(row, col);
    let error = move || {
        let s = state.0.get();
        let v = s.get(row, col);
        v != 0
            && ((!s.is_given(row, col) && v != s.solution[row * 9 + col])
                || !crate::sudoku_engine::conflicts(&s.board, row, col).is_empty())
    };
    let matching = move || {
        let s = state.0.get();
        s.active_number()
            .is_some_and(|n| s.selected != Some((row, col)) && s.get(row, col) == n)
    };
    let placement = move || state.0.get().placement_available(row, col);
    let blocker = move || state.0.get().placement_blocker(row, col);
    let peer = move || {
        state.0.get().selected.is_some_and(|(r, c)| {
            (r, c) != (row, col) && (r == row || c == col || (r / 3 == row / 3 && c / 3 == col / 3))
        })
    };
    let secondary = move || {
        let s = state.0.get();
        (s.secondary_highlight_rows >> row) & 1 == 1 || (s.secondary_highlight_cols >> col) & 1 == 1
    };
    let cell_state = move || {
        if error() {
            "error"
        } else if hinted() {
            "hint"
        } else if selected() {
            "selected"
        } else if matching() {
            "matching"
        } else if let Some(available) = placement() {
            if available {
                "available"
            } else if blocker() == Some("matching") {
                "matching-blocked"
            } else {
                "blocked"
            }
        } else if peer() {
            "peer"
        } else if secondary() {
            "secondary"
        } else {
            "default"
        }
    };
    let origin = move || {
        if hinted() {
            "hint"
        } else if given() {
            "given"
        } else if value() == 0 {
            "empty"
        } else {
            "user"
        }
    };

    view! {
        <button
            type="button"
            class=move || format!(
                "sudoku-cell relative flex items-center justify-center w-full aspect-square text-lg sm:text-2xl select-none border-[0.5px] transition-colors {}",
                if state.0.get().just_filled == Some((row, col)) { "cell-flash" } else { "" },
            )
            data-row=row data-col=col
            data-cell-state=cell_state
            data-selected=move || selected().to_string()
            data-matching=move || matching().to_string()
            data-placement=move || match placement() {
                Some(true) => "available", Some(false) => "blocked", None => "none",
            }
            data-blocker=move || blocker().unwrap_or("none")
            data-number-origin=origin
            aria-pressed=move || selected().to_string()
            aria-label=move || format!(
                "Linha {}, coluna {}, {}{}{}{}",
                row + 1, col + 1,
                if value() == 0 { "vazia".to_string() } else { format!("número {}", value()) },
                match origin() { "given" => ", número original", "hint" => ", dica", _ => "" },
                if error() { ", erro ou conflito" } else { "" },
                match placement() {
                    Some(true) => state.0.get().active_number().map_or_else(|| ", fora da linha, coluna e bloco selecionados".to_string(), |n| format!(", disponível para {n} pelas regras")),
                    Some(false) => state.0.get().active_number().map_or_else(|| ", na linha, coluna ou bloco selecionado".to_string(), |n| format!(", bloqueada para {n} pelas regras, {}",
                        if blocker() == Some("selected") { "pela célula selecionada" } else { "por outro número igual" })),
                    None => String::new(),
                },
            )
            on:click=move |_| state.select_cell(row, col)
            style="min-width: 0; min-height: 0;"
        >
            <Show when=move || value() != 0 fallback=move || view! {
                <div class="grid grid-cols-3 gap-0 w-full h-full p-[2px]">
                    {(1..=9).map(|note| {
                        let active = move || state.0.get().notes[row * 9 + col] >> (note - 1) & 1 == 1;
                        view! {
                            <span class=move || {
                                let s = state.0.get();
                                let matches = s.active_number() == Some(note);
                                format!("flex items-center justify-center text-[8px] sm:text-[10px] leading-none {}",
                                    if matches { "text-accent font-bold underline" } else { "text-notes" })
                            }>{move || if active() { note.to_string() } else { String::new() }}</span>
                        }
                    }).collect::<Vec<_>>()}
                </div>
            }>
                <span class=move || format!("cell-number {} {}",
                    if error() { "text-error-text" } else if hinted() { "text-hint-text" }
                    else if given() { "text-text" } else { "text-user" },
                    if given() && !hinted() { "font-bold" } else { "font-medium" },
                )>{move || value().to_string()}</span>
            </Show>
            <Show when=error>
                <span class="cell-error-marker" aria-hidden="true">"!"</span>
            </Show>
        </button>
    }
}
