use crate::components::{
    game_controls::GameStatus,
    icon::{Icon, IconName},
};
use crate::i18n::{use_i18n, Message};
use crate::state::AppState;
use leptos::prelude::*;
use leptos_router::components::A;
use wasm_bindgen::JsCast;

#[component]
pub fn Header() -> impl IntoView {
    let i18n = use_i18n();
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
        let label = i18n.difficulty(s.difficulty);
        let technique = s
            .rating
            .as_ref()
            .map(|rating| i18n.technique(rating.strongest))
            .unwrap_or_else(|| i18n.text(Message::PreviousGame));
        format!("{label} · {technique}")
    };

    view! {
        <header class="game-header">
            <div class="header-main">
                <GameStatus state=state />
                <nav class="header-tools" aria-label=move || i18n.text(Message::GameOptions)>
                    <A href="/config" attr:aria-label=move || i18n.text(Message::Settings) attr:title=move || i18n.text(Message::Settings) attr:class="ui-icon-button">
                        <Icon name=IconName::Settings />
                    </A>
                    <A href="/help" attr:aria-label=move || i18n.text(Message::Help) attr:title=move || i18n.text(Message::Help) attr:class="ui-icon-button">
                        <Icon name=IconName::Help />
                    </A>
                    <button class="ui-icon-button" aria-label=move || if dark_mode.get() { i18n.text(Message::LightTheme) } else { i18n.text(Message::DarkTheme) }
                        title=move || if dark_mode.get() { i18n.text(Message::LightTheme) } else { i18n.text(Message::DarkTheme) }
                        on:click=move |_| dark_mode.update(|d| *d = !*d)>
                        {move || view! { <Icon name=if dark_mode.get() { IconName::Sun } else { IconName::Moon } /> }}
                    </button>
                    <Show when=move || install_visible.get()>
                        <button class="ui-icon-button" aria-label=move || i18n.text(Message::InstallApp) title=move || i18n.text(Message::InstallApp) on:click=move |_| {
                            let _ = js_sys::eval("window.__sudoku&&window.__sudoku.showInstall()");
                            install_visible.set(false);
                        }><Icon name=IconName::Install /></button>
                    </Show>
                </nav>
            </div>
            <div class="header-caption">
                <h1>"Sudoku"</h1><p>{difficulty_label}</p>
            </div>
        </header>
    }
}
