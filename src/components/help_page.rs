use crate::components::icon::{Icon, IconName};
use crate::i18n::{use_i18n, Message};
use leptos::prelude::*;
use leptos_router::components::A;

#[component]
pub fn HelpPage() -> impl IntoView {
    let i18n = use_i18n();
    let items: Vec<(Option<IconName>, Message, Message)> = vec![
        (None, Message::Numbers, Message::HelpNumbers),
        (Some(IconName::Erase), Message::Erase, Message::HelpErase),
        (
            Some(IconName::Pencil),
            Message::NotesMode,
            Message::HelpNotes,
        ),
        (Some(IconName::Drop), Message::DropMode, Message::HelpDrop),
        (
            Some(IconName::Menu),
            Message::GameHistory,
            Message::HelpHistory,
        ),
        (
            Some(IconName::Menu),
            Message::ShareSudoku,
            Message::HelpSharing,
        ),
        (
            Some(IconName::Pencil),
            Message::CellColors,
            Message::HelpColors,
        ),
        (Some(IconName::Pause), Message::Pause, Message::HelpPause),
        (
            Some(IconName::Clock),
            Message::ErrorCounter,
            Message::HelpErrors,
        ),
        (
            Some(IconName::Reset),
            Message::NewGame,
            Message::HelpNewGame,
        ),
        (Some(IconName::Undo), Message::Undo, Message::HelpUndo),
        (Some(IconName::Redo), Message::Redo, Message::HelpRedo),
        (
            Some(IconName::AutoNotes),
            Message::AutoNotes,
            Message::HelpAutoNotes,
        ),
        (Some(IconName::Hint), Message::Hint, Message::HelpHint),
        (Some(IconName::Domino), Message::Domino, Message::HelpDomino),
        (Some(IconName::Sound), Message::Sound, Message::HelpSound),
        (Some(IconName::Moon), Message::DarkMode, Message::HelpDark),
        (
            Some(IconName::Install),
            Message::Install,
            Message::HelpInstall,
        ),
    ];

    view! {
        <main class="settings-page">
            <A href="/" attr:class="ui-back"><Icon name=IconName::Back /><span>{move || i18n.text(Message::Back)}</span></A>
            <h1 class="page-title">{move || i18n.text(Message::HowPlay)}</h1>
            <p class="page-description">{move || i18n.text(Message::HelpIntro)}</p>
            <div class="settings-section">
                {items.into_iter().map(|(icon, name, desc)| {
                    view! {
                        <div class="help-item">
                            <span aria-hidden="true">{match icon {
                                Some(name) => view! { <Icon name=name /> }.into_any(),
                                None => view! { <span class="font-bold">"1–9"</span> }.into_any(),
                            }}</span>
                            <div>
                                <h2 class="preference-label">{move || i18n.text(name)}</h2>
                                <p class="section-description">{move || i18n.text(desc)}</p>
                            </div>
                        </div>
                    }
                }).collect::<Vec<_>>()}
            </div>

            <div class="text-center mt-10">
                <p class="text-sm text-muted max-w-[280px] mx-auto leading-relaxed">
                    {move || i18n.text(Message::FreeIntro)}
                </p>
                <p class="text-xs text-muted mt-5">
                    {move || i18n.text(Message::FoundProblem)}
                    <a
                        href="https://github.com/wuerges/free-sudoku-spa/issues"
                        target="_blank"
                        rel="noopener"
                        class="text-accent underline"
                    >
                        {move || i18n.text(Message::OpenIssue)}
                    </a>
                </p>
            </div>

        </main>
    }
}
