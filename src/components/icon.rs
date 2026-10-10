use leptos::prelude::*;

/// Small, local line icons. The surrounding control supplies its accessible name.
#[derive(Clone, Copy)]
pub enum IconName {
    Back,
    Settings,
    Help,
    Sun,
    Moon,
    Install,
    Erase,
    Pencil,
    Drop,
    Undo,
    Redo,
    AutoNotes,
    Hint,
    Domino,
    Sound,
    Reset,
    Pause,
    Play,
    Clock,
}

#[component]
pub fn Icon(name: IconName) -> impl IntoView {
    let path = match name {
        IconName::Back => "M15 5l-7 7 7 7M8 12h12",
        IconName::Settings => "M9 3h6l1 3 3 1 2 5-2 5-3 1-1 3H9l-1-3-3-1-2-5 2-5 3-1zM16 12a4 4 0 1 1-8 0 4 4 0 0 1 8 0",
        IconName::Help => "M22 12a10 10 0 1 1-20 0 10 10 0 0 1 20 0M9 8a3 3 0 0 1 6 0c0 2-3 2-3 4M12 16v.5",
        IconName::Sun => "M16 12a4 4 0 1 1-8 0 4 4 0 0 1 8 0M12 2v2M12 20v2M2 12h2M20 12h2M5 5l1.5 1.5M17.5 17.5L19 19M5 19l1.5-1.5M17.5 6.5L19 5",
        IconName::Moon => "M20 15A9 9 0 0 1 9 4a9 9 0 1 0 11 11",
        IconName::Install => "M12 3v12M7 10l5 5 5-5M4 16v5h16v-5",
        IconName::Erase => "M9 5h12v14H9l-7-7zM12 9l6 6M18 9l-6 6",
        IconName::Pencil => "M4 16L16 4l4 4L8 20H4zM13 7l4 4",
        IconName::Drop => "M21 12a9 9 0 1 1-18 0 9 9 0 0 1 18 0M17 12a5 5 0 1 1-10 0 5 5 0 0 1 10 0M13 12a1 1 0 1 1-2 0 1 1 0 0 1 2 0",
        IconName::Undo => "M8 4L3 9l5 5M3 9h10a7 7 0 0 1 0 14",
        IconName::Redo => "M16 4l5 5-5 5M21 9H11a7 7 0 0 0 0 14",
        IconName::AutoNotes => "M4 3h10l4 4v4M4 3v18h7M14 3v5h4M13 17l5-5 3 3-5 5-4 1zM7 8h3M7 12h3",
        IconName::Hint => "M9 18h6M9 21h6M8 15a7 7 0 1 1 8 0l-1 3H9z",
        IconName::Domino => "M5 3h14v18H5zM5 12h14M9 7v.5M15 16v.5",
        IconName::Sound => "M3 9h4l5-4v14l-5-4H3zM16 8a6 6 0 0 1 0 8M19 5a10 10 0 0 1 0 14",
        IconName::Reset => "M3 4v6h6M3 10a9 9 0 1 1 1 8",
        IconName::Pause => "M8 5v14M16 5v14",
        IconName::Play => "M7 4l13 8-13 8z",
        IconName::Clock => "M22 12a10 10 0 1 1-20 0 10 10 0 0 1 20 0M12 6v6l4 2",
    };
    view! {
        <svg class="ui-icon" width="20" height="20" viewBox="0 0 24 24" fill="none"
            stroke="currentColor" stroke-width="1.8" stroke-linecap="round" stroke-linejoin="round"
            aria-hidden="true" focusable="false"><path d=path /></svg>
    }
}
