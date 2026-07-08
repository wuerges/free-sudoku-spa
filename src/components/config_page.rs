use crate::state::AppState;
use leptos::prelude::*;
use leptos_router::components::A;

use crate::state::GameState;

struct Toggle {
    icon: &'static str,
    label: &'static str,
    desc: &'static str,
    get: fn(&GameState) -> bool,
    toggle: fn(&AppState),
}

#[component]
pub fn ConfigPage() -> impl IntoView {
    let state: AppState = use_context().unwrap();

    let toggles: Vec<Toggle> = vec![
        Toggle {
            icon: "↩↪",
            label: "Desfazer / Refazer",
            desc: "Mostra ou esconde os botões de desfazer e refazer.",
            get: |s| s.undo_enabled,
            toggle: |s| s.toggle_undo(),
        },
        Toggle {
            icon: "📝",
            label: "Auto Notas",
            desc: "Mostra ou esconde o botão de preencher notas automaticamente.",
            get: |s| s.auto_notes_enabled,
            toggle: |s| s.toggle_auto_notes(),
        },
        Toggle {
            icon: "💡",
            label: "Dica",
            desc: "Mostra ou esconde o botão de dica.",
            get: |s| s.hint_enabled,
            toggle: |s| s.toggle_hint(),
        },
        Toggle {
            icon: "🀄",
            label: "Efeito Dominó",
            desc: "Após acertar um número, abre automaticamente células com apenas um candidato. O primeiro após 400ms, cada vez mais rápido.",
            get: |s| s.domino_enabled,
            toggle: |s| s.toggle_domino(),
        },
    ];

    view! {
        <div class="w-full max-w-[min(90vw,500px)] mx-auto pb-8">
            <A
                href="/"
                attr:class="flex items-center gap-0.5 text-blue-500 dark:text-blue-400 active:opacity-60 select-none py-2.5 px-1 -ml-1 rounded-lg transition-opacity no-underline"
            >
                <span class="text-xl leading-none">"‹"</span>
                <span class="text-[17px] font-normal">"Voltar"</span>
            </A>

            <h1 class="text-3xl font-bold tracking-wider text-center mt-4 mb-10">"CONFIGURAÇÕES"</h1>

            <div class="flex flex-col">
                {toggles.into_iter().map(|t| {
                    let state = state;
                    view! {
                        <div class="grid gap-2" style="grid-template-columns: 80px 1fr 80px; padding: 1.25rem 0; border-bottom: 0.5px solid #9ca3af;">
                            <span class="text-lg text-center" style="width: 80px; line-height: 1.25;">{t.icon}</span>
                            <div>
                                <strong class="text-sm font-semibold">{t.label}</strong>
                                <p class="text-xs text-gray-500 dark:text-gray-400" style="margin-top: 2px; line-height: 1.5;">{t.desc}</p>
                            </div>
                            <label class="flex items-center justify-center cursor-pointer self-stretch">
                                <input type="checkbox" prop:checked=move || (t.get)(&state.0.get()) on:change=move |_| (t.toggle)(&state) class="accent-blue-500 cursor-pointer" style="transform: scale(2);" />
                            </label>
                        </div>
                    }
                }).collect::<Vec<_>>()}
            </div>
        </div>
    }
}
