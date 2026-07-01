// ponytail: single RwSignal<GameState>. ceiling: 81 cells re-render on any change. upgrade: per-cell signals if frame drops on low-end devices.

use crate::serde_helpers::{u16_81, u8_81};
use crate::sudoku_engine::{self, Difficulty};
use leptos::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct GameState {
    #[serde(with = "u8_81")]
    pub board: [u8; 81],
    #[serde(with = "u8_81")]
    pub solution: [u8; 81],
    #[serde(with = "u16_81")]
    pub notes: [u16; 81],
    pub difficulty: Difficulty,
    pub note_mode: bool,
    pub selected: Option<(usize, usize)>,
    pub timer_seconds: u32,
    pub paused: bool,
    pub won: bool,
    pub history: Vec<Snapshot>,
    pub redo_stack: Vec<Snapshot>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Snapshot {
    #[serde(with = "u8_81")]
    pub board: [u8; 81],
    #[serde(with = "u16_81")]
    pub notes: [u16; 81],
}

impl Default for GameState {
    fn default() -> Self {
        let board = sudoku_engine::generate(40..=45);
        Self {
            board: board.cells,
            solution: board.solution,
            notes: [0u16; 81],
            difficulty: Difficulty::Easy,
            note_mode: false,
            selected: None,
            timer_seconds: 0,
            paused: false,
            won: false,
            history: Vec::new(),
            redo_stack: Vec::new(),
        }
    }
}

impl GameState {
    fn idx(r: usize, c: usize) -> usize {
        r * 9 + c
    }

    pub fn get(&self, r: usize, c: usize) -> u8 {
        self.board[Self::idx(r, c)]
    }

    pub fn is_given(&self, r: usize, c: usize) -> bool {
        self.solution[Self::idx(r, c)] != 0 && self.get(r, c) != 0
            && self.get(r, c) == self.solution[Self::idx(r, c)]
    }

    pub fn push_snapshot(&mut self) {
        self.history.push(Snapshot {
            board: self.board,
            notes: self.notes,
        });
        self.redo_stack.clear();
    }

    pub fn undo(&mut self) {
        if let Some(snap) = self.history.pop() {
            self.redo_stack.push(Snapshot {
                board: self.board,
                notes: self.notes,
            });
            self.board = snap.board;
            self.notes = snap.notes;
        }
    }

    pub fn redo(&mut self) {
        if let Some(snap) = self.redo_stack.pop() {
            self.history.push(Snapshot {
                board: self.board,
                notes: self.notes,
            });
            self.board = snap.board;
            self.notes = snap.notes;
        }
    }
}

#[derive(Clone, Copy)]
pub struct AppState(pub RwSignal<GameState>);

impl AppState {
    pub fn new() -> Self {
        // ponytail: localStorage load with console.warn on failure. ceiling: user doesn't see console. upgrade: toast notification if players report lost progress.
        let saved = load_state();
        AppState(RwSignal::new(saved.unwrap_or_default()))
    }

    pub fn new_game(&self, difficulty: Difficulty) {
        let board = match difficulty {
            Difficulty::Easy => sudoku_engine::generate(40..=45),
            Difficulty::Medium => sudoku_engine::generate(32..=38),
            Difficulty::Hard => sudoku_engine::generate(26..=31),
            Difficulty::Expert => sudoku_engine::generate(20..=25),
            Difficulty::Master => sudoku_engine::generate(17..=19),
        };
        self.0.update(|s| {
            *s = GameState {
                board: board.cells,
                solution: board.solution,
                notes: [0u16; 81],
                difficulty,
                note_mode: s.note_mode,
                selected: None,
                timer_seconds: 0,
                paused: false,
                won: false,
                history: Vec::new(),
                redo_stack: Vec::new(),
            };
        });
    }

    pub fn select_cell(&self, r: usize, c: usize) {
        self.0.update(|s| s.selected = Some((r, c)));
    }

    pub fn place_number(&self, v: u8) {
        self.0.update(|s| {
            if let Some((r, c)) = s.selected {
                if s.is_given(r, c) || s.won {
                    return;
                }
                s.push_snapshot();

                if s.note_mode {
                    if v == 0 {
                        s.notes[GameState::idx(r, c)] = 0;
                    } else {
                        let bit = 1 << (v - 1);
                        s.notes[GameState::idx(r, c)] ^= bit;
                    }
                } else {
                    if v == 0 {
                        s.board[GameState::idx(r, c)] = 0;
                    } else {
                        let idx = GameState::idx(r, c);
                        s.board[idx] = v;
                        s.notes[idx] = 0;
                        // If correct, remove notes of this number from row/col/box
                        if v == s.solution[idx] {
                            let bit = !(1 << (v - 1));
                            for i in 0..9 {
                                s.notes[GameState::idx(r, i)] &= bit;
                                s.notes[GameState::idx(i, c)] &= bit;
                            }
                            let br = (r / 3) * 3;
                            let bc = (c / 3) * 3;
                            for rr in br..br + 3 {
                                for cc in bc..bc + 3 {
                                    s.notes[GameState::idx(rr, cc)] &= bit;
                                }
                            }
                        }
                    }
                }

                // Check win
                if !s.note_mode && s.board == s.solution {
                    s.won = true;
                }
            }
        });
    }

    pub fn undo(&self) {
        self.0.update(|s| s.undo());
    }

    pub fn redo(&self) {
        self.0.update(|s| s.redo());
    }

    pub fn toggle_note_mode(&self) {
        self.0.update(|s| s.note_mode = !s.note_mode);
    }

    pub fn hint(&self) {
        self.0.update(|s| {
            // Hint: pick the unsolved cell with fewest candidates.
            let mut best: Option<(usize, u32)> = None;
            for i in 0..81 {
                if s.board[i] != s.solution[i] {
                    let cands = (1..=9).filter(|&v| {
                        let r = i / 9;
                        let c = i % 9;
                        crate::sudoku_engine::is_valid_move(&s.board, r, c, v)
                    }).count() as u32;
                    if cands > 0 && best.is_none_or(|(_, n)| cands < n) {
                        best = Some((i, cands));
                    }
                }
            }
            if let Some((i, _)) = best {
                s.push_snapshot();
                s.board[i] = s.solution[i];
                s.notes[i] = 0;
                if s.board == s.solution {
                    s.won = true;
                }
            }
        });
    }

    pub fn tick_timer(&self) {
        self.0.update(|s| {
            if !s.paused && !s.won {
                s.timer_seconds += 1;
            }
        });
    }

    pub fn auto_notes(&self) {
        self.0.update(|s| {
            s.push_snapshot();
            for i in 0..81 {
                if s.board[i] == 0 {
                    let row = i / 9;
                    let col = i % 9;
                    let cands = sudoku_engine::candidates(&s.board, row, col);
                    let mut bits = 0u16;
                    for v in cands {
                        bits |= 1 << (v - 1);
                    }
                    s.notes[i] = bits;
                } else {
                    s.notes[i] = 0;
                }
            }
        });
    }

    pub fn toggle_pause(&self) {
        self.0.update(|s| s.paused = !s.paused);
    }
}

fn load_state() -> Option<GameState> {
    #[cfg(target_arch = "wasm32")]
    {
        let window = web_sys::window()?;
        let storage = window.local_storage().ok()??;
        let json = storage.get_item("sudoku_state").ok()??;
        match serde_json::from_str(&json) {
            Ok(s) => Some(s),
            Err(e) => {
                web_sys::console::warn_1(&format!("sudoku: failed to load state: {e}").into());
                None
            }
        }
    }
    #[cfg(not(target_arch = "wasm32"))]
    None
}

pub fn save_state(_state: &GameState) {
    #[cfg(target_arch = "wasm32")]
    if let Some(window) = web_sys::window() {
        if let Ok(Some(storage)) = window.local_storage() {
            if let Ok(json) = serde_json::to_string(_state) {
                let _ = storage.set_item("sudoku_state", &json);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::sudoku_engine::Difficulty;

    #[test]
    fn test_new_game_resets_state() {
        let state = AppState::new();
        // Place a number to dirty the state
        state.select_cell(0, 0);
        state.place_number(5);
        // New game
        state.new_game(Difficulty::Hard);
        let s = state.0.get();
        assert_eq!(s.difficulty, Difficulty::Hard);
        assert_eq!(s.timer_seconds, 0);
        assert!(s.history.is_empty());
        assert!(s.redo_stack.is_empty());
        assert!(!s.won);
        // Board should have clues (not empty)
        let clues = s.board.iter().filter(|&&c| c != 0).count();
        assert!(clues >= 20);
        assert!(clues <= 45);
    }

    #[test]
    fn test_select_cell() {
        let state = AppState::new();
        state.select_cell(3, 5);
        assert_eq!(state.0.get().selected, Some((3, 5)));
        state.select_cell(0, 0);
        assert_eq!(state.0.get().selected, Some((0, 0)));
    }

    #[test]
    fn test_place_number_and_undo_redo() {
        let state = AppState::new();
        // Find an empty cell
        let empty = (0..81).find(|&i| state.0.get().board[i] == 0).unwrap();
        let r = empty / 9;
        let c = empty % 9;
        let old_val = state.0.get().get(r, c);
        assert_eq!(old_val, 0);

        state.select_cell(r, c);
        state.place_number(3);
        assert_eq!(state.0.get().get(r, c), 3);
        assert_eq!(state.0.get().history.len(), 1);

        state.undo();
        assert_eq!(state.0.get().get(r, c), 0);
        assert_eq!(state.0.get().redo_stack.len(), 1);

        state.redo();
        assert_eq!(state.0.get().get(r, c), 3);
        assert_eq!(state.0.get().redo_stack.len(), 0);
    }

    #[test]
    fn test_note_mode_toggle() {
        let state = AppState::new();
        assert!(!state.0.get().note_mode);
        state.toggle_note_mode();
        assert!(state.0.get().note_mode);
        state.toggle_note_mode();
        assert!(!state.0.get().note_mode);
    }

    #[test]
    fn test_note_placement() {
        let state = AppState::new();
        // Find an empty cell
        let i = (0..81).find(|&i| state.0.get().board[i] == 0).unwrap();
        let r = i / 9;
        let c = i % 9;
        state.toggle_note_mode();
        state.select_cell(r, c);
        state.place_number(5);
        assert_eq!(state.0.get().notes[i], 1 << 4); // bit 4 for number 5
        // Toggle same note off
        state.place_number(5);
        assert_eq!(state.0.get().notes[i], 0);
    }

    #[test]
    fn test_hint_fills_one_cell() {
        let state = AppState::new();
        let before = state.0.get().board;
        state.hint();
        let after = state.0.get().board;
        // At least one cell should have changed
        let changed = (0..81).filter(|&i| before[i] != after[i]).count();
        assert!(changed >= 1);
        // Hint should place correct value
        for i in 0..81 {
            if after[i] != 0 && before[i] == 0 {
                assert_eq!(after[i], state.0.get().solution[i]);
            }
        }
    }

    #[test]
    fn test_pause_toggle() {
        let state = AppState::new();
        assert!(!state.0.get().paused);
        state.toggle_pause();
        assert!(state.0.get().paused);
        state.toggle_pause();
        assert!(!state.0.get().paused);
    }

    #[test]
    fn test_timer_ticks_only_when_unpaused() {
        let state = AppState::new();
        assert_eq!(state.0.get().timer_seconds, 0);
        state.tick_timer();
        assert_eq!(state.0.get().timer_seconds, 1);
        state.toggle_pause();
        state.tick_timer();
        assert_eq!(state.0.get().timer_seconds, 1); // unchanged
        state.toggle_pause();
        state.tick_timer();
        assert_eq!(state.0.get().timer_seconds, 2);
    }

    #[test]
    fn test_cannot_edit_given_cell() {
        let state = AppState::new();
        // Find a given cell
        if let Some(i) = (0..81).find(|&i| state.0.get().is_given(i / 9, i % 9)) {
            let r = i / 9;
            let c = i % 9;
            let original = state.0.get().get(r, c);
            state.select_cell(r, c);
            state.place_number(if original == 1 { 2 } else { 1 });
            // Should not change
            assert_eq!(state.0.get().get(r, c), original);
        }
    }
}
