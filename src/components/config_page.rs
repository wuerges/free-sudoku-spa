use crate::components::icon::{Icon, IconName};
use crate::i18n::{use_i18n, Message};
use crate::state::{AppState, GameState, SoundType};
use leptos::prelude::*;
use leptos_router::components::A;

#[component]
fn PreferenceToggle(
    id: &'static str,
    label: Message,
    description: Message,
    icon: IconName,
    get: fn(&GameState) -> bool,
    toggle: fn(&AppState),
) -> impl IntoView {
    let i18n = use_i18n();
    let state: AppState = use_context().unwrap();
    let description_id = format!("{id}-description");
    view! {
        <label for=id class="preference-row">
            <Icon name=icon />
            <span class="min-w-0">
                <span class="preference-label">{move || i18n.text(label)}</span>
                <span id=description_id.clone() class="preference-description">{move || i18n.text(description)}</span>
            </span>
            <input id=id type="checkbox" aria-label=move || i18n.text(label) aria-describedby=description_id
                class="preference-checkbox" prop:checked=move || get(&state.0.get())
                on:change=move |_| toggle(&state) />
        </label>
    }
}

#[component]
fn DominoSlider(
    target: u8,
    id: &'static str,
    label: Message,
    description: Message,
) -> impl IntoView {
    let i18n = use_i18n();
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
        _ if value() == 0 => i18n.text(Message::NoLimit).to_string(),
        _ => i18n.format(Message::CellCount, &[("count", value().to_string())]),
    };
    view! {
        <div class="preference-slider">
            <label for=id class="slider-label"><span>{move || i18n.text(label)}</span><output for=id>{formatted}</output></label>
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
            <p id=description_id class="preference-description">{move || i18n.text(description)}</p>
        </div>
    }
}

#[component]
pub fn ConfigPage() -> impl IntoView {
    let i18n = use_i18n();
    let state: AppState = use_context().unwrap();
    view! {
        <main class="settings-page">
            <A href="/" attr:class="ui-back"><Icon name=IconName::Back /><span>{move || i18n.text(Message::Back)}</span></A>
            <h1 class="page-title">{move || i18n.text(Message::Settings)}</h1>
            <p class="page-description">{move || i18n.text(Message::SettingsIntro)}</p>

            <section class="settings-section" aria-labelledby="language-settings">
                <h2 id="language-settings">{move || i18n.text(Message::Language)}</h2>
                <p class="section-description">{move || i18n.text(Message::LanguageIntro)}</p>
                <div class="language-options" role="group" aria-labelledby="language-settings">
                    {crate::i18n::Locale::ALL.into_iter().map(|locale| view! {
                        <button type="button" class="ui-action language-choice" lang=locale.tag()
                            aria-label=locale.name() aria-pressed=move || (i18n.locale.get() == locale).to_string()
                            data-active=move || (i18n.locale.get() == locale).to_string()
                            on:click=move |_| i18n.select(locale)>
                            <span aria-hidden="true">{locale.flag()}</span><span>{locale.name()}</span>
                        </button>
                    }).collect::<Vec<_>>()}
                </div>
            </section>
            <section class="settings-section" aria-labelledby="highlight-settings">
                <h2 id="highlight-settings">{move || i18n.text(Message::BoardHighlights)}</h2>
                <p class="section-description">{move || i18n.text(Message::HighlightsIntro)}</p>
                {[(0, Message::SelectedShading, "selected-shading"),
                  (1, Message::MatchingShading, "matching-shading"),
                  (2, Message::AvailableShading, "available-shading")].into_iter().map(|(target, label, id)| {
                    let value = move || { let h = state.0.get().highlights;
                        match target { 0 => h.selected_shading, 1 => h.matching_shading, _ => h.available_shading } };
                    view! {
                        <div class="preference-slider">
                            <label for=id class="slider-label"><span>{move || i18n.text(label)}</span><output for=id>{move || format!("{}%", value())}</output></label>
                            <input id=id type="range" min="0" max="100" step="1"
                                aria-valuetext=move || format!("{}%", value()) prop:value=move || value().to_string()
                                on:input=move |ev| { if let Ok(value) = event_target_value(&ev).parse::<u8>() {
                                    state.0.update(|s| match target { 0 => s.highlights.selected_shading = value.min(100),
                                        1 => s.highlights.matching_shading = value.min(100), _ => s.highlights.available_shading = value.min(100) });
                                } } />
                        </div>
                    }
                }).collect::<Vec<_>>()}
                <label class="simple-preference-row"><span>{move || i18n.text(Message::AvailableDots)}</span>
                    <input type="checkbox" class="preference-checkbox" prop:checked=move || state.0.get().highlights.dots
                        on:change=move |ev| state.0.update(|s| s.highlights.dots = event_target_checked(&ev)) />
                </label>
                <label class="simple-preference-row"><span>{move || i18n.text(Message::BlockedStripes)}</span>
                    <input type="checkbox" class="preference-checkbox" prop:checked=move || state.0.get().highlights.stripes
                        on:change=move |ev| state.0.update(|s| s.highlights.stripes = event_target_checked(&ev)) />
                </label>
            </section>

            <section class="settings-section" aria-labelledby="completed-settings">
                <h2 id="completed-settings">{move || i18n.text(Message::CompletedNumbers)}</h2>
                <p id="completed-contrast-description" class="section-description">{move || i18n.text(Message::CompletedIntro)}</p>
                <div class="preference-slider">
                    <label for="completed-contrast" class="slider-label"><span>{move || i18n.text(Message::CompletedContrast)}</span>
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
                <h2 id="assistance-settings">{move || i18n.text(Message::Assistances)}</h2>
                <p class="section-description">{move || i18n.text(Message::AssistanceIntro)}</p>
                <PreferenceToggle id="undo-enabled" label=Message::UndoRedo
                    description=Message::UndoToggleIntro icon=IconName::Undo
                    get=|s| s.undo_enabled toggle=|s| s.toggle_undo() />
                <PreferenceToggle id="auto-notes-enabled" label=Message::AutoNotes
                    description=Message::AutoNotesToggleIntro icon=IconName::AutoNotes
                    get=|s| s.auto_notes_enabled toggle=|s| s.toggle_auto_notes() />
                <PreferenceToggle id="hint-enabled" label=Message::Hint
                    description=Message::HintToggleIntro icon=IconName::Hint
                    get=|s| s.hint_enabled toggle=|s| s.toggle_hint() />

                <PreferenceToggle id="drop-pick-solved" label=Message::PickSolved
                    description=Message::PickSolvedIntro
                    icon=IconName::Drop get=|s| s.drop_pick_solved toggle=|s| s.toggle_drop_pick_solved() />

                <div class="domino-preferences" aria-labelledby="domino-settings">
                    <h3 id="domino-settings">{move || i18n.text(Message::Domino)}</h3>
                    <PreferenceToggle id="domino-enabled" label=Message::EnableDomino
                        description=Message::DominoIntro
                        icon=IconName::Domino get=|s| s.domino_enabled toggle=|s| s.toggle_domino() />
                    <DominoSlider target=3 id="domino-threshold" label=Message::ActivateUpTo
                        description=Message::ThresholdIntro />
                    <details class="timing-details">
                        <summary>{move || i18n.text(Message::AdjustSpeed)}</summary>
                        <DominoSlider target=0 id="domino-initial" label=Message::InitialDelay
                            description=Message::InitialDelayIntro />
                        <DominoSlider target=1 id="domino-acceleration" label=Message::Acceleration
                            description=Message::AccelerationIntro />
                        <DominoSlider target=2 id="domino-minimum" label=Message::MinimumDelay
                            description=Message::MinimumDelayIntro />
                    </details>
                </div>
            </section>

            <section class="settings-section" aria-labelledby="sound-settings">
                <h2 id="sound-settings" class="flex items-center gap-2"><Icon name=IconName::Sound />{move || i18n.text(Message::Sound)}</h2>
                <fieldset class="sound-options">
                    <legend class="section-description">{move || i18n.text(Message::CorrectSound)}</legend>
                    {[(SoundType::None, Message::Off), (SoundType::Beep, Message::Beep), (SoundType::Explosion, Message::Explosion)].into_iter().map(|(sound, label)| {
                        view! {
                            <label class="sound-choice" data-selected=move || (state.0.get().sound_type == sound).to_string()>
                                <input type="radio" name="sound" value=move || i18n.text(label) prop:checked=move || state.0.get().sound_type == sound
                                    on:change=move |_| state.set_sound(sound) />
                                <span>{move || i18n.text(label)}</span>
                            </label>
                        }
                    }).collect::<Vec<_>>()}
                </fieldset>
            </section>

            <footer class="settings-footer">
                <button class="ui-action w-full" on:click=move |_| state.reset_config()>
                    <Icon name=IconName::Reset />{move || i18n.text(Message::RestoreDefaults)}
                </button>
                <p class="preference-description">{move || i18n.text(Message::RestoreIntro)}</p>
            </footer>
        </main>
    }
}
