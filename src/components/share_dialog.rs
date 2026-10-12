use crate::i18n::{use_i18n, Message};
use crate::{sharing::SharedGame, state::AppState};
use leptos::prelude::*;

fn run(script: &str) {
    let _ = js_sys::eval(script);
}
fn clear_fragment() {
    run("history.replaceState(history.state,'',location.pathname+location.search)");
}
#[derive(Clone, Copy)]
pub struct ShareUi(pub RwSignal<bool>);

#[component]
pub fn ShareDialogs(state: AppState) -> impl IntoView {
    let i18n = use_i18n();
    let history = expect_context::<crate::state::GameHistory>();
    let sharing = expect_context::<ShareUi>().0;
    let initial = RwSignal::new(false);
    let url = RwSignal::new(String::new());
    let error = RwSignal::new(None::<Message>);
    let generation_error = RwSignal::new(None::<Message>);
    let incoming = RwSignal::new(None::<SharedGame>);
    let import_initial = RwSignal::new(false);
    let preview = move || {
        incoming.get().map(|game| {
            if import_initial.get() {
                game.initial_position()
            } else {
                game
            }
        })
    };
    let inspect = move || {
        let hash = js_sys::eval("location.hash")
            .ok()
            .and_then(|v| v.as_string())
            .unwrap_or_default();
        if let Some(payload) = hash.strip_prefix("#sudoku=") {
            import_initial.set(false);
            match SharedGame::decode(payload) {
                Ok(game) => {
                    incoming.set(Some(game));
                    error.set(None);
                }
                Err(message) => {
                    incoming.set(None);
                    error.set(Some(message));
                }
            }
            run("document.getElementById('import-dialog').showModal()");
        }
    };
    // Deferred until the dialogs are mounted; also handles in-app hash navigation.
    set_timeout(inspect, std::time::Duration::ZERO);
    window_event_listener(leptos::ev::hashchange, move |_| inspect());
    let generate = move |initial_mode: bool| {
        run("window.__sudokuI18n.shareStatus('')");
        match SharedGame::capture(&state.0.get_untracked(), initial_mode).link() {
            Ok(link) => {
                url.set(link);
                generation_error.set(None);
            }
            Err(message) => {
                url.set(String::new());
                generation_error.set(Some(message));
            }
        }
    };
    Effect::new(move || {
        if sharing.get() {
            initial.set(false);
            generate(false);
            run("document.getElementById('share-dialog').showModal()");
        }
    });
    let close_share = move || {
        sharing.set(false);
        run("document.getElementById('share-dialog').close();document.getElementById('share-trigger')?.focus()");
    };
    let close_import = move || {
        incoming.set(None);
        clear_fragment();
        run("document.getElementById('import-dialog').close()");
    };
    view! {
        <dialog id="share-dialog" class="share-dialog" aria-labelledby="share-title" on:cancel=move |_: web_sys::Event| sharing.set(false)>
            <h2 id="share-title">{move || i18n.text(Message::ShareSudoku)}</h2>
            <p>{move || i18n.text(Message::ShareIntro)}</p>
            <label><input type="radio" name="share-mode" prop:checked=move || !initial.get() on:change=move |_| {initial.set(false); generate(false);}/>{move || i18n.text(Message::CurrentState)}</label>
            <label><input type="radio" name="share-mode" prop:checked=move || initial.get() on:change=move |_| {initial.set(true); generate(true);}/>{move || i18n.text(Message::InitialSudoku)}</label>
            <Show when=move || !url.get().is_empty()>
                <label for="share-url">{move || i18n.text(Message::GeneratedLink)}</label>
                <textarea id="share-url" readonly prop:value=move || url.get() rows="4"></textarea>
                <button class="ui-action" on:click=move |_| run("window.__sudoku.copyShare()")>{move || i18n.text(Message::CopyLink)}</button>
                <Show when=move || js_sys::eval("typeof navigator.share==='function'").ok().and_then(|v|v.as_bool()).unwrap_or(false)>
                    <button class="ui-action" on:click=move |_| run("window.__sudoku.sendShare()")>{move || i18n.text(Message::ShareLink)}</button>
                </Show>
            </Show>
            <p role="alert">{move || generation_error.get().map(|message| i18n.text(message)).unwrap_or_default()}</p>
            <p id="share-status" role="status"></p>
            <button class="ui-action" on:click=move |_| close_share()>{move || i18n.text(Message::Close)}</button>
        </dialog>
        <dialog id="import-dialog" class="share-dialog" aria-labelledby="import-title" on:cancel=move |_: web_sys::Event| {incoming.set(None); clear_fragment();}>
            <h2 id="import-title">{move || i18n.text(Message::SharedSudoku)}</h2>
            <p role="alert">{move || error.get().map(|message| i18n.text(message)).unwrap_or_default()}</p>
            <Show when=move || incoming.get().is_some()>
                <fieldset>
                    <legend>{move || i18n.text(Message::HowStart)}</legend>
                    <label><input type="radio" name="import-mode" prop:checked=move || !import_initial.get() on:change=move |_| import_initial.set(false)/>{move || i18n.text(Message::CurrentState)}</label>
                    <label><input type="radio" name="import-mode" prop:checked=move || import_initial.get() on:change=move |_| import_initial.set(true)/>{move || i18n.text(Message::InitialSudoku)}</label>
                </fieldset>
            </Show>
            <Show when=move || incoming.get().is_some() && history.error.get().is_some()>
                <p role="alert">{move ||i18n.notice(history.error.get())}</p>
                <button class="ui-action" disabled=move ||history.busy.get() on:click=move |_|history.refresh(state)>{move || i18n.text(Message::TryAgain)}</button>
            </Show>
            {move || preview().map(|game| {
                let difficulty=i18n.difficulty(game.difficulty);
                view! {
                    <p>{i18n.format(Message::ImportSummary, &[("difficulty", difficulty.into()), ("minutes", (game.seconds/60).to_string()), ("seconds", (game.seconds%60).to_string()), ("errors", game.errors.to_string()), ("completed", if game.board==game.solution { i18n.text(Message::CompletedSuffix).into() } else { String::new() })])}</p>
                    <crate::components::board_preview::BoardPreview game=game/>
                    <p>{move || i18n.text(Message::ImportIntro)}</p>
                    <button class="ui-action" disabled=move || history.busy.get() || !history.ready.get() on:click=move |_| {if let Some(game)=incoming.get_untracked() {let selected=if import_initial.get_untracked() {game.initial_position()} else {game}; history.import(state,selected,move |success|if success {close_import();});}}>{move || i18n.text(Message::LoadGame)}</button>
                }
            })}
            <button class="ui-action" on:click=move |_| close_import()>{move || i18n.text(Message::Cancel)}</button>
        </dialog>
    }
}
