use crate::components::icon::{Icon, IconName};
use crate::state::AppState;
use crate::sudoku_engine::Difficulty;
use leptos::prelude::*;

fn format_time(seconds: u32) -> String {
    let m = (seconds % 3600) / 60;
    let s = seconds % 60;
    format!("{m:02}:{s:02}")
}

#[component]
pub fn GameStatus(state: AppState) -> impl IntoView {
    let paused = move || state.0.get().paused;
    view! {
        <div class="game-status" role="group" aria-label="Estado do jogo">
            <span class="game-time flex items-center gap-2" aria-label=move || format!("Tempo de jogo: {}", format_time(state.0.get().timer_seconds))>
                <Icon name=IconName::Clock /><span class="font-mono tabular-nums">{move || format_time(state.0.get().timer_seconds)}</span>
            </span>
            <span class="text-sm text-muted">{move || format!("Erros: {}", state.0.get().error_count)}</span>
            <button class="ui-icon-button" aria-label=move || if paused() { "Retomar jogo" } else { "Pausar jogo" }
                title=move || if paused() { "Retomar jogo" } else { "Pausar jogo" }
                aria-pressed=move || paused().to_string() on:click=move |_| state.toggle_pause()>
                {move || view! { <Icon name=if paused() { IconName::Play } else { IconName::Pause } /> }}
            </button>
        </div>
    }
}

#[component]
pub fn GameControls(state: AppState) -> impl IntoView {
    let show_new_game = RwSignal::new(false);
    let show_actions = RwSignal::new(false);
    let trigger = NodeRef::<leptos::html::Button>::new();
    let close = move || {
        show_actions.set(false);
        show_new_game.set(false);
        if let Some(button) = trigger.get() {
            let _ = button.focus();
        }
    };
    view! {
        <div class="game-actions" on:keydown=move |event| {
            if event.key() == "Escape" && show_actions.get_untracked() {
                event.prevent_default(); close();
            }
        }>
            <button type="button" class="ui-action actions-trigger" node_ref=trigger
                aria-expanded=move || show_actions.get().to_string() aria-controls="game-actions-panel"
                on:click=move |_| { if show_actions.get_untracked() { close(); } else { show_actions.set(true); } }>
                <Icon name=IconName::Menu />"Ações"
            </button>
            <Show when=move || show_actions.get()>
                <div class="actions-dismiss-layer" aria-hidden="true" on:click=move |_| close()></div>
                <div id="game-actions-panel" class="actions-panel" role="region" aria-label="Ações do jogo">
                    <div class="actions-panel-header">
                        <h2>"Ações do jogo"</h2>
                        <button type="button" class="ui-icon-button" aria-label="Fechar ações" on:click=move |_| close()><Icon name=IconName::Close /></button>
                    </div>
                    <div class="assistance-actions" role="group" aria-label="Assistências do jogo">
                        <Show when=move || state.0.get().undo_enabled>
                            <button class="ui-action" on:click=move |_| state.undo() disabled=move || state.0.get().history.is_empty()>
                                <Icon name=IconName::Undo />"Desfazer"
                            </button>
                            <button class="ui-action" on:click=move |_| state.redo() disabled=move || state.0.get().redo_stack.is_empty()>
                                <Icon name=IconName::Redo />"Refazer"
                            </button>
                        </Show>
                        <Show when=move || state.0.get().auto_notes_enabled>
                            <button class="ui-action" on:click=move |_| state.auto_notes()><Icon name=IconName::AutoNotes />"Auto notas"</button>
                        </Show>
                        <Show when=move || state.0.get().hint_enabled>
                            <button class="ui-action ui-hint-action" on:click=move |_| state.hint()><Icon name=IconName::Hint />"Dica"</button>
                        </Show>
                    </div>
                    <button class="ui-action new-game-action" aria-expanded=move || show_new_game.get().to_string()
                        aria-controls="difficulty-choices" on:click=move |_| show_new_game.update(|v| *v = !*v)>
                        <Icon name=IconName::Reset />"Novo jogo"
                    </button>
                    <Show when=move || show_new_game.get()>
                        <div id="difficulty-choices" class="difficulty-choices" role="group" aria-label="Dificuldade do novo jogo">
                            {[Difficulty::Easy, Difficulty::Medium, Difficulty::Hard, Difficulty::Expert, Difficulty::Master].into_iter().map(|difficulty| {
                                let label = match difficulty { Difficulty::Easy => "Fácil", Difficulty::Medium => "Médio",
                                    Difficulty::Hard => "Difícil", Difficulty::Expert => "Expert", Difficulty::Master => "Mestre" };
                                view! {
                                    <button class="ui-action" data-active=move || (state.0.get().difficulty == difficulty).to_string()
                                        on:click=move |_| { state.new_game(difficulty); close(); }>{label}</button>
                                }
                            }).collect::<Vec<_>>()}
                        </div>
                    </Show>
                </div>
            </Show>
        </div>
    }
}
