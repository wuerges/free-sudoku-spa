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
    let history = expect_context::<crate::state::GameHistory>();
    let sharing = expect_context::<ShareUi>().0;
    let initial = RwSignal::new(false);
    let url = RwSignal::new(String::new());
    let error = RwSignal::new(String::new());
    let generation_error = RwSignal::new(String::new());
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
                    error.set(String::new());
                }
                Err(message) => {
                    incoming.set(None);
                    error.set(message.into());
                }
            }
            run("document.getElementById('import-dialog').showModal()");
        }
    };
    // Deferred until the dialogs are mounted; also handles in-app hash navigation.
    set_timeout(inspect, std::time::Duration::ZERO);
    window_event_listener(leptos::ev::hashchange, move |_| inspect());
    let generate = move |initial_mode: bool| {
        run("document.getElementById('share-status').textContent=''");
        match SharedGame::capture(&state.0.get_untracked(), initial_mode).link() {
            Ok(link) => {
                url.set(link);
                generation_error.set(String::new());
            }
            Err(message) => {
                url.set(String::new());
                generation_error.set(message.into());
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
            <h2 id="share-title">"Compartilhar Sudoku"</h2>
            <p>"O link inclui o Sudoku e sua solução, sem suas configurações."</p>
            <label><input type="radio" name="share-mode" prop:checked=move || !initial.get() on:change=move |_| {initial.set(false); generate(false);}/>" Estado atual"</label>
            <label><input type="radio" name="share-mode" prop:checked=move || initial.get() on:change=move |_| {initial.set(true); generate(true);}/>" Sudoku inicial"</label>
            <Show when=move || !url.get().is_empty()>
                <label for="share-url">"Link gerado"</label>
                <textarea id="share-url" readonly prop:value=move || url.get() rows="4"></textarea>
                <button class="ui-action" on:click=move |_| run("window.__sudoku.copyShare()")>"Copiar link"</button>
                <Show when=move || js_sys::eval("typeof navigator.share==='function'").ok().and_then(|v|v.as_bool()).unwrap_or(false)>
                    <button class="ui-action" on:click=move |_| run("window.__sudoku.sendShare()")>"Compartilhar link"</button>
                </Show>
            </Show>
            <p role="alert">{move || generation_error.get()}</p>
            <p id="share-status" role="status"></p>
            <button class="ui-action" on:click=move |_| close_share()>"Fechar"</button>
        </dialog>
        <dialog id="import-dialog" class="share-dialog" aria-labelledby="import-title" on:cancel=move |_: web_sys::Event| {incoming.set(None); clear_fragment();}>
            <h2 id="import-title">"Sudoku compartilhado"</h2>
            <p role="alert">{move || error.get()}</p>
            <Show when=move || incoming.get().is_some()>
                <fieldset>
                    <legend>"Como deseja começar?"</legend>
                    <label><input type="radio" name="import-mode" prop:checked=move || !import_initial.get() on:change=move |_| import_initial.set(false)/>" Estado atual"</label>
                    <label><input type="radio" name="import-mode" prop:checked=move || import_initial.get() on:change=move |_| import_initial.set(true)/>" Sudoku inicial"</label>
                </fieldset>
            </Show>
            <Show when=move || incoming.get().is_some() && !history.error.get().is_empty()>
                <p role="alert">{move ||history.error.get()}</p>
                <button class="ui-action" disabled=move ||history.busy.get() on:click=move |_|history.refresh(state)>"Tentar novamente"</button>
            </Show>
            {move || preview().map(|game| {
                let difficulty=match game.difficulty {crate::sudoku_engine::Difficulty::Easy=>"Fácil",crate::sudoku_engine::Difficulty::Medium=>"Médio",crate::sudoku_engine::Difficulty::Hard=>"Difícil",crate::sudoku_engine::Difficulty::Expert=>"Expert",crate::sudoku_engine::Difficulty::Master=>"Mestre"};
                view! {
                    <p>{format!("{} · Tempo: {}min {}s · Erros: {}{}",difficulty,game.seconds/60,game.seconds%60,game.errors,if game.board==game.solution {" · Concluído"}else{""})}</p>
                    <crate::components::board_preview::BoardPreview game=game/>
                    <p>"Seu jogo atual será guardado no histórico. Suas configurações serão mantidas. O jogo começa pausado."</p>
                    <button class="ui-action" disabled=move || history.busy.get() || !history.ready.get() on:click=move |_| {if let Some(game)=incoming.get_untracked() {let selected=if import_initial.get_untracked() {game.initial_position()} else {game}; history.import(state,selected,move |success|if success {close_import();});}}>"Carregar jogo"</button>
                }
            })}
            <button class="ui-action" on:click=move |_| close_import()>"Cancelar"</button>
        </dialog>
    }
}
