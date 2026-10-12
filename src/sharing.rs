//! Untrusted, versioned link format, independent from local save serialization.
use crate::i18n::Message;
use crate::{
    serde_helpers::{u16_81, u8_81},
    state::GameState,
    sudoku_engine::{self, Difficulty, Rating},
};
use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine};
use serde::{Deserialize, Serialize};
pub const PUBLIC_URL: &str = env!("SUDOKU_RESOLVED_PUBLIC_URL");
pub const MAX_PAYLOAD: usize = 8192;
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SharedGame {
    v: u8,
    #[serde(with = "u8_81")]
    pub board: [u8; 81],
    #[serde(with = "u8_81")]
    pub givens: [u8; 81],
    #[serde(with = "u8_81")]
    pub solution: [u8; 81],
    #[serde(with = "u16_81")]
    pub notes: [u16; 81],
    #[serde(with = "u8_81")]
    pub colors: [u8; 81],
    #[serde(with = "u8_81")]
    pub hinted: [u8; 81],
    pub difficulty: Difficulty,
    pub requested: Option<Difficulty>,
    pub rating: Option<Rating>,
    pub seed: u64,
    pub seconds: u32,
    pub errors: u32,
}
impl SharedGame {
    pub fn capture(s: &GameState, initial: bool) -> Self {
        let game = Self {
            v: 1,
            board: s.board,
            givens: s.givens,
            solution: s.solution,
            notes: s.notes,
            colors: s.cell_colors,
            hinted: s.hinted,
            difficulty: s.difficulty,
            requested: s.requested_difficulty,
            rating: s.rating.clone(),
            seed: s.seed,
            seconds: s.timer_seconds,
            errors: s.error_count,
        };
        if initial {
            game.initial_position()
        } else {
            game
        }
    }

    /// Derive the initial puzzle without modifying the received snapshot.
    pub fn initial_position(&self) -> Self {
        let mut game = self.clone();
        game.board = game.givens;
        game.notes = [0; 81];
        game.colors = [0; 81];
        game.hinted = [0; 81];
        game.seconds = 0;
        game.errors = 0;
        game
    }

    pub fn validate(&self) -> Result<(), Message> {
        if self.v != 1 {
            return Err(Message::UnsupportedShareVersion);
        }
        for i in 0..81 {
            let n = self.solution[i];
            if !(1..=9).contains(&n)
                || !sudoku_engine::is_valid_move(&self.solution, i / 9, i % 9, n)
                || self.board[i] > 9
                || self.givens[i] > 9
                || self.notes[i] > 511
                || self.colors[i] > 4
                || self.hinted[i] > 9
                || (self.givens[i] != 0 && (self.givens[i] != n || self.board[i] != n))
                || (self.hinted[i] != 0 && self.board[i] != n)
            {
                return Err(Message::InvalidSudoku);
            }
        }
        Ok(())
    }
    pub fn link(&self) -> Result<String, Message> {
        self.validate()?;
        let payload = URL_SAFE_NO_PAD
            .encode(serde_json::to_vec(self).map_err(|_| Message::GenerateLinkFailed)?);
        if payload.len() > MAX_PAYLOAD {
            return Err(Message::LinkTooLarge);
        }
        Ok(format!("{PUBLIC_URL}/#sudoku={payload}"))
    }
    pub fn decode(payload: &str) -> Result<Self, Message> {
        if payload.len() > MAX_PAYLOAD {
            return Err(Message::LinkTooLarge);
        }
        let bytes = URL_SAFE_NO_PAD
            .decode(payload)
            .map_err(|_| Message::InvalidLink)?;
        let game: Self = serde_json::from_slice(&bytes).map_err(|_| Message::InvalidLink)?;
        game.validate()?;
        Ok(game)
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn restart_received_completed_game_preserves_puzzle_and_settings() {
        let mut sender = GameState::default();
        sender.board = sender.solution;
        sender.notes[0] = 511;
        sender.cell_colors[0] = 4;
        let i = sender.givens.iter().position(|&v| v == 0).unwrap();
        sender.hinted[i] = 1;
        sender.timer_seconds = 456;
        sender.error_count = 7;
        let current = SharedGame::capture(&sender, false);
        let initial = current.initial_position();
        initial.validate().unwrap();
        assert_eq!(initial.board, sender.givens);
        assert_eq!(initial.solution, sender.solution);
        assert_eq!(initial.rating, sender.rating);
        assert_eq!(initial.seed, sender.seed);
        assert_eq!(initial.notes, [0; 81]);
        assert_eq!(initial.colors, [0; 81]);
        assert_eq!(initial.hinted, [0; 81]);
        assert_eq!(initial.seconds, 0);
        assert_eq!(initial.errors, 0);
        assert_eq!(current.board, sender.solution);
        assert_eq!(current.seconds, 456);
        let mut recipient = sender.clone();
        recipient.highlights.completed_contrast = 37;
        recipient.import_shared(&initial);
        assert!(!recipient.won && recipient.paused);
        assert_eq!(recipient.highlights.completed_contrast, 37);
        recipient.import_shared(&current);
        assert!(recipient.won && recipient.celebrated && recipient.paused);
        assert_eq!(recipient.timer_seconds, 456);
    }
    #[test]
    fn imported_counters_do_not_overflow() {
        let owner = leptos::prelude::Owner::new();
        owner.with(|| {
            let mut s = GameState::default();
            s.timer_seconds = u32::MAX;
            s.error_count = u32::MAX;
            let shared = SharedGame::capture(&s, false);
            s.import_shared(&shared);
            s.paused = false;
            let app = crate::state::AppState(leptos::prelude::RwSignal::new(s));
            app.tick_timer();
            use leptos::prelude::Get;
            assert_eq!(app.0.get().timer_seconds, u32::MAX);
        });
    }
    #[test]
    fn legacy_clues_and_hints_can_overlap() {
        let mut state = GameState::default();
        let i = state.board.iter().position(|&v| v != 0).unwrap();
        state.hinted[i] = 1;
        assert!(SharedGame::capture(&state, false).link().is_ok());
    }
    #[test]
    fn import_preserves_preferences_and_clears_transients() {
        let mut recipient = GameState::default();
        recipient.undo_enabled = false;
        recipient.highlights.completed_contrast = 37;
        recipient.domino.initial_delay_ms = 1234;
        recipient.sound_type = crate::state::SoundType::None;
        recipient.note_mode = true;
        recipient.drop_mode = true;
        recipient.paint_tool = Some(2);
        recipient.selected = Some((0, 0));
        recipient.history.push(crate::state::Snapshot {
            board: recipient.board,
            notes: recipient.notes,
            cell_colors: recipient.cell_colors,
        });
        recipient.redo_stack = recipient.history.clone();
        let mut sender = recipient.clone();
        sender.board = sender.solution;
        sender.timer_seconds = 42;
        let game = SharedGame::capture(&sender, false);
        let decoded =
            SharedGame::decode(game.link().unwrap().split("#sudoku=").nth(1).unwrap()).unwrap();
        let generation = recipient.domino_gen;
        recipient.import_shared(&decoded);
        assert_eq!(recipient.board, sender.solution);
        assert!(recipient.won && recipient.celebrated && recipient.paused);
        assert_eq!(recipient.timer_seconds, 42);
        assert!(!recipient.undo_enabled);
        assert_eq!(recipient.highlights.completed_contrast, 37);
        assert_eq!(recipient.domino.initial_delay_ms, 1234);
        assert_eq!(recipient.sound_type, crate::state::SoundType::None);
        assert!(recipient.history.is_empty() && recipient.redo_stack.is_empty());
        assert!(recipient.selected.is_none() && recipient.paint_tool.is_none());
        assert!(recipient.note_mode && recipient.drop_mode);
        assert_ne!(recipient.domino_gen, generation);
    }
    #[test]
    fn modes_and_validation() {
        let mut s = GameState::default();
        let i = s.board.iter().position(|&v| v == 0).unwrap();
        s.board[i] = s.solution[i] % 9 + 1;
        s.notes[i] = 511;
        s.cell_colors[i] = 4;
        s.error_count = 9;
        s.timer_seconds = 123;
        let g = SharedGame::capture(&s, false);
        let link = g.link().unwrap();
        let decoded = SharedGame::decode(link.split("#sudoku=").nth(1).unwrap()).unwrap();
        assert_eq!(decoded.board, s.board);
        assert_eq!(decoded.notes, s.notes);
        assert_eq!(decoded.colors, s.cell_colors);
        let initial = SharedGame::capture(&s, true);
        assert_eq!(initial.board, s.givens);
        assert_eq!(initial.notes, [0; 81]);
        assert_eq!(initial.seconds, 0);
        for mutate in [0, 1, 2, 3, 4, 5] {
            let mut bad = g.clone();
            match mutate {
                0 => bad.v = 2,
                1 => bad.solution[0] = 0,
                2 => bad.notes[0] = 512,
                3 => bad.colors[0] = 5,
                4 => bad.givens[i] = bad.solution[i],
                _ => bad.hinted[i] = 1,
            };
            assert!(bad.validate().is_err());
        }
        assert!(SharedGame::decode(&"a".repeat(MAX_PAYLOAD + 1)).is_err());
        assert!(SharedGame::decode("!!!").is_err());
        let mut json = serde_json::to_value(&g).unwrap();
        json["board"] = serde_json::json!([0]);
        assert!(
            SharedGame::decode(&URL_SAFE_NO_PAD.encode(serde_json::to_vec(&json).unwrap()))
                .is_err()
        );
    }
}
