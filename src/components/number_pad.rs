use crate::components::{
    game_controls::GameControls,
    icon::{Icon, IconName},
};
use crate::state::{color_name, AppState};
use leptos::prelude::*;

#[component]
pub fn NumberPad(state: AppState) -> impl IntoView {
    let note_mode = move || state.0.get().note_mode;
    let drop_mode = move || state.0.get().drop_mode;
    let drop_number = move || state.0.get().drop_number;

    view! {
        <div class="number-pad" style=move || format!("--completed-contrast: {}%;", state.0.get().highlights.completed_contrast.min(100))>
            <div class="number-choices" role="group" aria-label="Números">
                {(1..=9).map(|v| view! { <NumberBtn state=state v=v /> }).collect::<Vec<_>>()}
            </div>
            <div class="color-tools" role="group" aria-label="Cores das células">
                {(1..=4u8).map(|color| view! {
                    <button type="button" class="ui-action color-tool" data-color=color
                        aria-label=color_name(color) title=color_name(color)
                        aria-pressed=move || (state.0.get().paint_tool == Some(color)).to_string()
                        disabled=move || { let s = state.0.get(); s.paused || s.won || (!s.drop_mode && s.selected.is_none()) }
                        on:click=move |_| state.select_paint_tool(color)>
                        <span class="color-swatch" aria-hidden="true"></span>
                        <span class="sr-only">{color_name(color)}</span>
                    </button>
                }).collect::<Vec<_>>()}
                <button type="button" class="ui-action color-tool" aria-label="Limpar cores" title="Limpar cores"
                    disabled=move || { let s = state.0.get(); s.paused || s.won || s.cell_colors.iter().all(|&c| c == 0) }
                    on:click=move |_| state.clear_colors()><Icon name=IconName::Reset /><span aria-hidden="true">"Limpar"</span></button>
            </div>
            <div class="entry-actions" role="group" aria-label="Modo de entrada">
                <button class="ui-action ui-erase-action" on:click=move |_| state.place_number(0)>
                    <Icon name=IconName::Erase />"Apagar"
                </button>
                <button class="ui-action" data-active=move || note_mode().to_string()
                    aria-pressed=move || note_mode().to_string() on:click=move |_| state.toggle_note_mode()>
                    <Icon name=IconName::Pencil />"Notas"
                </button>
                <button class="ui-action" data-active=move || drop_mode().to_string()
                    aria-pressed=move || drop_mode().to_string() on:click=move |_| state.toggle_drop_mode()>
                    <Icon name=IconName::Drop />"Drop"
                    <span class="drop-number" aria-hidden="true">{move || drop_number().map(|n| n.to_string()).unwrap_or_default()}</span>
                </button>
                <GameControls state=state />
            </div>
            <p class="paint-instruction" role="status" aria-live="polite">{move || state.0.get().paint_tool.map(|color|
                format!("{}: pinte células; toque na cor para sair.", color_name(color))).unwrap_or_default()}</p>
            <Show when=move || drop_mode() && state.0.get().paint_tool.is_none()>
                <p class="drop-instruction" role="status">{move || match drop_number() {
                    Some(number) => format!("Drop: número {number}. Toque nas células para aplicar."),
                    None => "Drop: escolha um número e toque nas células.".to_string(),
                }}</p>
            </Show>
        </div>
    }
}

#[component]
fn NumberBtn(state: AppState, v: u8) -> impl IntoView {
    let disabled = move || {
        let s = state.0.get();
        if s.drop_mode {
            s.paint_tool.is_none() && !s.note_mode && s.number_is_solved(v)
        } else {
            s.board.iter().filter(|&&c| c == v).count() >= 9
        }
    };
    let completed = move || state.0.get().number_is_solved(v);
    let description_id = format!("number-{v}-completion");
    let drop_active = move || state.0.get().drop_mode && state.0.get().drop_number == Some(v);

    view! {
        <button
            aria-label=v.to_string()
            aria-pressed=move || drop_active().to_string()
            class="relative flex items-center justify-center number-choice w-full rounded font-medium transition-colors"
            data-completed=move || completed().to_string()
            aria-describedby=description_id.clone()
            title=move || if completed() { format!("{v}: concluído, nove ocorrências corretas") } else { v.to_string() }
            on:click=move |_| {
                if state.0.get_untracked().drop_mode {
                    state.select_drop_number(v);
                } else {
                    state.place_number(v);
                }
            }
            disabled=disabled
        >
            {v.to_string()}
            <Show when=completed><span class="number-complete-marker" aria-hidden="true">"✓"</span></Show>
            <span id=description_id.clone() class="sr-only">{move || if completed() { "Número concluído: nove ocorrências corretas." } else { "" }}</span>
        </button>
    }
}
