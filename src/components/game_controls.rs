use crate::components::icon::{Icon, IconName};
use crate::i18n::{use_i18n, Message};
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
    let i18n = use_i18n();
    let paused = move || state.0.get().paused;
    view! {
        <div class="game-status" role="group" aria-label=move || i18n.text(Message::GameStatus)>
            <span class="game-time flex items-center gap-2" aria-label=move || i18n.format(Message::GameTime, &[("time", format_time(state.0.get().timer_seconds))])>
                <Icon name=IconName::Clock /><span class="font-mono tabular-nums">{move || format_time(state.0.get().timer_seconds)}</span>
            </span>
            <span class="text-sm text-muted">{move || i18n.format(Message::ErrorCount, &[("count", state.0.get().error_count.to_string())])}</span>
            <button class="ui-icon-button" aria-label=move || if paused() { i18n.text(Message::Resume) } else { i18n.text(Message::PauseGame) }
                title=move || if paused() { i18n.text(Message::Resume) } else { i18n.text(Message::PauseGame) }
                aria-pressed=move || paused().to_string() on:click=move |_| state.toggle_pause()>
                {move || view! { <Icon name=if paused() { IconName::Play } else { IconName::Pause } /> }}
            </button>
        </div>
    }
}

#[component]
pub fn GameControls(state: AppState) -> impl IntoView {
    let i18n = use_i18n();
    let history = expect_context::<crate::state::GameHistory>();
    let sharing = expect_context::<crate::components::share_dialog::ShareUi>().0;
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
                <Icon name=IconName::Menu />{move || i18n.text(Message::Actions)}
            </button>
            <Show when=move || show_actions.get()>
                <div class="actions-dismiss-layer" aria-hidden="true" on:click=move |_| close()></div>
                <div id="game-actions-panel" class="actions-panel" role="region" aria-label=move || i18n.text(Message::GameActions)>
                    <div class="actions-panel-header">
                        <h2>{move || i18n.text(Message::GameActions)}</h2>
                        <button type="button" class="ui-icon-button" aria-label=move || i18n.text(Message::CloseActions) on:click=move |_| close()><Icon name=IconName::Close /></button>
                    </div>
                    <div class="assistance-actions" role="group" aria-label=move || i18n.text(Message::GameAssistances)>
                        <Show when=move || state.0.get().undo_enabled>
                            <button class="ui-action" on:click=move |_| state.undo() disabled=move || state.0.get().history.is_empty()>
                                <Icon name=IconName::Undo />{move || i18n.text(Message::Undo)}
                            </button>
                            <button class="ui-action" on:click=move |_| state.redo() disabled=move || state.0.get().redo_stack.is_empty()>
                                <Icon name=IconName::Redo />{move || i18n.text(Message::Redo)}
                            </button>
                        </Show>
                        <Show when=move || state.0.get().auto_notes_enabled>
                            <button class="ui-action" on:click=move |_| state.auto_notes()><Icon name=IconName::AutoNotes />{move || i18n.text(Message::AutoNotes)}</button>
                        </Show>
                        <Show when=move || state.0.get().hint_enabled>
                            <button class="ui-action ui-hint-action" on:click=move |_| state.hint()><Icon name=IconName::Hint />{move || i18n.text(Message::Hint)}</button>
                        </Show>
                    </div>
                    <leptos_router::components::A href="/history" attr:class="ui-action">{move || i18n.text(Message::History)}</leptos_router::components::A>
                    <button id="share-trigger" class="ui-action" on:click=move |_| sharing.set(true)>{move || i18n.text(Message::Share)}</button>
                    <button class="ui-action new-game-action" aria-expanded=move || show_new_game.get().to_string()
                        aria-controls="difficulty-choices" on:click=move |_| show_new_game.update(|v| *v = !*v)>
                        <Icon name=IconName::Reset />{move || i18n.text(Message::NewGame)}
                    </button>
                    <Show when=move || show_new_game.get()>
                        <div id="difficulty-choices" class="difficulty-choices" role="group" aria-label=move || i18n.text(Message::NewDifficulty)>
                            {[Difficulty::Easy, Difficulty::Medium, Difficulty::Hard, Difficulty::Expert, Difficulty::Master].into_iter().map(|difficulty| {
                                let label = move || i18n.difficulty(difficulty);
                                view! {
                                    <button class="ui-action" data-active=move || (state.0.get().difficulty == difficulty).to_string()
                                        disabled=move || history.busy.get() || !history.ready.get()
                                        on:click=move |_| { history.new_game(state,difficulty); close(); }>{label}</button>
                                }
                            }).collect::<Vec<_>>()}
                        </div>
                    </Show>
                </div>
            </Show>
        </div>
    }
}
