use crate::state::AppState;
use crate::sudoku_engine::Difficulty;
use leptos::prelude::*;
use wasm_bindgen::JsCast;

#[component]
pub fn Header(state: AppState, dark_mode: RwSignal<bool>) -> impl IntoView {
    let install_visible = RwSignal::new(false);

    // ponytail: beforeinstallprompt captured in index.html JS, stored at window.__sudoku.
    // Check on mount if already available.
    if let Some(_window) = web_sys::window() {
        let already = js_sys::eval("!!(window.__sudoku&&window.__sudoku._installPrompt)")
            .map(|v| v.as_bool().unwrap_or(false))
            .unwrap_or(false);
        install_visible.set(already);

        let iv = install_visible;
        let closure = wasm_bindgen::closure::Closure::wrap(Box::new(move || {
            iv.set(true);
        }) as Box<dyn FnMut()>);
        let _ = web_sys::window()
            .unwrap()
            .add_event_listener_with_callback("sudoku:installavailable", closure.as_ref().unchecked_ref());
        closure.forget();
    }

    let difficulty_label = move || match state.0.get().difficulty {
        Difficulty::Easy => "Fácil",
        Difficulty::Medium => "Médio",
        Difficulty::Hard => "Difícil",
        Difficulty::Expert => "Expert",
    };

    view! {
        <header class="w-full max-w-[min(90vw,500px)] mx-auto flex items-center justify-between pt-2">
            <div>
                <h1 class="text-lg sm:text-xl font-bold">"Sudoku"</h1>
                <p class="text-xs text-gray-500 dark:text-gray-400">{difficulty_label}</p>
            </div>
            <div class="flex items-center gap-2">
                <button
                    class="text-2xl active:opacity-70 select-none"
                    on:click=move |_| dark_mode.update(|d| *d = !*d)
                >
                    {move || if dark_mode.get() { "☀️" } else { "🌙" }}
                </button>
                {move || install_visible.get().then(|| view! {
                    <button
                        class="px-3 py-1 rounded-lg text-sm font-medium bg-green-500 text-white active:bg-green-600 select-none"
                        on:click=move |_| {
                            let _ = js_sys::eval("window.__sudoku&&window.__sudoku.showInstall()");
                            install_visible.set(false);
                        }
                    >
                        "📲 Instalar"
                    </button>
                })}
            </div>
        </header>
    }
}
