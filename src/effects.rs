//! Saved effect preferences; browser resources never enter saved game data.
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct EffectsSettings {
    pub volume: u8,
    pub balloons: u8,
}
impl Default for EffectsSettings {
    fn default() -> Self {
        Self {
            volume: 75,
            balloons: 25,
        }
    }
}
impl EffectsSettings {
    pub fn normalize(&mut self) {
        self.volume = self.volume.min(100);
        self.balloons = self.balloons.min(100);
    }
}

#[cfg(target_arch = "wasm32")]
pub fn celebrate(sound: crate::state::SoundType, settings: EffectsSettings) {
    crate::celebration::start(settings.balloons);
    crate::audio::play(sound, settings.volume, true);
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn partial_preferences_default_and_normalize() {
        let mut settings: EffectsSettings = serde_json::from_str("{\"volume\":255}").unwrap();
        assert_eq!(settings.balloons, 25);
        settings.normalize();
        assert_eq!(settings.volume, 100);
        settings.balloons = 255;
        settings.normalize();
        assert_eq!(settings.balloons, 100);
        assert_eq!(
            serde_json::from_str::<EffectsSettings>("{}").unwrap(),
            EffectsSettings::default()
        );
    }
}
