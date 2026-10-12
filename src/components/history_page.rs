use crate::{
    components::board_preview::BoardPreview,
    game_history::{evictions, HistoryEntry},
    sharing::SharedGame,
    state::{AppState, GameHistory},
    sudoku_engine::Difficulty,
};
use leptos::prelude::*;
use leptos_router::{components::A, hooks::use_navigate};

fn details(game: &SharedGame) -> String {
    let label = match game.difficulty {
        Difficulty::Easy => "Fácil",
        Difficulty::Medium => "Médio",
        Difficulty::Hard => "Difícil",
        Difficulty::Expert => "Expert",
        Difficulty::Master => "Mestre",
    };
    format!(
        "{label} · {}min {}s · {} erros · {}",
        game.seconds / 60,
        game.seconds % 60,
        game.errors,
        if game.board == game.solution {
            "Concluído"
        } else {
            "Em andamento"
        }
    )
}
fn played_date(ms: f64) -> String {
    let date = js_sys::Date::new(&wasm_bindgen::JsValue::from_f64(ms));
    format!(
        "{:02}/{:02}/{} às {:02}:{:02}",
        date.get_date(),
        date.get_month() + 1,
        date.get_full_year(),
        date.get_hours(),
        date.get_minutes()
    )
}

#[component]
pub fn HistoryFeedback(state: AppState, history: GameHistory) -> impl IntoView {
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
            <h2 id="saving-title">"Salvando histórico…"</h2><p>"Aguarde um instante."</p>
        </dialog>
        <Show when=move || !history.error.get().is_empty()>
            <div class="history-error" role="alert">
                <p>{move || history.error.get()}</p>
                <button class="ui-action" disabled=move || history.busy.get() on:click=move |_|history.refresh(state)>"Tentar novamente"</button>
                <button class="ui-action" on:click=move |_|history.error.set(String::new())>"Fechar aviso"</button>
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
            if evictions(&entries,&state.0.get_untracked().game_id).contains(&value.id)
                && !web_sys::window().and_then(|w|w.confirm_with_message("Ao desafixar, este jogo ficará fora dos 10 mais recentes e será removido do histórico. Desafixar?").ok()).unwrap_or(false) {return;}
        }
        history.set_pinned(state, value.id, !value.pinned);
    };
    view! {
        <article class="history-card" data-game-id=entry.get_value().id data-pinned=entry.get_value().pinned.to_string()>
            <div>{move || view! {<BoardPreview game=game() miniature=true/>}}</div>
            <div class="history-card-info">
                <h3>{if active {"Jogo atual"}else if entry.get_value().pinned {"Jogo fixado"}else{"Jogo salvo"}}</h3>
                <p>{move || details(&game())}</p>
                <p class="text-muted text-sm">{format!("Última partida: {}",played_date(entry.get_value().last_played_ms))}</p>
                <div class="history-card-actions">
                    <Show when=move || !active>
                        <button class="ui-action" on:click=move |_| {initial.set(false);selected.set(Some(entry.get_value()));}>"Abrir jogo"</button>
                    </Show>
                    <button id=format!("pin-{}",entry.get_value().id) class="ui-action" aria-pressed=entry.get_value().pinned.to_string()
                        on:click=pin>{if entry.get_value().pinned {"Desafixar"}else{"Fixar"}}</button>
                </div>
            </div>
        </article>
    }
}

#[component]
pub fn HistoryPage() -> impl IntoView {
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
            <div class="history-heading"><h1>"Histórico"</h1><A href="/" attr:class="ui-action">"Voltar"</A></div>
            <p>"Seu jogo atual, os 10 jogos anteriores mais recentes e todos os jogos fixados ficam salvos neste aparelho."</p>
            <section aria-label="Jogo atual">
                <h2>"Jogo atual"</h2>
                {move || history.entries.get().into_iter().find(|e| e.id==active_id.get()).map(|entry|view! {<HistoryCard entry=entry active=true selected=selected initial=initial/>})}
            </section>
            <section aria-label="Jogos fixados">
                <h2>"Fixados"</h2>
                <Show when=move || records(true).is_empty()><p class="text-muted">"Nenhum jogo fixado."</p></Show>
                <div class="history-cards">{move || records(true).into_iter().map(|entry|view! {<HistoryCard entry=entry active=false selected=selected initial=initial/>}).collect::<Vec<_>>()}</div>
            </section>
            <section aria-label="Jogos recentes">
                <h2>"Recentes"</h2>
                <Show when=move || records(false).is_empty()><p class="text-muted">"Seus jogos anteriores aparecerão aqui."</p></Show>
                <div class="history-cards">{move || records(false).into_iter().map(|entry|view! {<HistoryCard entry=entry active=false selected=selected initial=initial/>}).collect::<Vec<_>>()}</div>
            </section>
        </main>
        <dialog id="history-load" class="share-dialog" aria-labelledby="history-load-title" on:cancel=move |_:web_sys::Event|selected.set(None)>
            <h2 id="history-load-title">"Abrir jogo salvo"</h2>
            <fieldset><legend>"Como deseja começar?"</legend>
                <label><input type="radio" name="history-mode" prop:checked=move || !initial.get() on:change=move |_|initial.set(false)/>" Estado atual"</label>
                <label><input type="radio" name="history-mode" prop:checked=move || initial.get() on:change=move |_|initial.set(true)/>" Sudoku inicial"</label>
            </fieldset>
            <Show when=move ||selected.get().is_some() && !history.error.get().is_empty()>
                <p role="alert">{move ||history.error.get()}</p>
                <button class="ui-action" disabled=move ||history.busy.get() on:click=move |_|history.refresh(state)>"Tentar novamente"</button>
            </Show>
            {move ||selected.get().map(|entry|{
                let game=if initial.get(){entry.game.initial_position()}else{entry.game};
                view! {<p>{details(&game)}</p><BoardPreview game=game/>}
            })}
            <p>{move ||if initial.get(){"Começar do início cria outra partida e mantém o progresso anterior no histórico."}else{"Seu jogo atual será guardado no histórico. O jogo salvo começa pausado, com suas configurações atuais."}}</p>
            <button class="ui-action" disabled=move ||history.busy.get() on:click=move |_| {
                if let Some(entry)=selected.get_untracked(){let nav=navigate.clone();history.load(state,entry,initial.get_untracked(),move |success|{if success {selected.set(None);nav("/",Default::default());}});}
            }>"Carregar jogo"</button>
            <button class="ui-action" on:click=move |_|selected.set(None)>"Cancelar"</button>
        </dialog>
    }
}
