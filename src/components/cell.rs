use crate::i18n::{use_i18n, Message};
use crate::state::AppState;
use leptos::prelude::*;

#[component]
pub fn Cell(state: AppState, row: usize, col: usize) -> impl IntoView {
    let i18n = use_i18n();
    let value = move || state.0.get().get(row, col);
    let selected = move || {
        let s = state.0.get();
        !s.drop_mode && s.selected == Some((row, col))
    };
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
        s.active_number().is_some_and(|n| {
            (s.drop_mode || s.selected != Some((row, col))) && s.get(row, col) == n
        })
    };
    let placement = move || state.0.get().placement_available(row, col);
    let blocker = move || state.0.get().placement_blocker(row, col);
    let peer = move || {
        let s = state.0.get();
        !s.drop_mode
            && s.selected.is_some_and(|(r, c)| {
                (r, c) != (row, col)
                    && (r == row || c == col || (r / 3 == row / 3 && c / 3 == col / 3))
            })
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
                "sudoku-cell relative flex items-center justify-center w-full aspect-square select-none border-[0.5px] transition-colors {}",
                if state.0.get().just_filled == Some((row, col)) { "cell-flash" } else { "" },
            )
            data-row=row data-col=col
            data-color=move || state.0.get().cell_colors[row * 9 + col]
            data-cell-state=cell_state
            data-selected=move || selected().to_string()
            data-matching=move || matching().to_string()
            data-placement=move || match placement() {
                Some(true) => "available", Some(false) => "blocked", None => "none",
            }
            data-blocker=move || blocker().unwrap_or("none")
            data-number-origin=origin
            aria-pressed=move || selected().to_string()
            aria-label=move || {
                let placement = match placement() {
                    Some(true) => state.0.get().active_number().map_or_else(
                        || i18n.text(Message::OutsideUnits).to_string(),
                        |n| i18n.format(Message::AvailableFor, &[("number", n.to_string())])),
                    Some(false) => state.0.get().active_number().map_or_else(
                        || i18n.text(Message::InsideUnits).to_string(),
                        |n| i18n.format(Message::BlockedFor, &[("number", n.to_string()),
                            ("source", i18n.text(if blocker() == Some("selected") { Message::SelectedBlocker } else { Message::MatchingBlocker }).into())])),
                    None => String::new(),
                };
                let color = state.0.get().cell_colors[row * 9 + col];
                i18n.format(Message::CellLabel, &[
                    ("row", (row + 1).to_string()), ("column", (col + 1).to_string()),
                    ("value", if value() == 0 { i18n.text(Message::Empty).into() }
                        else { i18n.format(Message::CellNumber, &[("number", value().to_string())]) }),
                    ("origin", match origin() { "given" => i18n.text(Message::OriginalNumber),
                        "hint" => i18n.text(Message::HintCell), _ => "" }.into()),
                    ("error", if error() { i18n.text(Message::ErrorCell) } else { "" }.into()),
                    ("placement", placement),
                    ("color", if color == 0 { String::new() } else { i18n.format(Message::CellColor, &[("color", i18n.color(color).to_lowercase())]) }),
                ])
            }
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
                                format!("cell-note flex items-center justify-center leading-none {}",
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
