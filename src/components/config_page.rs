use crate::state::{AppState, SoundType};
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
                attr:class="flex items-center gap-0.5 text-accent active:bg-control select-none py-2.5 px-1 -ml-1 rounded-lg transition-opacity no-underline"
            >
                <span class="text-xl leading-none">"‹"</span>
                <span class="text-[17px] font-normal">"Voltar"</span>
            </A>

            <h1 class="text-3xl font-bold tracking-wider text-center mt-4 mb-10">"CONFIGURAÇÕES"</h1>

            <div class="flex flex-col">
                {toggles.into_iter().map(|t| {
                    view! {
                        <div class="grid gap-2" style="grid-template-columns: 80px 1fr 80px; padding: 1.25rem 0; border-bottom: 0.5px solid var(--ui-grid-thin);">
                            <span class="text-lg text-center" style="width: 80px; line-height: 1.25;">{t.icon}</span>
                            <div>
                                <strong class="text-sm font-semibold">{t.label}</strong>
                                <p class="text-xs text-muted" style="margin-top: 2px; line-height: 1.5;">{t.desc}</p>
                            </div>
                            <label class="flex items-center justify-center cursor-pointer self-stretch">
                                <input type="checkbox" prop:checked=move || (t.get)(&state.0.get()) on:change=move |_| (t.toggle)(&state) class="accent-accent cursor-pointer" style="transform: scale(2);" />
                            </label>
                        </div>
                    }
                }).collect::<Vec<_>>()}

                // Sound selector
                <div class="grid gap-2" style="grid-template-columns: 80px 1fr 80px; padding: 1.25rem 0; border-bottom: 0.5px solid var(--ui-grid-thin);">
                    <span class="text-lg text-center" style="width: 80px; line-height: 1.25;">"🔊"</span>
                    <div>
                        <strong class="text-sm font-semibold">"Som"</strong>
                        <p class="text-xs text-muted" style="margin-top: 2px; line-height: 1.5;">"Som ao acertar uma célula."</p>
                    </div>
                    <button
                        on:click=move |_| state.cycle_sound()
                        class="flex items-center justify-center text-sm font-medium rounded bg-control active:bg-control-hover transition-colors cursor-pointer"
                        style="padding: 4px 8px;"
                    >
                        {move || match state.0.get().sound_type {
                            SoundType::Beep => "Beep",
                            SoundType::Explosion => "💥",
                            SoundType::None => "Off",
                        }}
                    </button>
                </div>

                <section class="py-5 border-b border-grid-thin space-y-3" aria-labelledby="highlight-settings">
                    <h2 id="highlight-settings" class="text-sm font-semibold">"Destaques do tabuleiro"</h2>
                    {[(false, "Sombreamento pela seleção"), (true, "Sombreamento pelos iguais")].into_iter().map(|(matching, label)| {
                        let id = if matching { "matching-shading" } else { "selected-shading" };
                        view! {
                            <div>
                                <label for=id class="flex justify-between gap-2 text-sm">
                                    <span>{label}</span>
                                    <span>{move || { let h = state.0.get().highlights; format!("{}%", if matching { h.matching_shading } else { h.selected_shading }) }}</span>
                                </label>
                                <input id=id type="range" min="0" max="100" step="1" class="w-full h-12 accent-accent cursor-pointer"
                                    prop:value=move || { let h = state.0.get().highlights; (if matching { h.matching_shading } else { h.selected_shading }).to_string() }
                                    on:input=move |ev| { if let Ok(value) = event_target_value(&ev).parse::<u8>() { state.0.update(|s| { if matching { s.highlights.matching_shading = value.min(100); } else { s.highlights.selected_shading = value.min(100); } }); } }
                                />
                            </div>
                        }
                    }).collect::<Vec<_>>()}
                    <label class="flex items-center justify-between gap-3 min-h-12 text-sm cursor-pointer">
                        <span>"Pontos nas células disponíveis"</span>
                        <input type="checkbox" class="accent-accent w-5 h-5" prop:checked=move || state.0.get().highlights.dots
                            on:change=move |ev| state.0.update(|s| s.highlights.dots = event_target_checked(&ev)) />
                    </label>
                    <label class="flex items-center justify-between gap-3 min-h-12 text-sm cursor-pointer">
                        <span>"Listras nas células bloqueadas"</span>
                        <input type="checkbox" class="accent-accent w-5 h-5" prop:checked=move || state.0.get().highlights.stripes
                            on:change=move |ev| state.0.update(|s| s.highlights.stripes = event_target_checked(&ev)) />
                    </label>
                </section>

                // Reset button
                <div class="flex justify-center" style="margin-top: 2rem;">
                    <button
                        on:click=move |_| state.reset_config()
                        style="padding: 1.25rem 1rem;" class="rounded-lg text-sm font-medium bg-error text-error-text active:bg-error transition-colors cursor-pointer"
                    >
                        "↺ Resetar Configurações"
                    </button>
                </div>
            </div>
        </div>
    }
}
