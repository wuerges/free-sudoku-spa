use crate::i18n::{use_i18n, I18n, Message};
use crate::{
    components::board_preview::BoardPreview,
    game_history::{evictions, HistoryEntry},
    sharing::SharedGame,
    state::{AppState, GameHistory},
};
use leptos::prelude::*;
use leptos_router::{components::A, hooks::use_navigate};

fn details(i18n: I18n, game: &SharedGame) -> String {
    i18n.format(
        Message::HistorySummary,
        &[
            ("difficulty", i18n.difficulty(game.difficulty).into()),
            ("minutes", (game.seconds / 60).to_string()),
            ("seconds", (game.seconds % 60).to_string()),
            ("errors", game.errors.to_string()),
            (
                "status",
                i18n.text(if game.board == game.solution {
                    Message::Completed
                } else {
                    Message::InProgress
                })
                .into(),
            ),
        ],
    )
}

#[component]
pub fn HistoryFeedback(state: AppState, history: GameHistory) -> impl IntoView {
    let i18n = use_i18n();
    Effect::new(move || {
        let busy = history.busy.get();
        let _ = js_sys::eval(if busy {
            "document.getElementById('history-saving').showModal()"
        } else {
            "document.getElementById('history-saving').close()"
        });
    });
    view! {
        <dialog id="history-saving" class="share-dialog" aria-labelledby="saving-title" on:cancel=move |event: web_sys::Event| event.prevent_default()>
            <h2 id="saving-title">{move || i18n.text(Message::SavingHistory)}</h2><p>{move || i18n.text(Message::PleaseWait)}</p>
        </dialog>
        <Show when=move || history.error.get().is_some()>
            <div class="history-error" role="alert">
                <p>{move || i18n.notice(history.error.get())}</p>
                <button class="ui-action" disabled=move || history.busy.get() on:click=move |_|history.refresh(state)>{move || i18n.text(Message::TryAgain)}</button>
                <button class="ui-action" on:click=move |_|history.error.set(None)>{move || i18n.text(Message::CloseNotice)}</button>
            </div>
        </Show>
    }
}

#[component]
fn HistoryCard(
    entry: HistoryEntry,
    active: bool,
    selected: RwSignal<Option<HistoryEntry>>,
    initial: RwSignal<bool>,
) -> impl IntoView {
    let i18n = use_i18n();
    let state = expect_context::<AppState>();
    let history = expect_context::<GameHistory>();
    let entry = StoredValue::new(entry);
    let game = move || {
        if active {
            SharedGame::capture(&state.0.get(), false)
        } else {
            entry.get_value().game
        }
    };
    let pin = move |_| {
        let value = entry.get_value();
        if value.pinned {
            let mut entries = history.entries.get_untracked();
            for e in &mut entries {
                if e.id == value.id {
                    e.pinned = false;
                }
            }
            if evictions(&entries, &state.0.get_untracked().game_id).contains(&value.id)
                && !web_sys::window()
                    .and_then(|w| {
                        w.confirm_with_message(i18n.text(Message::UnpinConfirm))
                            .ok()
                    })
                    .unwrap_or(false)
            {
                return;
            }
        }
        history.set_pinned(state, value.id, !value.pinned);
    };
    view! {
        <article class="history-card" data-game-id=entry.get_value().id data-pinned=entry.get_value().pinned.to_string()>
            <div>{move || view! {<BoardPreview game=game() miniature=true/>}}</div>
            <div class="history-card-info">
                <h3>{move || if active {i18n.text(Message::CurrentGame)}else if entry.get_value().pinned {i18n.text(Message::PinnedGame)}else{i18n.text(Message::SavedGame)}}</h3>
                <p>{move || details(i18n, &game())}</p>
                <p class="text-muted text-sm">{move || i18n.format(Message::LastPlayed, &[("date", i18n.date(entry.get_value().last_played_ms))])}</p>
                <div class="history-card-actions">
                    <Show when=move || !active>
                        <button class="ui-action" on:click=move |_| {initial.set(false);selected.set(Some(entry.get_value()));}>{move || i18n.text(Message::OpenGame)}</button>
                    </Show>
                    <button id=format!("pin-{}",entry.get_value().id) class="ui-action" aria-pressed=entry.get_value().pinned.to_string()
                        on:click=pin>{move || if entry.get_value().pinned {i18n.text(Message::Unpin)}else{i18n.text(Message::Pin)}}</button>
                </div>
            </div>
        </article>
    }
}

#[component]
pub fn HistoryPage() -> impl IntoView {
    let i18n = use_i18n();
    let state = expect_context::<AppState>();
    let history = expect_context::<GameHistory>();
    let selected = RwSignal::new(None::<HistoryEntry>);
    let initial = RwSignal::new(false);
    let navigate = use_navigate();
    Effect::new(move || history.refresh(state));
    Effect::new(move || {
        let _ = js_sys::eval(if selected.get().is_some() {
            "document.getElementById('history-load').showModal()"
        } else {
            "document.getElementById('history-load').close()"
        });
    });
    let active_id = Memo::new(move |_| state.0.with(|s| s.game_id.clone()));
    let records = move |pinned: bool| {
        let active = active_id.get();
        let mut entries: Vec<_> = history
            .entries
            .get()
            .into_iter()
            .filter(|e| e.id != active && e.pinned == pinned)
            .collect();
        entries.sort_by_key(|e| std::cmp::Reverse(e.order));
        entries
    };
    view! {
        <main class="history-page w-full max-w-4xl">
            <div class="history-heading"><h1>{move || i18n.text(Message::History)}</h1><A href="/" attr:class="ui-action">{move || i18n.text(Message::Back)}</A></div>
            <p>{move || i18n.text(Message::HistoryIntro)}</p>
            <section aria-label=move || i18n.text(Message::CurrentGame)>
                <h2>{move || i18n.text(Message::CurrentGame)}</h2>
                {move || history.entries.get().into_iter().find(|e| e.id==active_id.get()).map(|entry|view! {<HistoryCard entry=entry active=true selected=selected initial=initial/>})}
            </section>
            <section aria-label=move || i18n.text(Message::PinnedGames)>
                <h2>{move || i18n.text(Message::Pinned)}</h2>
                <Show when=move || records(true).is_empty()><p class="text-muted">{move || i18n.text(Message::NoPinned)}</p></Show>
                <div class="history-cards">{move || records(true).into_iter().map(|entry|view! {<HistoryCard entry=entry active=false selected=selected initial=initial/>}).collect::<Vec<_>>()}</div>
            </section>
            <section aria-label=move || i18n.text(Message::RecentGames)>
                <h2>{move || i18n.text(Message::Recent)}</h2>
                <Show when=move || records(false).is_empty()><p class="text-muted">{move || i18n.text(Message::NoRecent)}</p></Show>
                <div class="history-cards">{move || records(false).into_iter().map(|entry|view! {<HistoryCard entry=entry active=false selected=selected initial=initial/>}).collect::<Vec<_>>()}</div>
            </section>
        </main>
        <dialog id="history-load" class="share-dialog" aria-labelledby="history-load-title" on:cancel=move |_:web_sys::Event|selected.set(None)>
            <h2 id="history-load-title">{move || i18n.text(Message::OpenSaved)}</h2>
            <fieldset><legend>{move || i18n.text(Message::HowStart)}</legend>
                <label><input type="radio" name="history-mode" prop:checked=move || !initial.get() on:change=move |_|initial.set(false)/>{move || i18n.text(Message::CurrentState)}</label>
                <label><input type="radio" name="history-mode" prop:checked=move || initial.get() on:change=move |_|initial.set(true)/>{move || i18n.text(Message::InitialSudoku)}</label>
            </fieldset>
            <Show when=move ||selected.get().is_some() && history.error.get().is_some()>
                <p role="alert">{move ||i18n.notice(history.error.get())}</p>
                <button class="ui-action" disabled=move ||history.busy.get() on:click=move |_|history.refresh(state)>{move || i18n.text(Message::TryAgain)}</button>
            </Show>
            {move ||selected.get().map(|entry|{
                let game=if initial.get(){entry.game.initial_position()}else{entry.game};
                view! {<p>{details(i18n, &game)}</p><BoardPreview game=game/>}
            })}
            <p>{move ||if initial.get(){i18n.text(Message::RestartIntro)}else{i18n.text(Message::ResumeIntro)}}</p>
            <button class="ui-action" disabled=move ||history.busy.get() on:click=move |_| {
                if let Some(entry)=selected.get_untracked(){let nav=navigate.clone();history.load(state,entry,initial.get_untracked(),move |success|{if success {selected.set(None);nav("/",Default::default());}});}
            }>{move || i18n.text(Message::LoadGame)}</button>
            <button class="ui-action" on:click=move |_|selected.set(None)>{move || i18n.text(Message::Cancel)}</button>
        </dialog>
    }
}
