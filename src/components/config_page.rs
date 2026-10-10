use crate::components::icon::{Icon, IconName};
use crate::state::{AppState, GameState, SoundType};
use leptos::prelude::*;
use leptos_router::components::A;

#[component]
fn PreferenceToggle(
    id: &'static str,
    label: &'static str,
    description: &'static str,
    icon: IconName,
    get: fn(&GameState) -> bool,
    toggle: fn(&AppState),
) -> impl IntoView {
    let state: AppState = use_context().unwrap();
    let description_id = format!("{id}-description");
    view! {
        <label for=id class="preference-row">
            <Icon name=icon />
            <span class="min-w-0">
                <span class="preference-label">{label}</span>
                <span id=description_id.clone() class="preference-description">{description}</span>
            </span>
            <input id=id type="checkbox" aria-label=label aria-describedby=description_id
                class="preference-checkbox" prop:checked=move || get(&state.0.get())
                on:change=move |_| toggle(&state) />
        </label>
    }
}

#[component]
fn DominoSlider(
    target: u8,
    id: &'static str,
    label: &'static str,
    description: &'static str,
) -> impl IntoView {
    let state: AppState = use_context().unwrap();
    let description_id = format!("{id}-description");
    let value = move || {
        let d = state.0.get().domino;
        match target {
            0 => d.initial_delay_ms,
            1 => d.acceleration_percent,
            2 => d.minimum_delay_ms,
            _ => d.empty_cell_threshold,
        }
    };
    let formatted = move || match target {
        0 | 2 => format!("{} ms", value()),
        1 => format!("{}%", value()),
        _ if value() == 0 => "Sem limite".to_string(),
        _ => format!("{} células", value()),
    };
    view! {
        <div class="preference-slider">
            <label for=id class="slider-label"><span>{label}</span><output for=id>{formatted}</output></label>
            <input id=id type="range" aria-describedby=description_id.clone() aria-valuetext=formatted
                min=match target { 0 => 100, 2 => 50, _ => 0 }
                max=move || match target { 0 => 2000, 1 => 50, 2 => state.0.get().domino.initial_delay_ms, _ => 81 }
                step=match target { 0 | 2 => 50, _ => 1 } prop:value=move || value().to_string()
                on:input=move |ev| {
                    if let Ok(value) = event_target_value(&ev).parse::<u32>() {
                        let mut d = state.0.get_untracked().domino;
                        match target { 0 => d.initial_delay_ms = value, 1 => d.acceleration_percent = value,
                            2 => d.minimum_delay_ms = value, _ => d.empty_cell_threshold = value }
                        state.set_domino_settings(d);
                    }
                } />
            <p id=description_id class="preference-description">{description}</p>
        </div>
    }
}

#[component]
pub fn ConfigPage() -> impl IntoView {
    let state: AppState = use_context().unwrap();
    view! {
        <main class="settings-page">
            <A href="/" attr:class="ui-back"><Icon name=IconName::Back /><span>"Voltar"</span></A>
            <h1 class="page-title">"Configurações"</h1>
            <p class="page-description">"Deixe o jogo do seu jeito. As alterações são salvas automaticamente."</p>

            <section class="settings-section" aria-labelledby="highlight-settings">
                <h2 id="highlight-settings">"Destaques do tabuleiro"</h2>
                <p class="section-description">"Ajuste a intensidade das cores e os indicadores nas células."</p>
                {[(0, "Sombreamento pela seleção", "selected-shading"),
                  (1, "Sombreamento pelos iguais", "matching-shading"),
                  (2, "Sombreamento das células disponíveis", "available-shading")].into_iter().map(|(target, label, id)| {
                    let value = move || { let h = state.0.get().highlights;
                        match target { 0 => h.selected_shading, 1 => h.matching_shading, _ => h.available_shading } };
                    view! {
                        <div class="preference-slider">
                            <label for=id class="slider-label"><span>{label}</span><output for=id>{move || format!("{}%", value())}</output></label>
                            <input id=id type="range" min="0" max="100" step="1"
                                aria-valuetext=move || format!("{}%", value()) prop:value=move || value().to_string()
                                on:input=move |ev| { if let Ok(value) = event_target_value(&ev).parse::<u8>() {
                                    state.0.update(|s| match target { 0 => s.highlights.selected_shading = value.min(100),
                                        1 => s.highlights.matching_shading = value.min(100), _ => s.highlights.available_shading = value.min(100) });
                                } } />
                        </div>
                    }
                }).collect::<Vec<_>>()}
                <label class="simple-preference-row"><span>"Pontos nas células disponíveis"</span>
                    <input type="checkbox" class="preference-checkbox" prop:checked=move || state.0.get().highlights.dots
                        on:change=move |ev| state.0.update(|s| s.highlights.dots = event_target_checked(&ev)) />
                </label>
                <label class="simple-preference-row"><span>"Listras nas células bloqueadas"</span>
                    <input type="checkbox" class="preference-checkbox" prop:checked=move || state.0.get().highlights.stripes
                        on:change=move |ev| state.0.update(|s| s.highlights.stripes = event_target_checked(&ev)) />
                </label>
            </section>

            <section class="settings-section" aria-labelledby="completed-settings">
                <h2 id="completed-settings">"Números concluídos"</h2>
                <p id="completed-contrast-description" class="section-description">"Destaque no teclado os números com nove ocorrências corretas. A marca de conclusão permanece visível; no modo Drop com Notas, eles continuam disponíveis."</p>
                <div class="preference-slider">
                    <label for="completed-contrast" class="slider-label"><span>"Contraste dos números concluídos"</span>
                        <output for="completed-contrast">{move || format!("{}%", state.0.get().highlights.completed_contrast)}</output>
                    </label>
                    <input id="completed-contrast" type="range" min="0" max="100" step="1"
                        aria-describedby="completed-contrast-description"
                        aria-valuetext=move || format!("{}%", state.0.get().highlights.completed_contrast)
                        prop:value=move || state.0.get().highlights.completed_contrast.to_string()
                        on:input=move |ev| { if let Ok(value) = event_target_value(&ev).parse::<u8>() {
                            state.0.update(|s| s.highlights.completed_contrast = value.min(100));
                        } } />
                </div>
            </section>

            <section class="settings-section" aria-labelledby="assistance-settings">
                <h2 id="assistance-settings">"Assistências"</h2>
                <p class="section-description">"Personalize os controles e as assistências durante o jogo."</p>
                <PreferenceToggle id="undo-enabled" label="Desfazer / Refazer"
                    description="Mostrar os botões para voltar ou repetir uma jogada." icon=IconName::Undo
                    get=|s| s.undo_enabled toggle=|s| s.toggle_undo() />
                <PreferenceToggle id="auto-notes-enabled" label="Auto notas"
                    description="Mostrar o botão que preenche os candidatos nas células vazias." icon=IconName::AutoNotes
                    get=|s| s.auto_notes_enabled toggle=|s| s.toggle_auto_notes() />
                <PreferenceToggle id="hint-enabled" label="Dica"
                    description="Mostrar o botão que revela uma célula." icon=IconName::Hint
                    get=|s| s.hint_enabled toggle=|s| s.toggle_hint() />

                <PreferenceToggle id="drop-pick-solved" label="Selecionar número pelas células resolvidas"
                    description="No modo Drop, tocar em uma célula preenchida corretamente seleciona o número dela sem alterar a célula."
                    icon=IconName::Drop get=|s| s.drop_pick_solved toggle=|s| s.toggle_drop_pick_solved() />

                <div class="domino-preferences" aria-labelledby="domino-settings">
                    <h3 id="domino-settings">"Efeito dominó"</h3>
                    <PreferenceToggle id="domino-enabled" label="Ativar efeito dominó"
                        description="Após um acerto, preencher células com um único candidato pela linha, coluna e bloco 3×3."
                        icon=IconName::Domino get=|s| s.domino_enabled toggle=|s| s.toggle_domino() />
                    <DominoSlider target=3 id="domino-threshold" label="Ativar com até"
                        description="Quantidade de células vazias após um acerto. 0 significa sem limite." />
                    <details class="timing-details">
                        <summary>"Ajustar velocidade"</summary>
                        <DominoSlider target=0 id="domino-initial" label="Intervalo inicial"
                            description="Espera antes da primeira célula automática." />
                        <DominoSlider target=1 id="domino-acceleration" label="Aceleração por célula"
                            description="Redução do intervalo a cada célula. 0% mantém a velocidade constante." />
                        <DominoSlider target=2 id="domino-minimum" label="Intervalo mínimo"
                            description="Menor intervalo entre duas células automáticas." />
                    </details>
                </div>
            </section>

            <section class="settings-section" aria-labelledby="sound-settings">
                <h2 id="sound-settings" class="flex items-center gap-2"><Icon name=IconName::Sound />"Som"</h2>
                <fieldset class="sound-options">
                    <legend class="section-description">"Som ao acertar uma célula"</legend>
                    {[(SoundType::None, "Desligado"), (SoundType::Beep, "Bip"), (SoundType::Explosion, "Explosão")].into_iter().map(|(sound, label)| {
                        view! {
                            <label class="sound-choice" data-selected=move || (state.0.get().sound_type == sound).to_string()>
                                <input type="radio" name="sound" value=label prop:checked=move || state.0.get().sound_type == sound
                                    on:change=move |_| state.set_sound(sound) />
                                <span>{label}</span>
                            </label>
                        }
                    }).collect::<Vec<_>>()}
                </fieldset>
            </section>

            <footer class="settings-footer">
                <button class="ui-action w-full" on:click=move |_| state.reset_config()>
                    <Icon name=IconName::Reset />"Restaurar padrões"
                </button>
                <p class="preference-description">"Restaura as preferências. Seu progresso no jogo é preservado."</p>
            </footer>
        </main>
    }
}
