use crate::components::icon::{Icon, IconName};
use crate::state::AppState;
use crate::sudoku_engine::Difficulty;
use leptos::prelude::*;
use leptos_router::components::A;
use wasm_bindgen::JsCast;

#[component]
pub fn Header() -> impl IntoView {
    let state: AppState = use_context().unwrap();
    let dark_mode: RwSignal<bool> = use_context().unwrap();
    let install_visible = RwSignal::new(false);

    // ponytail: beforeinstallprompt via window.__sudoku JS bridge. ceiling: no typed API. upgrade: web-sys BeforeInstallPromptEvent if the feature lands.
    if let Some(_window) = web_sys::window() {
        let already = js_sys::eval("!!(window.__sudoku&&window.__sudoku._installPrompt)")
            .map(|v| v.as_bool().unwrap_or(false))
            .unwrap_or(false);
        install_visible.set(already);

        let iv = install_visible;
        let closure = wasm_bindgen::closure::Closure::wrap(Box::new(move || {
            iv.set(true);
        }) as Box<dyn FnMut()>);
        let _ = web_sys::window().unwrap().add_event_listener_with_callback(
            "sudoku:installavailable",
            closure.as_ref().unchecked_ref(),
        );
        closure.forget();
    }

    let difficulty_label = move || {
        let s = state.0.get();
        let label = match s.difficulty {
            Difficulty::Easy => "Fácil",
            Difficulty::Medium => "Médio",
            Difficulty::Hard => "Difícil",
            Difficulty::Expert => "Expert",
            Difficulty::Master => "Mestre",
        };
        match &s.rating {
            Some(rating) => format!("{label} · {}", rating.strongest.label()),
            None => format!("{label} · jogo anterior"),
        }
    };

    view! {
        <header class="game-header">
            <div class="header-main">
                <div class="min-w-0 pr-2">
                    <h1 class="text-xl font-bold">"Sudoku"</h1>
                    <p class="text-xs text-muted leading-relaxed">{difficulty_label}</p>
                </div>
                <nav class="header-tools" aria-label="Opções do jogo">
                    <A href="/config" attr:aria-label="Configurações" attr:title="Configurações" attr:class="ui-icon-button">
                        <Icon name=IconName::Settings />
                    </A>
                    <A href="/help" attr:aria-label="Ajuda" attr:title="Ajuda" attr:class="ui-icon-button">
                        <Icon name=IconName::Help />
                    </A>
                    <button class="ui-icon-button" aria-label=move || if dark_mode.get() { "Ativar tema claro" } else { "Ativar tema escuro" }
                        title=move || if dark_mode.get() { "Ativar tema claro" } else { "Ativar tema escuro" }
                        on:click=move |_| dark_mode.update(|d| *d = !*d)>
                        {move || view! { <Icon name=if dark_mode.get() { IconName::Sun } else { IconName::Moon } /> }}
                    </button>
                </nav>
            </div>
            <Show when=move || install_visible.get()>
                <div class="install-row">
                    <button class="ui-action" on:click=move |_| {
                        let _ = js_sys::eval("window.__sudoku&&window.__sudoku.showInstall()");
                        install_visible.set(false);
                    }><Icon name=IconName::Install />"Instalar aplicativo"</button>
                </div>
            </Show>
        </header>
    }
}
