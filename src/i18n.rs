//! Interface language is independent of saved games, archives and shared links.
use crate::sudoku_engine::{Difficulty, Technique};
use leptos::prelude::*;
use wasm_bindgen::prelude::*;

include!(concat!(env!("OUT_DIR"), "/messages.rs"));

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Locale {
    English,
    Spanish,
    Guarani,
    Portuguese,
}
impl Locale {
    pub const ALL: [Self; 4] = [
        Self::Spanish,
        Self::Guarani,
        Self::Portuguese,
        Self::English,
    ];
    pub fn tag(self) -> &'static str {
        match self {
            Self::English => "en-US",
            Self::Spanish => "es-AR",
            Self::Guarani => "gn-PY",
            Self::Portuguese => "pt-BR",
        }
    }
    pub fn from_tag(tag: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|locale| locale.tag() == tag)
    }
    pub fn flag(self) -> &'static str {
        match self {
            Self::English => "🇺🇸",
            Self::Spanish => "🇦🇷",
            Self::Guarani => "🇵🇾",
            Self::Portuguese => "🇧🇷",
        }
    }
    pub fn name(self) -> &'static str {
        match self {
            Self::English => "English (United States)",
            Self::Spanish => "Español (Argentina)",
            Self::Guarani => "Avañe’ẽ (Paraguái)",
            Self::Portuguese => "Português (Brasil)",
        }
    }
    pub fn text(self, message: Message) -> &'static str {
        MESSAGES[self as usize][message as usize]
    }
}

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = __sudokuI18n, js_name = setLocale)]
    fn set_locale(locale: &str);
    #[wasm_bindgen(js_namespace = __sudokuI18n, js_name = currentLocale)]
    fn current_locale() -> String;
    #[wasm_bindgen(js_namespace = __sudokuI18n, js_name = failedToSave)]
    fn save_failed() -> bool;
    #[wasm_bindgen(js_namespace = __sudokuI18n, js_name = playedDate)]
    fn played_date(ms: f64) -> String;
}

#[derive(Clone, Copy)]
pub struct I18n {
    pub locale: RwSignal<Locale>,
    pub save_failed: RwSignal<bool>,
}
impl Default for I18n {
    fn default() -> Self {
        #[cfg(target_arch = "wasm32")]
        let (locale, failed) = (
            Locale::from_tag(&current_locale()).unwrap_or(Locale::English),
            save_failed(),
        );
        #[cfg(not(target_arch = "wasm32"))]
        let (locale, failed) = (Locale::from_tag("en-US").unwrap(), false);
        Self {
            locale: RwSignal::new(locale),
            save_failed: RwSignal::new(failed),
        }
    }
}
impl I18n {
    pub fn select(self, locale: Locale) {
        self.locale.set(locale);
        #[cfg(target_arch = "wasm32")]
        {
            set_locale(locale.tag());
            self.save_failed.set(save_failed());
        }
    }
    pub fn text(self, message: Message) -> &'static str {
        self.locale.get().text(message)
    }
    pub fn format(self, message: Message, args: &[(&str, String)]) -> String {
        format_message(self.text(message), args)
    }
    pub fn date(self, ms: f64) -> String {
        let _ = self.locale.get();
        played_date(ms)
    }
    pub fn difficulty(self, difficulty: Difficulty) -> &'static str {
        self.text(match difficulty {
            Difficulty::Easy => Message::Easy,
            Difficulty::Medium => Message::Medium,
            Difficulty::Hard => Message::Hard,
            Difficulty::Expert => Message::Expert,
            Difficulty::Master => Message::Master,
        })
    }
    pub fn technique(self, technique: Technique) -> &'static str {
        self.text(match technique {
            Technique::NakedSingle => Message::NakedSingle,
            Technique::HiddenSingle => Message::HiddenSingle,
            Technique::LockedCandidates => Message::LockedCandidates,
            Technique::NakedSubset => Message::NakedSubset,
            Technique::HiddenSubset => Message::HiddenSubset,
            Technique::XWing => Message::XWing,
            Technique::XYWing => Message::XYWing,
            Technique::SimpleColoring => Message::SimpleColoring,
            Technique::ShortChain => Message::ShortChain,
            Technique::XChain => Message::XChain,
        })
    }
    pub fn color(self, color: u8) -> &'static str {
        self.text(match color {
            1 => Message::Green,
            2 => Message::Purple,
            3 => Message::Pink,
            4 => Message::Orange,
            _ => Message::NoColor,
        })
    }
    pub fn notice(self, notice: Option<Notice>) -> String {
        notice
            .map(|notice| {
                let error = self.text(notice.message).to_string();
                if notice.previous_saved {
                    self.format(Message::PreviousSaved, &[("error", error)])
                } else {
                    error
                }
            })
            .unwrap_or_default()
    }
}

pub fn use_i18n() -> I18n {
    expect_context::<I18n>()
}

/// Named substitution in a single pass; argument values are never reparsed.
fn format_message(template: &str, args: &[(&str, String)]) -> String {
    let mut result = String::new();
    let mut rest = template;
    while let Some(start) = rest.find('{') {
        result.push_str(&rest[..start]);
        rest = &rest[start..];
        let Some(end) = rest.find('}') else { break };
        let key = &rest[1..end];
        if let Some((_, value)) = args.iter().find(|(name, _)| *name == key) {
            result.push_str(value);
        } else {
            result.push_str(&rest[..=end]);
        }
        rest = &rest[end + 1..];
    }
    result.push_str(rest);
    result
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Notice {
    pub message: Message,
    pub previous_saved: bool,
}
impl From<Message> for Notice {
    fn from(message: Message) -> Self {
        Self {
            message,
            previous_saved: false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn named_arguments_can_be_reordered_and_are_not_reparsed() {
        assert_eq!(
            format_message(
                "{second}: {first}",
                &[("first", "{second}".into()), ("second", "value".into())]
            ),
            "value: {second}"
        );
    }
    #[test]
    fn notices_follow_the_current_language() {
        let owner = Owner::new();
        owner.with(|| {
            let i18n = I18n::default();
            let notice = Some(Notice {
                message: Message::SaveFailed,
                previous_saved: true,
            });
            assert!(i18n.notice(notice).contains("could not be saved"));
            i18n.select(Locale::Portuguese);
            assert!(i18n.notice(notice).contains("Não foi possível salvar"));
        });
    }
    #[test]
    fn every_locale_has_different_settings_and_complete_cell_templates() {
        for locale in Locale::ALL {
            assert!(!locale.text(Message::Settings).is_empty());
            assert!(locale.text(Message::CellLabel).contains("{row}"));
        }
        assert_eq!(Locale::from_tag("unknown"), None);
    }
}
