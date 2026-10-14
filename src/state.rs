use crate::i18n::Message;
// ponytail: single RwSignal<GameState>. ceiling: 81 cells re-render on any change. upgrade: per-cell signals if frame drops on low-end devices.

use crate::game_history::{evictions, HistoryEntry};
use crate::serde_helpers::{u16_81, u8_81};
use crate::sharing::SharedGame;
use crate::sudoku_engine::{self, Difficulty};
use leptos::prelude::*;
use serde::{Deserialize, Serialize};
use wasm_bindgen::prelude::*;

#[derive(Clone, Copy, PartialEq, Eq, Debug, Serialize, Deserialize, Default)]
pub enum DropAvailability {
    DropOnly,
    NoDrop,
    #[default]
    Both,
}

const fn yes() -> bool {
    true
}
const fn empty_givens() -> [u8; 81] {
    [0; 81]
}

#[derive(Clone, Copy, PartialEq, Eq, Debug, Serialize, Deserialize, Default)]
pub enum SoundType {
    Beep,
    #[default]
    Explosion,
    None,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct HighlightSettings {
    pub selected_shading: u8,
    pub matching_shading: u8,
    pub available_shading: u8,
    pub completed_contrast: u8,
    pub dots: bool,
    pub stripes: bool,
}

impl Default for HighlightSettings {
    fn default() -> Self {
        Self {
            selected_shading: 20,
            matching_shading: 20,
            available_shading: 100,
            completed_contrast: 100,
            dots: true,
            stripes: true,
        }
    }
}

/// Player preferences; candidates always come from a simple row/column/box scan.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct DominoSettings {
    pub initial_delay_ms: u32,
    pub acceleration_percent: u32,
    pub minimum_delay_ms: u32,
    pub empty_cell_threshold: u32,
}

impl Default for DominoSettings {
    fn default() -> Self {
        Self {
            initial_delay_ms: 600,
            acceleration_percent: 20,
            minimum_delay_ms: 100,
            empty_cell_threshold: 10,
        }
    }
}

impl DominoSettings {
    fn normalize(&mut self) {
        self.initial_delay_ms = self.initial_delay_ms.clamp(100, 2000);
        self.acceleration_percent = self.acceleration_percent.min(50);
        self.minimum_delay_ms = self.minimum_delay_ms.clamp(50, self.initial_delay_ms);
        self.empty_cell_threshold = self.empty_cell_threshold.min(81);
    }

    #[cfg_attr(not(target_arch = "wasm32"), allow(dead_code))]
    fn next_delay(&self, delay_ms: u32) -> u32 {
        ((delay_ms * (100 - self.acceleration_percent) + 50) / 100).max(self.minimum_delay_ms)
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct GameState {
    #[serde(default)]
    pub game_id: String,
    #[serde(with = "u8_81")]
    pub board: [u8; 81],
    #[serde(with = "u8_81", default = "empty_givens")]
    pub givens: [u8; 81],
    #[serde(with = "u8_81")]
    pub solution: [u8; 81],
    #[serde(with = "u16_81")]
    pub notes: [u16; 81],
    #[serde(with = "u8_81", default = "empty_givens")]
    pub cell_colors: [u8; 81],
    pub difficulty: Difficulty,
    #[serde(default)]
    pub requested_difficulty: Option<Difficulty>,
    #[serde(default)]
    pub rating: Option<sudoku_engine::Rating>,
    pub seed: u64,
    #[serde(with = "u8_81")]
    pub hinted: [u8; 81],
    pub error_count: u32,
    pub note_mode: bool,
    #[serde(default)]
    pub drop_mode: bool,
    #[serde(default)]
    pub drop_availability: DropAvailability,
    /// Transient painting tool: 1–4 toggle a preset color.
    #[serde(skip)]
    pub paint_tool: Option<u8>,
    #[serde(default)]
    pub drop_number: Option<u8>,
    #[serde(default = "yes")]
    pub drop_pick_solved: bool,
    #[serde(default = "yes")]
    pub undo_enabled: bool,
    #[serde(default = "yes")]
    pub auto_notes_enabled: bool,
    #[serde(default = "yes")]
    pub hint_enabled: bool,
    #[serde(default = "yes")]
    pub domino_enabled: bool,
    #[serde(default)]
    pub sound_type: SoundType,
    #[serde(default)]
    pub effects: crate::effects::EffectsSettings,
    #[serde(default)]
    pub highlights: HighlightSettings,
    #[serde(default)]
    pub domino: DominoSettings,
    #[serde(skip)]
    pub domino_gen: u32,
    #[serde(skip)]
    pub just_filled: Option<(usize, usize)>,
    pub selected: Option<(usize, usize)>,
    pub timer_seconds: u32,
    pub paused: bool,
    pub won: bool,
    #[serde(skip)]
    #[cfg_attr(not(target_arch = "wasm32"), allow(dead_code))]
    pub celebrated: bool,
    pub history: Vec<Snapshot>,
    pub redo_stack: Vec<Snapshot>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Snapshot {
    #[serde(with = "u8_81")]
    pub board: [u8; 81],
    #[serde(with = "u16_81")]
    pub notes: [u16; 81],
    #[serde(with = "u8_81", default = "empty_givens")]
    pub cell_colors: [u8; 81],
}

// Selection history is independent of game saves, archives and shared links.
fn generate_board(difficulty: Difficulty) -> sudoku_engine::Board {
    #[cfg(target_arch = "wasm32")]
    let storage = web_sys::window().and_then(|w| w.local_storage().ok().flatten());
    #[cfg(target_arch = "wasm32")]
    let mut progress: sudoku_engine::BankProgress = storage
        .as_ref()
        .and_then(|s| s.get_item("sudoku_bank_v2").ok().flatten())
        .and_then(|s| serde_json::from_str(&s).ok())
        .unwrap_or_default();
    #[cfg(not(target_arch = "wasm32"))]
    let mut progress = sudoku_engine::BankProgress::default();
    let board = sudoku_engine::generate_with_progress(difficulty, &mut progress);
    #[cfg(target_arch = "wasm32")]
    if let Some(storage) = storage {
        if let Ok(json) = serde_json::to_string(&progress) {
            let _ = storage.set_item("sudoku_bank_v2", &json);
        }
    }
    board
}

impl Default for GameState {
    fn default() -> Self {
        let board = generate_board(Difficulty::Easy);
        Self {
            game_id: new_game_id(),
            board: board.cells,
            givens: board.cells,
            solution: board.solution,
            notes: [0u16; 81],
            cell_colors: [0; 81],
            difficulty: board.difficulty,
            requested_difficulty: Some(Difficulty::Easy),
            rating: board.rating,
            seed: board.seed,
            hinted: [0u8; 81],
            error_count: 0,
            note_mode: false,
            drop_mode: false,
            drop_availability: DropAvailability::Both,
            paint_tool: None,
            drop_number: None,
            drop_pick_solved: true,
            undo_enabled: true,
            auto_notes_enabled: true,
            hint_enabled: true,
            domino_enabled: true,
            domino: DominoSettings::default(),
            domino_gen: 0,
            sound_type: SoundType::default(),
            effects: crate::effects::EffectsSettings::default(),
            highlights: HighlightSettings::default(),
            just_filled: None,
            selected: None,
            timer_seconds: 0,
            paused: false,
            won: false,
            celebrated: false,
            history: Vec::new(),
            redo_stack: Vec::new(),
        }
    }
}

impl GameState {
    pub fn start_new_game(&mut self, difficulty: Difficulty) {
        let board = generate_board(difficulty);
        let s = self;
        *s = GameState {
            game_id: new_game_id(),
            board: board.cells,
            givens: board.cells,
            solution: board.solution,
            notes: [0u16; 81],
            cell_colors: [0; 81],
            difficulty: board.difficulty,
            requested_difficulty: Some(difficulty),
            rating: board.rating,
            seed: board.seed,
            hinted: [0u8; 81],
            error_count: 0,
            note_mode: s.note_mode,
            drop_mode: s.drop_mode,
            drop_availability: s.drop_availability,
            paint_tool: None,
            drop_number: None,
            drop_pick_solved: s.drop_pick_solved,
            undo_enabled: s.undo_enabled,
            auto_notes_enabled: s.auto_notes_enabled,
            hint_enabled: s.hint_enabled,
            domino_enabled: s.domino_enabled,
            domino: s.domino,
            domino_gen: s.domino_gen.wrapping_add(1),
            sound_type: s.sound_type,
            effects: s.effects,
            highlights: s.highlights,
            just_filled: None,
            selected: None,
            timer_seconds: 0,
            paused: false,
            won: false,
            celebrated: false,
            history: Vec::new(),
            redo_stack: Vec::new(),
        };
    }

    pub fn freeze_for_replacement(&mut self) {
        self.cancel_domino();
        self.paused = true;
    }

    /// Replace only game data, preserving every recipient preference.
    pub fn import_shared(&mut self, game: &crate::sharing::SharedGame) {
        self.cancel_domino();
        self.board = game.board;
        self.givens = game.givens;
        self.solution = game.solution;
        self.notes = game.notes;
        self.cell_colors = game.colors;
        self.hinted = game.hinted;
        self.difficulty = game.difficulty;
        self.requested_difficulty = game.requested;
        self.rating = game.rating.clone();
        self.seed = game.seed;
        self.timer_seconds = game.seconds;
        self.error_count = game.errors;
        self.history.clear();
        self.redo_stack.clear();
        self.selected = None;
        self.drop_number = None;
        self.paint_tool = None;
        self.just_filled = None;
        self.paused = true;
        self.won = self.board == self.solution;
        self.celebrated = self.won;
    }

    fn idx(r: usize, c: usize) -> usize {
        r * 9 + c
    }

    pub fn get(&self, r: usize, c: usize) -> u8 {
        self.board[Self::idx(r, c)]
    }

    /// Completion counts only correct occurrences, never incorrect duplicates.
    pub fn number_is_solved(&self, value: u8) -> bool {
        (1..=9).contains(&value)
            && self
                .board
                .iter()
                .zip(self.solution.iter())
                .filter(|(cell, solution)| **cell == value && cell == solution)
                .count()
                == 9
    }

    /// The digit being inspected; Drop mode owns the preview while enabled.
    pub fn active_number(&self) -> Option<u8> {
        let number = if self.drop_mode {
            self.drop_number
        } else {
            self.selected.map(|(r, c)| self.get(r, c))
        };
        number.filter(|n| (1..=9).contains(n))
    }

    /// Rule-based preview for empty cells. Without a digit, previews selected
    /// row/column/box membership instead. Never consults the solution.
    pub fn placement_available(&self, r: usize, c: usize) -> Option<bool> {
        if self.get(r, c) != 0 {
            return None;
        }
        self.active_number()
            .map(|n| sudoku_engine::is_valid_move(&self.board, r, c, n))
            .or_else(|| {
                if self.drop_mode {
                    return None;
                }
                self.selected
                    .map(|(sr, sc)| !(sr == r || sc == c || (sr / 3 == r / 3 && sc / 3 == c / 3)))
            })
    }

    /// Selected-cell blockers take precedence where both sources overlap.
    /// Without a selected occurrence (e.g. keypad Drop), all blockers are matching.
    pub fn placement_blocker(&self, r: usize, c: usize) -> Option<&'static str> {
        if self.placement_available(r, c) != Some(false) {
            return None;
        }
        let number = self.active_number();
        if !self.drop_mode
            && self.selected.is_some_and(|(sr, sc)| {
                ((!self.drop_mode && number.is_none()) || number == Some(self.get(sr, sc)))
                    && (sr == r || sc == c || (sr / 3 == r / 3 && sc / 3 == c / 3))
            })
        {
            Some("selected")
        } else {
            Some("matching")
        }
    }

    pub fn is_given(&self, r: usize, c: usize) -> bool {
        self.givens[Self::idx(r, c)] != 0
    }

    pub fn is_locked(&self, r: usize, c: usize) -> bool {
        self.is_given(r, c) || self.is_hinted(r, c)
    }

    pub fn is_hinted(&self, r: usize, c: usize) -> bool {
        self.hinted[Self::idx(r, c)] != 0
    }

    fn cancel_domino(&mut self) {
        self.domino_gen = self.domino_gen.wrapping_add(1);
    }

    fn domino_eligible(&self) -> bool {
        let empty = self.board.iter().filter(|&&v| v == 0).count() as u32;
        self.domino_enabled
            && !self.paused
            && !self.won
            && empty > 0
            && (self.domino.empty_cell_threshold == 0 || empty <= self.domino.empty_cell_threshold)
    }

    fn clear_related_notes(&mut self, row: usize, col: usize, value: u8) {
        let mask = !(1 << (value - 1));
        for i in 0..81 {
            let (r, c) = (i / 9, i % 9);
            if r == row || c == col || (r / 3 == row / 3 && c / 3 == col / 3) {
                self.notes[i] &= mask;
            }
        }
    }

    /// Apply a player entry; both normal and Drop mode use this transition.
    fn enter_number(&mut self, row: usize, col: usize, value: u8) -> bool {
        if self.is_locked(row, col) || self.won || self.paused || value > 9 {
            return false;
        }
        if self.drop_mode && !self.note_mode && self.number_is_solved(value) {
            return false;
        }
        self.push_snapshot();
        let i = Self::idx(row, col);
        if self.note_mode {
            if value == 0 {
                self.notes[i] = 0;
            } else {
                self.notes[i] ^= 1 << (value - 1);
            }
            return false;
        }
        self.board[i] = value;
        self.notes[i] = 0;
        if value == 0 {
            return false;
        }
        if value != self.solution[i] {
            self.error_count = self.error_count.saturating_add(1);
            return false;
        }
        self.clear_related_notes(row, col, value);
        self.just_filled = Some((row, col));
        self.won = self.board == self.solution;
        #[cfg(target_arch = "wasm32")]
        crate::audio::play(self.sound_type, self.effects.volume, false);
        self.domino_eligible()
    }

    /// Fill only naked singles from current row/column/box constraints.
    /// The solution check prevents propagating an incorrect player entry.
    #[cfg_attr(not(target_arch = "wasm32"), allow(dead_code))]
    fn domino_step(&mut self, generation: u32) -> bool {
        if self.domino_gen != generation || !self.domino_eligible() {
            return false;
        }
        self.just_filled = None;
        for i in 0..81 {
            if self.board[i] != 0 {
                continue;
            }
            let (row, col) = (i / 9, i % 9);
            let candidates = sudoku_engine::candidates(&self.board, row, col);
            if candidates.len() != 1 || candidates[0] != self.solution[i] {
                continue;
            }
            let value = candidates[0];
            self.board[i] = value;
            self.notes[i] = 0;
            self.clear_related_notes(row, col, value);
            self.just_filled = Some((row, col));
            self.won = self.board == self.solution;
            return true;
        }
        false
    }

    fn paint_cell(&mut self, row: usize, col: usize, color: u8) {
        if row >= 9 || col >= 9 || !(1..=4).contains(&color) || self.paused || self.won {
            return;
        }
        let i = Self::idx(row, col);
        self.push_snapshot();
        self.cell_colors[i] = if self.cell_colors[i] == color {
            0
        } else {
            color
        };
    }

    pub fn push_snapshot(&mut self) {
        self.cancel_domino();
        self.history.push(Snapshot {
            board: self.board,
            notes: self.notes,
            cell_colors: self.cell_colors,
        });
        self.redo_stack.clear();
    }

    // Legacy migration can mark correct player entries as givens. A historical
    // snapshot that changes one proves it was editable; release that lock.
    // Genuine original clues are unchanged in every valid history snapshot.
    fn reconcile_restored_givens(&mut self) {
        for (given, value) in self.givens.iter_mut().zip(self.board) {
            if *given != value {
                *given = 0;
            }
        }
    }

    pub fn undo(&mut self) {
        self.cancel_domino();
        if let Some(snap) = self.history.pop() {
            self.redo_stack.push(Snapshot {
                board: self.board,
                notes: self.notes,
                cell_colors: self.cell_colors,
            });
            self.board = snap.board;
            self.notes = snap.notes;
            self.cell_colors = snap.cell_colors;
            self.reconcile_restored_givens();
            self.won = self.board == self.solution;
            self.celebrated = false;
            self.just_filled = None;
        }
    }

    pub fn redo(&mut self) {
        self.cancel_domino();
        if let Some(snap) = self.redo_stack.pop() {
            self.history.push(Snapshot {
                board: self.board,
                notes: self.notes,
                cell_colors: self.cell_colors,
            });
            self.board = snap.board;
            self.notes = snap.notes;
            self.cell_colors = snap.cell_colors;
            self.reconcile_restored_givens();
            self.won = self.board == self.solution;
            self.celebrated = false;
            self.just_filled = None;
        }
    }
}

#[derive(Clone, Copy)]
pub struct AppState(pub RwSignal<GameState>);

impl AppState {
    pub fn new() -> Self {
        // ponytail: localStorage load with console.warn on failure. ceiling: user doesn't see console. upgrade: toast notification if players report lost progress.
        let mut saved = load_state().unwrap_or_default();
        if saved.game_id.is_empty() {
            saved.game_id = new_game_id();
        }
        AppState(RwSignal::new(saved))
    }

    #[cfg(test)]
    pub fn new_game(&self, difficulty: Difficulty) {
        self.0.update(|s| s.start_new_game(difficulty));
    }

    pub fn select_cell(&self, r: usize, c: usize) {
        let mut trigger_domino = false;
        self.0.update(|s| {
            if s.drop_mode {
                if let Some(color) = s.paint_tool {
                    s.paint_cell(r, c, color);
                    return;
                }
                let value = s.get(r, c);
                if s.drop_pick_solved
                    && !s.note_mode
                    && !s.paused
                    && !s.won
                    && value != 0
                    && value == s.solution[GameState::idx(r, c)]
                {
                    s.drop_number = Some(value);
                    s.selected = None;
                } else if let Some(number) = s.drop_number {
                    trigger_domino = s.enter_number(r, c, number);
                }
                return;
            }

            s.selected = if s.selected == Some((r, c)) {
                None
            } else {
                Some((r, c))
            };
        });
        self.finish_entry(trigger_domino);
    }

    pub fn select_paint_tool(&self, color: u8) {
        self.0.update(|s| {
            if !(1..=4).contains(&color) || s.paused || s.won {
                return;
            }
            if s.drop_mode {
                s.cancel_domino();
                s.note_mode = true;
                s.paint_tool = if s.paint_tool == Some(color) {
                    None
                } else {
                    Some(color)
                };
            } else if let Some((row, col)) = s.selected {
                s.note_mode = true;
                s.paint_cell(row, col, color);
            }
        });
    }

    pub fn clear_colors(&self) {
        self.0.update(|s| {
            if s.paused || s.won || s.cell_colors.iter().all(|&c| c == 0) {
                return;
            }
            s.push_snapshot();
            s.cell_colors = [0; 81];
        });
    }

    pub fn toggle_drop_mode(&self) {
        self.0.update(|s| {
            if s.drop_availability != DropAvailability::Both {
                return;
            }
            s.paint_tool = None;
            s.drop_mode = !s.drop_mode;
            s.drop_number = None;
            s.selected = None;
        });
    }

    pub fn set_drop_availability(&self, availability: DropAvailability) {
        self.0.update(|s| {
            if s.drop_availability == availability {
                return;
            }
            s.cancel_domino();
            s.drop_availability = availability;
            s.drop_mode = match availability {
                DropAvailability::DropOnly => true,
                DropAvailability::NoDrop => false,
                DropAvailability::Both => s.drop_mode,
            };
            s.drop_number = None;
            s.paint_tool = None;
            s.selected = None;
        });
    }

    pub fn toggle_drop_pick_solved(&self) {
        self.0.update(|s| s.drop_pick_solved = !s.drop_pick_solved);
    }

    pub fn select_drop_number(&self, v: u8) {
        self.0.update(|s| {
            if s.drop_number == Some(v) {
                s.drop_number = None;
            } else {
                s.drop_number = Some(v);
            }
        });
    }

    pub fn place_number(&self, value: u8) {
        let mut trigger_domino = false;
        self.0.update(|s| {
            s.paint_tool = None;
            if let Some((row, col)) = s.selected {
                trigger_domino = s.enter_number(row, col, value);
            }
        });
        self.finish_entry(trigger_domino);
    }

    fn finish_entry(&self, trigger_domino: bool) {
        #[cfg(target_arch = "wasm32")]
        {
            Self::celebrate(self.0);
            if trigger_domino {
                let (delay, generation) =
                    self.0.with(|s| (s.domino.initial_delay_ms, s.domino_gen));
                Self::domino_chain(self.0, delay, generation);
            }
        }
        #[cfg(not(target_arch = "wasm32"))]
        let _ = trigger_domino;
    }

    #[cfg(target_arch = "wasm32")]
    fn celebrate(signal: RwSignal<GameState>) {
        if signal.with(|s| s.won && !s.celebrated) {
            let (sound, effects) = signal.with(|s| (s.sound_type, s.effects));
            signal.update(|s| s.celebrated = true);
            crate::effects::celebrate(sound, effects);
        }
    }

    #[cfg(target_arch = "wasm32")]
    fn domino_chain(signal: RwSignal<GameState>, delay_ms: u32, generation: u32) {
        set_timeout(
            move || {
                let mut filled = false;
                signal.update(|s| filled = s.domino_step(generation));
                if filled {
                    signal.with(|s| crate::audio::play(s.sound_type, s.effects.volume, false));
                    Self::celebrate(signal);
                    if signal.with(|s| s.domino_eligible() && s.domino_gen == generation) {
                        let next = signal.with(|s| s.domino.next_delay(delay_ms));
                        Self::domino_chain(signal, next, generation);
                    }
                }
            },
            std::time::Duration::from_millis(delay_ms as u64),
        );
    }

    pub fn undo(&self) {
        self.0.update(|s| s.undo());
    }

    pub fn redo(&self) {
        let was_won = self.0.get_untracked().won;
        self.0.update(|s| s.redo());
        if !was_won {
            #[cfg(target_arch = "wasm32")]
            Self::celebrate(self.0);
        }
    }

    pub fn toggle_note_mode(&self) {
        self.0.update(|s| {
            if s.paint_tool.take().is_some() {
                s.note_mode = true;
            } else {
                s.note_mode = !s.note_mode;
            }
        });
    }

    pub fn toggle_undo(&self) {
        self.0.update(|s| s.undo_enabled = !s.undo_enabled);
    }

    pub fn toggle_auto_notes(&self) {
        self.0
            .update(|s| s.auto_notes_enabled = !s.auto_notes_enabled);
    }

    pub fn toggle_hint(&self) {
        self.0.update(|s| s.hint_enabled = !s.hint_enabled);
    }

    pub fn toggle_domino(&self) {
        self.0.update(|s| {
            s.cancel_domino();
            s.domino_enabled = !s.domino_enabled;
        });
    }

    pub fn set_domino_settings(&self, mut settings: DominoSettings) {
        settings.normalize();
        self.0.update(|s| {
            s.cancel_domino();
            s.domino = settings;
        });
    }

    pub fn reset_config(&self) {
        self.0.update(|s| {
            s.drop_availability = DropAvailability::Both;
            s.drop_pick_solved = true;
            s.undo_enabled = true;
            s.auto_notes_enabled = true;
            s.hint_enabled = true;
            s.cancel_domino();
            s.domino_enabled = true;
            s.domino = DominoSettings::default();
            s.sound_type = SoundType::default();
            s.effects = crate::effects::EffectsSettings::default();
            s.highlights = HighlightSettings::default();
        });
    }

    pub fn hint(&self) {
        self.0.update(|s| {
            if !s.hint_enabled || s.won {
                return;
            }
            s.cancel_domino();
            // Hint: pick the unsolved cell with fewest candidates.
            // Deterministic pseudo-random traversal order from puzzle seed.
            let mut order: Vec<usize> = (0..81).collect();
            {
                let mut r = s.seed;
                for i in (1..order.len()).rev() {
                    r ^= r << 13;
                    r ^= r >> 7;
                    r ^= r << 17;
                    let j = (r as usize) % (i + 1);
                    order.swap(i, j);
                }
            }
            let mut best: Option<(usize, u32)> = None;
            for &i in &order {
                if s.board[i] != s.solution[i] {
                    let r = i / 9;
                    let c = i % 9;
                    let cands = (1..=9)
                        .filter(|&v| crate::sudoku_engine::is_valid_move(&s.board, r, c, v))
                        .count() as u32;
                    if cands > 0 && best.is_none_or(|(_, n)| cands < n) {
                        best = Some((i, cands));
                    }
                }
            }
            if let Some((i, _)) = best {
                s.history.clear();
                s.redo_stack.clear();
                let v = s.solution[i];
                s.hinted[i] = 1;
                s.board[i] = v;
                s.notes[i] = 0;
                // Clear notes of this number from row/col/box
                let r = i / 9;
                let c = i % 9;
                let bit = !(1 << (v - 1));
                for j in 0..9 {
                    s.notes[GameState::idx(r, j)] &= bit;
                    s.notes[GameState::idx(j, c)] &= bit;
                }
                let br = (r / 3) * 3;
                let bc = (c / 3) * 3;
                for rr in br..br + 3 {
                    for cc in bc..bc + 3 {
                        s.notes[GameState::idx(rr, cc)] &= bit;
                    }
                }
                if s.board == s.solution {
                    s.won = true;
                }
            }
        });
        #[cfg(target_arch = "wasm32")]
        Self::celebrate(self.0);
    }

    pub fn tick_timer(&self) {
        self.0.update(|s| {
            if !s.paused && !s.won {
                s.timer_seconds = s.timer_seconds.saturating_add(1);
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
        self.0.update(|s| {
            s.cancel_domino();
            s.paused = !s.paused;
        });
    }

    pub fn set_effect_volume(&self, volume: u8) {
        self.0.update(|s| {
            s.effects.volume = volume;
            s.effects.normalize();
        });
    }
    pub fn set_balloon_count(&self, count: u8) {
        self.0.update(|s| {
            s.effects.balloons = count;
            s.effects.normalize();
        });
    }
    pub fn preview_sound(&self) {
        #[cfg(target_arch = "wasm32")]
        self.0
            .with_untracked(|s| crate::audio::play(s.sound_type, s.effects.volume, false));
    }

    pub fn set_sound(&self, sound: SoundType) {
        self.0.update(|s| s.sound_type = sound);
    }
}

// Legacy saves cannot reliably distinguish original clues from correct entries.
// Preserve their locked cells, then retain provenance for subsequent player moves.
#[cfg(any(target_arch = "wasm32", test))]
pub(crate) fn decode_saved_state(json: &str) -> Result<GameState, serde_json::Error> {
    let value: serde_json::Value = serde_json::from_str(json)?;
    let legacy = value.get("givens").is_none();
    let mut state: GameState = serde_json::from_value(value)?;
    for colors in std::iter::once(&mut state.cell_colors)
        .chain(state.history.iter_mut().map(|s| &mut s.cell_colors))
        .chain(state.redo_stack.iter_mut().map(|s| &mut s.cell_colors))
    {
        for color in colors {
            if *color > 4 {
                *color = 0;
            }
        }
    }
    state.drop_mode = match state.drop_availability {
        DropAvailability::DropOnly => true,
        DropAvailability::NoDrop => false,
        DropAvailability::Both => state.drop_mode,
    };
    state.drop_number = state
        .drop_number
        .filter(|n| state.drop_mode && (1..=9).contains(n));
    if state.drop_mode {
        state.selected = None;
    }
    // A completed save has already been celebrated, including shared imports.
    state.celebrated = state.won;
    state.domino.normalize();
    state.effects.normalize();
    state.highlights.selected_shading = state.highlights.selected_shading.min(100);
    state.highlights.matching_shading = state.highlights.matching_shading.min(100);
    state.highlights.available_shading = state.highlights.available_shading.min(100);
    state.highlights.completed_contrast = state.highlights.completed_contrast.min(100);
    if legacy {
        for i in 0..81 {
            if state.board[i] != 0 && state.board[i] == state.solution[i] {
                state.givens[i] = state.board[i];
            }
        }
    }
    Ok(state)
}

fn load_state() -> Option<GameState> {
    #[cfg(target_arch = "wasm32")]
    {
        if let Some(state) = crate::updates::take_resume() {
            return Some(state);
        }
        let window = web_sys::window()?;
        let storage = window.local_storage().ok()??;
        let json = storage.get_item("sudoku_state").ok()??;
        match decode_saved_state(&json) {
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

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = __sudokuHistory, js_name = list, catch)]
    async fn store_list() -> Result<JsValue, JsValue>;
    #[wasm_bindgen(js_namespace = __sudokuHistory, js_name = save, catch)]
    async fn store_save(json: &str, activate: bool) -> Result<JsValue, JsValue>;
    #[wasm_bindgen(js_namespace = __sudokuHistory, js_name = pin, catch)]
    async fn store_pin(id: &str, pinned: bool) -> Result<JsValue, JsValue>;
    #[wasm_bindgen(js_namespace = __sudokuHistory, js_name = prune, catch)]
    async fn store_prune(ids: &str, active: &str) -> Result<JsValue, JsValue>;
}

#[derive(Clone, Copy)]
pub struct GameHistory {
    pub entries: RwSignal<Vec<HistoryEntry>>,
    pub ready: RwSignal<bool>,
    pub busy: RwSignal<bool>,
    pub error: RwSignal<Option<crate::i18n::Notice>>,
}
impl GameHistory {
    pub fn new(state: AppState) -> Self {
        let value = Self {
            entries: RwSignal::new(Vec::new()),
            ready: RwSignal::new(false),
            busy: RwSignal::new(false),
            error: RwSignal::new(None),
        };
        value.refresh(state);
        value
    }
    async fn read(&self) -> Result<Vec<HistoryEntry>, Message> {
        let result = store_list()
            .await
            .map_err(|_| Message::HistoryAccessFailed)?;
        let json = result.as_string().ok_or(Message::InvalidHistory)?;
        let values: Vec<serde_json::Value> =
            serde_json::from_str(&json).map_err(|_| Message::InvalidHistory)?;
        let mut entries = Vec::new();
        for value in values {
            match serde_json::from_value::<HistoryEntry>(value) {
                Ok(entry) if entry.valid() => entries.push(entry),
                _ => self.error.set(Some(Message::InvalidHistoryRecords.into())),
            }
        }
        Ok(entries)
    }
    pub(crate) async fn save_entry(entry: &HistoryEntry, activate: bool) -> Result<(), Message> {
        let json = serde_json::to_string(entry).map_err(|_| Message::HistorySerializeFailed)?;
        store_save(&json, activate)
            .await
            .map_err(|_| Message::HistorySaveFailed)?;
        Ok(())
    }
    async fn cleanup(&self, state: AppState) -> Result<(), Message> {
        let entries = self.read().await?;
        self.entries.set(entries.clone());
        let active = state.0.get_untracked().game_id;
        let ids = evictions(&entries, &active);
        if !ids.is_empty() {
            let candidates: Vec<_> = entries
                .iter()
                .filter(|e| ids.contains(&e.id))
                .map(|e| serde_json::json!({"id":e.id,"order":e.order}))
                .collect();
            store_prune(&serde_json::to_string(&candidates).unwrap(), &active)
                .await
                .map_err(|_| Message::HistoryCleanupFailed)?;
        }
        self.entries.set(self.read().await?);
        Ok(())
    }
    pub fn refresh(self, state: AppState) {
        if self.busy.get_untracked() {
            return;
        }
        self.busy.set(true);
        self.error.set(None);
        leptos::task::spawn_local(async move {
            let result = async {
                Self::save_entry(&HistoryEntry::capture(&state.0.get_untracked()), false).await?;
                self.ready.set(true);
                self.cleanup(state).await
            }
            .await;
            if let Err(error) = result {
                self.error.set(Some(error.into()));
            }
            self.busy.set(false);
        });
    }
    pub fn new_game(self, state: AppState, difficulty: Difficulty) {
        let mut candidate = state.0.get_untracked();
        candidate.start_new_game(difficulty);
        self.replace(state, candidate, None, |_| {});
    }
    pub fn import(self, state: AppState, game: SharedGame, done: impl FnOnce(bool) + 'static) {
        let mut candidate = state.0.get_untracked();
        candidate.import_shared(&game);
        candidate.game_id = new_game_id();
        self.replace(state, candidate, None, done);
    }
    pub fn load(
        self,
        state: AppState,
        entry: HistoryEntry,
        initial: bool,
        done: impl FnOnce(bool) + 'static,
    ) {
        if !entry.valid() {
            self.error.set(Some(Message::InvalidHistoryGame.into()));
            done(false);
            return;
        }
        let candidate = entry.restore(&state.0.get_untracked(), initial);
        // Retain the previous attempt when restarting even at the ten-game limit.
        self.replace(
            state,
            candidate,
            if initial { Some(entry) } else { None },
            done,
        );
    }
    fn replace(
        self,
        state: AppState,
        candidate: GameState,
        source_attempt: Option<HistoryEntry>,
        done: impl FnOnce(bool) + 'static,
    ) {
        if self.busy.get_untracked() || !self.ready.get_untracked() {
            self.error.set(Some(Message::HistoryNotReady.into()));
            done(false);
            return;
        }
        let outgoing = state.0.get_untracked();
        self.busy.set(true);
        self.error.set(None);
        state.0.update(|s| s.freeze_for_replacement());
        leptos::task::spawn_local(async move {
            let saved = async {
                Self::save_entry(&HistoryEntry::capture(&outgoing), false).await?;
                if let Some(source) = source_attempt {
                    Self::save_entry(&source, true).await?;
                }
                Ok(())
            }
            .await;
            if let Err(error) = saved.and_then(|_| try_save_state(&candidate)) {
                state.0.update(|s| s.paused = outgoing.paused);
                self.error.set(Some(error.into()));
                self.busy.set(false);
                done(false);
                return;
            }
            // Keep the invalidation counter from the frozen outgoing game.
            let mut candidate = candidate;
            candidate.domino_gen = state.0.get_untracked().domino_gen;
            state.0.set(candidate.clone());
            let result = async {
                Self::save_entry(&HistoryEntry::capture(&candidate), true).await?;
                self.cleanup(state).await
            }
            .await;
            if let Err(error) = result {
                self.error.set(Some(crate::i18n::Notice {
                    message: error,
                    previous_saved: true,
                }));
            }
            self.busy.set(false);
            done(true);
        });
    }
    pub fn set_pinned(self, state: AppState, id: String, pinned: bool) {
        if self.busy.get_untracked() {
            return;
        }
        self.busy.set(true);
        self.error.set(None);
        leptos::task::spawn_local(async move {
            let result = async {
                if id == state.0.get_untracked().game_id {
                    Self::save_entry(&HistoryEntry::capture(&state.0.get_untracked()), false)
                        .await?;
                }
                store_pin(&id, pinned)
                    .await
                    .map_err(|_| Message::PinFailed)?;
                self.cleanup(state).await
            }
            .await;
            if let Err(error) = result {
                self.error.set(Some(error.into()));
            }
            self.busy.set(false);
            set_timeout(
                move || {
                    use wasm_bindgen::JsCast;
                    if let Some(element) =
                        web_sys::window().and_then(|w| w.document()).and_then(|d| {
                            d.get_element_by_id(&format!("pin-{id}"))
                                .or_else(|| d.query_selector(".history-heading a").ok().flatten())
                        })
                    {
                        if let Some(element) = element.dyn_ref::<web_sys::HtmlElement>() {
                            let _ = element.focus();
                        }
                    }
                },
                std::time::Duration::ZERO,
            );
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::sudoku_engine::Difficulty;

    #[test]
    fn effects_preferences_load_legacy_progress_and_survive_game_changes() {
        let mut saved = domino_fixture();
        saved.push_snapshot();
        saved.redo_stack = saved.history.clone();
        let mut old = serde_json::to_value(&saved).unwrap();
        old.as_object_mut().unwrap().remove("effects");
        let loaded = decode_saved_state(&old.to_string()).unwrap();
        assert_eq!(loaded.effects, crate::effects::EffectsSettings::default());
        assert_eq!(loaded.board, saved.board);
        assert_eq!(loaded.notes, saved.notes);
        assert_eq!(loaded.cell_colors, saved.cell_colors);
        assert_eq!(loaded.history.len(), saved.history.len());
        assert_eq!(loaded.redo_stack.len(), saved.redo_stack.len());
        old["effects"] = serde_json::json!({"volume":255,"balloons":255});
        assert_eq!(
            decode_saved_state(&old.to_string()).unwrap().effects,
            crate::effects::EffectsSettings {
                volume: 100,
                balloons: 100
            }
        );
        let app = AppState(RwSignal::new(loaded));
        app.set_effect_volume(37);
        app.set_balloon_count(100);
        let preferred = app.0.get().effects;
        app.new_game(Difficulty::Easy);
        assert_eq!(app.0.get().effects, preferred);
        let game = crate::sharing::SharedGame::capture(&saved, false);
        app.0.update(|s| s.import_shared(&game));
        assert_eq!(app.0.get().effects, preferred);
        let board = app.0.get().board;
        app.reset_config();
        assert_eq!(
            app.0.get().effects,
            crate::effects::EffectsSettings::default()
        );
        assert_eq!(app.0.get().board, board);
    }

    #[test]
    fn completed_import_reload_does_not_replay_celebration() {
        let mut s = GameState::default();
        s.board = s.solution;
        let shared = crate::sharing::SharedGame::capture(&s, false);
        s.import_shared(&shared);
        let loaded = decode_saved_state(&serde_json::to_string(&s).unwrap()).unwrap();
        assert!(loaded.won && loaded.celebrated && loaded.paused);
        assert_eq!(loaded.board, s.solution);
    }

    #[test]
    fn color_exit_keeps_notes_safe_and_explicit_toggle_restores_numbers() {
        for notes in [false, true] {
            let app = AppState(RwSignal::new(domino_fixture()));
            app.0.update(|s| {
                s.note_mode = notes;
                s.notes = [0; 81];
                s.domino_enabled = false;
            });
            app.select_paint_tool(1); // No target: ignore, including mode changes.
            assert_eq!(app.0.get().note_mode, notes);
            app.select_cell(0, 0);
            app.select_paint_tool(1);
            assert!(app.0.get().note_mode);
            app.toggle_drop_mode();
            assert_eq!(app.0.get().selected, None);
            app.select_drop_number(1);
            app.select_paint_tool(2);
            app.select_paint_tool(3);
            app.select_paint_tool(3); // Same-color exit stays in Notes.
            assert!(app.0.get().note_mode);
            assert_eq!(app.0.get().paint_tool, None);
            let board = app.0.get().board;
            app.select_cell(0, 0);
            assert_eq!(app.0.get().board, board);
            assert_eq!(app.0.get().notes[0], 1);
            app.select_paint_tool(2);
            app.toggle_note_mode(); // First Notes tap exits painting safely.
            assert!(app.0.get().note_mode);
            assert_eq!(app.0.get().paint_tool, None);
            app.toggle_note_mode();
            assert!(!app.0.get().note_mode);
            app.select_cell(0, 0);
            assert_eq!(app.0.get().board[0], 1);
            app.select_paint_tool(2);
            app.toggle_drop_mode();
            assert!(app.0.get().note_mode);
            assert_eq!(app.0.get().selected, None);
        }
    }

    #[test]
    fn repeated_normal_cell_taps_deselect_without_editing_progress() {
        let app = AppState(RwSignal::new(domino_fixture()));
        app.0.update(|s| {
            s.board[1] = 8;
            s.givens[1] = 0;
        });
        for target in [(0, 0), (0, 1), (0, 2)] {
            app.select_cell(target.0, target.1);
            let before = app.0.get();
            app.select_cell(target.0, target.1);
            let after = app.0.get();
            assert_eq!(after.selected, None);
            assert_eq!(after.active_number(), None);
            for r in 0..9 {
                for c in 0..9 {
                    assert_eq!(after.placement_available(r, c), None);
                }
            }
            app.place_number(9);
            app.select_paint_tool(1);
            let ignored = app.0.get();
            assert_eq!(ignored.board, before.board);
            assert_eq!(ignored.notes, before.notes);
            assert_eq!(ignored.cell_colors, before.cell_colors);
            assert_eq!(ignored.history.len(), before.history.len());
        }
    }

    #[test]
    fn drop_saves_default_legacy_fields_and_normalize_digits_without_losing_progress() {
        let mut s = domino_fixture();
        s.note_mode = true;
        s.drop_mode = true;
        s.drop_number = Some(3);
        s.selected = Some((0, 2));
        s.paint_tool = Some(2);
        s.push_snapshot();
        let json = serde_json::to_string(&s).unwrap();
        let loaded = decode_saved_state(&json).unwrap();
        assert!(loaded.drop_mode && loaded.note_mode);
        assert_eq!(loaded.drop_number, Some(3));
        assert_eq!(loaded.selected, None);
        assert_eq!(loaded.paint_tool, None);
        assert_eq!(loaded.board, s.board);
        assert_eq!(loaded.notes, s.notes);
        assert_eq!(loaded.history.len(), s.history.len());
        let mut value = serde_json::to_value(&s).unwrap();
        for invalid in [0, 10, 255] {
            value["drop_number"] = invalid.into();
            assert_eq!(
                decode_saved_state(&value.to_string()).unwrap().drop_number,
                None
            );
        }
        value.as_object_mut().unwrap().remove("drop_mode");
        value.as_object_mut().unwrap().remove("drop_number");
        let legacy = decode_saved_state(&value.to_string()).unwrap();
        assert!(!legacy.drop_mode);
        assert_eq!(legacy.drop_number, None);
        assert_eq!(legacy.board, s.board);
        assert_eq!(legacy.notes, s.notes);
        assert_eq!(legacy.history.len(), s.history.len());
    }

    #[test]
    fn drop_availability_forces_modes_and_protects_progress() {
        let app = AppState(RwSignal::new(domino_fixture()));
        let original = app.0.get();
        app.0.update(|s| {
            s.note_mode = true;
            s.selected = Some((0, 2));
        });
        app.set_drop_availability(DropAvailability::DropOnly);
        assert!(app.0.get().drop_mode);
        app.select_drop_number(3);
        app.select_paint_tool(2);
        app.toggle_drop_mode(); // Hidden controls cannot bypass the policy.
        assert!(app.0.get().drop_mode);
        assert_eq!(app.0.get().drop_number, Some(3));
        app.set_drop_availability(DropAvailability::NoDrop);
        let disabled = app.0.get();
        assert!(!disabled.drop_mode);
        assert!(disabled.note_mode);
        assert_eq!(disabled.drop_number, None);
        assert_eq!(disabled.paint_tool, None);
        assert_eq!(disabled.selected, None);
        app.toggle_drop_mode();
        assert!(!app.0.get().drop_mode);
        app.set_drop_availability(DropAvailability::Both);
        assert!(!app.0.get().drop_mode);
        app.toggle_drop_mode();
        let after = app.0.get();
        assert!(after.drop_mode);
        assert_eq!(after.board, original.board);
        assert_eq!(after.notes, original.notes);
        assert_eq!(after.cell_colors, original.cell_colors);
        assert_eq!(after.history.len(), original.history.len());
        app.set_drop_availability(DropAvailability::NoDrop);
        app.reset_config();
        assert_eq!(app.0.get().drop_availability, DropAvailability::Both);
        app.toggle_drop_mode();
        assert!(app.0.get().drop_mode);
    }

    #[test]
    fn saved_drop_availability_defaults_to_both_and_enforces_fixed_modes() {
        let s = domino_fixture();
        let mut json = serde_json::to_value(&s).unwrap();
        json.as_object_mut().unwrap().remove("drop_availability");
        assert_eq!(
            decode_saved_state(&json.to_string())
                .unwrap()
                .drop_availability,
            DropAvailability::Both
        );
        for (policy, enabled) in [
            (DropAvailability::DropOnly, true),
            (DropAvailability::NoDrop, false),
        ] {
            json["drop_availability"] = serde_json::to_value(policy).unwrap();
            json["drop_mode"] = (!enabled).into();
            json["drop_number"] = 3.into();
            let loaded = decode_saved_state(&json.to_string()).unwrap();
            assert_eq!(loaded.drop_mode, enabled);
            assert_eq!(loaded.drop_availability, policy);
            assert_eq!(loaded.drop_number, if enabled { Some(3) } else { None });
            assert_eq!(loaded.board, s.board);
            assert_eq!(loaded.notes, s.notes);
        }
    }

    #[test]
    fn game_replacement_keeps_fixed_drop_policy() {
        let app = AppState(RwSignal::new(domino_fixture()));
        for policy in [DropAvailability::DropOnly, DropAvailability::NoDrop] {
            app.set_drop_availability(policy);
            app.new_game(Difficulty::Easy);
            let current = app.0.get();
            let enabled = policy == DropAvailability::DropOnly;
            assert_eq!(current.drop_mode, enabled);
            assert_eq!(current.drop_availability, policy);
            let entry = HistoryEntry::capture(&domino_fixture());
            for initial in [false, true] {
                let restored = entry.restore(&current, initial);
                assert_eq!(restored.drop_mode, enabled);
                assert_eq!(restored.drop_availability, policy);
            }
        }
    }

    #[test]
    fn new_games_keep_modes_but_clear_digits_colors_and_selection() {
        let mut s = domino_fixture();
        s.note_mode = true;
        s.drop_mode = true;
        s.drop_number = Some(3);
        s.paint_tool = Some(2);
        s.selected = Some((0, 2));
        s.start_new_game(Difficulty::Easy);
        assert!(s.note_mode && s.drop_mode);
        assert_eq!(s.drop_number, None);
        assert_eq!(s.paint_tool, None);
        assert_eq!(s.selected, None);
    }

    #[test]
    fn drop_highlights_ignore_any_legacy_anchor() {
        let mut s = domino_fixture();
        s.drop_mode = true;
        s.drop_number = Some(3);
        let expected: Vec<_> = (0..81)
            .map(|i| {
                (
                    s.placement_available(i / 9, i % 9),
                    s.placement_blocker(i / 9, i % 9),
                )
            })
            .collect();
        for anchor in [None, Some((0, 2)), Some((8, 8)), Some((0, 0))] {
            s.selected = anchor;
            assert_eq!(s.active_number(), Some(3));
            let actual: Vec<_> = (0..81)
                .map(|i| {
                    (
                        s.placement_available(i / 9, i % 9),
                        s.placement_blocker(i / 9, i % 9),
                    )
                })
                .collect();
            assert_eq!(actual, expected);
            assert!(actual
                .iter()
                .all(|(_, blocker)| *blocker != Some("selected")));
        }
    }

    #[test]
    fn paint_preserves_values_and_groups_clear_with_mixed_history() {
        let owner = Owner::new();
        owner.set();
        let app = AppState(RwSignal::new(domino_fixture()));
        let before = app.0.get();
        app.toggle_drop_mode();
        app.select_paint_tool(1);
        for i in [0, 2, 10] {
            app.select_cell(i / 9, i % 9);
        }
        let marked = app.0.get();
        assert_eq!(marked.board, before.board);
        assert_eq!(marked.notes, before.notes);
        assert_eq!(marked.error_count, before.error_count);
        assert_eq!(marked.history.len(), 3);
        app.select_cell(0, 0);
        assert_eq!(app.0.get().cell_colors[0], 0);
        assert_eq!(app.0.get().history.len(), 4);
        app.undo();
        assert_eq!(app.0.get().cell_colors[0], 1);
        app.redo();
        assert_eq!(app.0.get().cell_colors[0], 0);
        app.undo();
        app.select_paint_tool(2);
        app.select_cell(0, 0);
        app.undo();
        assert_eq!(app.0.get().cell_colors[0], 1);
        app.redo();
        assert_eq!(app.0.get().cell_colors[0], 2);
        app.select_cell(0, 0);
        assert_eq!(app.0.get().cell_colors[0], 0);
        app.undo();
        app.clear_colors();
        assert_eq!(app.0.get().cell_colors, [0; 81]);
        app.undo();
        assert_eq!(app.0.get().cell_colors[0], 2);
        assert_eq!(app.0.get().cell_colors[2], 1);
        app.0.update(|s| s.domino_enabled = false);
        app.toggle_drop_mode();
        app.toggle_note_mode();
        app.select_cell(0, 0);
        app.place_number(before.solution[0]);
        assert_eq!(app.0.get().paint_tool, None);
        assert_eq!(app.0.get().cell_colors[0], 2);
        app.undo();
        assert_eq!(app.0.get().board, before.board);
        assert_eq!(app.0.get().cell_colors[0], 2);
        app.redo();
        assert_eq!(app.0.get().board[0], before.solution[0]);
    }

    #[test]
    fn paint_guards_modes_and_domino_cancellation() {
        let owner = Owner::new();
        owner.set();
        let app = AppState(RwSignal::new(domino_fixture()));
        app.toggle_drop_mode();
        app.select_drop_number(3);
        let generation = app.0.get().domino_gen;
        app.select_paint_tool(3);
        assert!(app.0.get().drop_mode);
        assert_eq!(app.0.get().drop_number, Some(3));
        assert_ne!(app.0.get().domino_gen, generation);
        app.0.update(|s| assert!(!s.domino_step(generation)));
        app.select_cell(0, 2);
        app.0.update(|s| s.hinted[10] = s.board[10]);
        app.select_cell(1, 1);
        assert_eq!(app.0.get().cell_colors[10], 3);
        app.toggle_note_mode();
        assert_eq!(app.0.get().paint_tool, None);
        app.select_paint_tool(4);
        app.toggle_drop_mode();
        assert_eq!(app.0.get().paint_tool, None);
        app.toggle_drop_mode();
        app.select_paint_tool(4);
        app.select_paint_tool(4);
        assert_eq!(app.0.get().paint_tool, None);
        app.select_paint_tool(1);
        for won in [false, true] {
            app.0.update(|s| {
                s.won = won;
                s.paused = !won;
            });
            let before = app.0.get();
            app.select_cell(0, 0);
            app.clear_colors();
            app.select_paint_tool(2);
            assert_eq!(app.0.get().cell_colors, before.cell_colors);
            assert_eq!(app.0.get().history.len(), before.history.len());
            assert_eq!(app.0.get().paint_tool, before.paint_tool);
        }
        app.0.update(|s| {
            s.won = false;
            s.paused = false;
        });
        let colors = app.0.get().cell_colors;
        app.reset_config();
        assert_eq!(app.0.get().cell_colors, colors);
        app.new_game(Difficulty::Easy);
        assert_eq!(app.0.get().cell_colors, [0; 81]);
        assert_eq!(app.0.get().paint_tool, None);
    }

    #[test]
    fn selection_first_colors_apply_without_activating_a_tool() {
        let owner = Owner::new();
        owner.set();
        let app = AppState(RwSignal::new(domino_fixture()));
        app.select_paint_tool(1);
        assert!(app.0.get().history.is_empty());
        assert_eq!(app.0.get().paint_tool, None);
        app.select_cell(0, 2);
        // Coloring preserves the selected target.
        let before = app.0.get();
        app.select_paint_tool(2);
        let colored = app.0.get();
        assert_eq!(colored.cell_colors[2], 2);
        assert_eq!(colored.board, before.board);
        assert_eq!(colored.notes, before.notes);
        assert_eq!(colored.selected, before.selected);
        assert_eq!(colored.paint_tool, None);
        app.select_cell(0, 0);
        assert_eq!(app.0.get().cell_colors[0], 0);
        app.select_cell(0, 2);
        app.select_paint_tool(2);
        assert_eq!(app.0.get().cell_colors[2], 0);
        app.undo();
        assert_eq!(app.0.get().cell_colors[2], 2);
        app.redo();
        assert_eq!(app.0.get().cell_colors[2], 0);
    }

    #[test]
    fn drop_edits_use_only_the_independent_highlight_digit() {
        let owner = Owner::new();
        owner.set();
        let mut s = domino_fixture();
        s.domino_enabled = false;
        let app = AppState(RwSignal::new(s));
        app.select_cell(0, 2);
        app.toggle_drop_mode();
        app.select_paint_tool(1);
        assert_eq!(app.0.get().active_number(), None);
        app.select_cell(0, 1);
        assert_eq!(app.0.get().cell_colors[1], 1);
        assert_eq!(app.0.get().active_number(), None);
        app.select_drop_number(3);
        let before = app.0.get();
        app.select_cell(1, 1); // A solved cell must paint, never pick its digit.
        assert_eq!(app.0.get().active_number(), Some(3));
        assert_eq!(app.0.get().board, before.board);
        assert_eq!(app.0.get().cell_colors[10], 1);
        app.select_drop_number(5);
        assert_eq!(app.0.get().paint_tool, Some(1));
        app.select_cell(0, 0);
        assert_eq!(app.0.get().active_number(), Some(5));
        app.select_paint_tool(1); // Resume entry without changing the digit.
        assert_eq!(app.0.get().paint_tool, None);
        assert!(app.0.get().note_mode);
        let before_notes = app.0.get();
        app.select_cell(1, 1); // Notes cannot pick a solved digit.
        assert_eq!(app.0.get().drop_number, Some(5));
        assert_eq!(app.0.get().history.len(), before_notes.history.len());
        app.select_cell(0, 0);
        assert_eq!(app.0.get().notes[0], before_notes.notes[0] ^ (1 << 4));
        app.toggle_note_mode();
        app.select_drop_number(1);
        app.select_cell(0, 0);
        let after = app.0.get();
        assert_eq!(after.board[0], 1);
        assert_eq!(after.selected, None);
        app.select_cell(1, 1); // Ordinary solved-cell picking is still intentional.
        assert_eq!(app.0.get().selected, None);
        assert_eq!(app.0.get().drop_number, Some(after.board[10]));
    }

    #[test]
    fn color_saves_preserve_legacy_progress_and_both_history_stacks() {
        let mut s = domino_fixture();
        s.push_snapshot();
        s.board[0] = s.solution[0];
        s.undo();
        let mut legacy = serde_json::to_value(&s).unwrap();
        legacy.as_object_mut().unwrap().remove("cell_colors");
        for key in ["history", "redo_stack"] {
            for snapshot in legacy[key].as_array_mut().unwrap() {
                snapshot.as_object_mut().unwrap().remove("cell_colors");
            }
        }
        let loaded = decode_saved_state(&legacy.to_string()).unwrap();
        assert_eq!(loaded.board, s.board);
        assert_eq!(loaded.notes, s.notes);
        assert_eq!(loaded.history.len(), s.history.len());
        assert_eq!(loaded.redo_stack.len(), s.redo_stack.len());
        assert_eq!(loaded.cell_colors, [0; 81]);
        assert_eq!(loaded.redo_stack[0].cell_colors, [0; 81]);
        s.paint_tool = Some(1);
        s.cell_colors[0] = 4;
        s.redo_stack[0].cell_colors[2] = 255;
        let loaded = decode_saved_state(&serde_json::to_string(&s).unwrap()).unwrap();
        assert_eq!(loaded.cell_colors[0], 4);
        assert_eq!(loaded.redo_stack[0].cell_colors[2], 0);
        assert_eq!(loaded.paint_tool, None);
        let mut invalid = serde_json::to_value(&s).unwrap();
        invalid["cell_colors"] = serde_json::json!([1, 2]);
        assert!(decode_saved_state(&invalid.to_string()).is_err());
    }

    fn domino_fixture() -> GameState {
        let mut s = GameState::default();
        for r in 0..9 {
            for c in 0..9 {
                s.solution[r * 9 + c] = ((r * 3 + r / 3 + c) % 9 + 1) as u8;
            }
        }
        s.board = s.solution;
        for i in [0, 1, 9] {
            s.board[i] = 0;
        }
        s.givens = s.board;
        s.notes = [511; 81];
        s.domino_enabled = true;
        s.domino.empty_cell_threshold = 0;
        s.sound_type = SoundType::None;
        s
    }

    #[test]
    fn drop_picks_solved_digits_without_editing_any_source_or_history() {
        for source in ["given", "hint", "player"] {
            let mut s = domino_fixture();
            s.board[17] = 0;
            s.givens[17] = 0; // Keep the picked digit incomplete for placement.
            if source != "given" {
                s.givens[2] = 0;
            }
            if source == "hint" {
                s.hinted[2] = s.board[2];
            }
            s.history.push(Snapshot {
                board: s.board,
                notes: s.notes,
                cell_colors: s.cell_colors,
            });
            s.redo_stack = s.history.clone();
            let before = serde_json::to_value(&s).unwrap();
            let generation = s.domino_gen;
            let app = AppState(RwSignal::new(s));
            app.toggle_drop_mode();
            app.select_drop_number(9);
            for _ in 0..2 {
                app.select_cell(0, 2);
            }
            let after = app.0.get();
            assert_eq!(after.drop_number, Some(after.solution[2]));
            assert_eq!(after.selected, None);
            assert_eq!(after.domino_gen, generation);
            let mut encoded = serde_json::to_value(&after).unwrap();
            encoded["selected"] = before["selected"].clone();
            encoded["drop_mode"] = before["drop_mode"].clone();
            encoded["drop_number"] = before["drop_number"].clone();
            assert_eq!(
                encoded, before,
                "picking must not mutate persisted gameplay"
            );
            app.select_cell(0, 0);
            let entered = app.0.get();
            assert_eq!(entered.board[0], after.solution[2]);
        }
    }

    #[test]
    fn solved_digit_pick_respects_mode_preference_and_game_guards() {
        let mut s = domino_fixture();
        s.givens[2] = 0;
        s.board[8] = 0;
        s.givens[8] = 0; // Nine must remain available for the legacy entry path.
        let app = AppState(RwSignal::new(s));
        app.select_cell(0, 2);
        assert_eq!(app.0.get().drop_number, None);
        assert!(!app.0.get().drop_mode);
        app.toggle_drop_mode();
        app.select_drop_number(9);
        app.toggle_drop_pick_solved();
        app.select_cell(0, 2);
        assert_eq!(app.0.get().board[2], 9);
        assert_eq!(app.0.get().drop_number, Some(9));
        assert_eq!(app.0.get().history.len(), 1);
        app.undo();
        assert_eq!(app.0.get().board[2], 3);
        app.toggle_drop_pick_solved();
        app.0.update(|s| s.board[2] = 8); // Incorrect entries remain editable.
        app.select_cell(0, 2);
        assert_eq!(app.0.get().board[2], 9);
        assert_eq!(app.0.get().drop_number, Some(9));
        for won in [false, true] {
            app.0.update(|s| {
                s.board[2] = 3;
                s.paused = !won;
                s.won = won;
            });
            app.select_cell(0, 2);
            assert_eq!(app.0.get().drop_number, Some(9));
            assert_eq!(app.0.get().board[2], 3);
        }
    }

    #[test]
    fn completed_digits_block_drop_placement_but_allow_notes() {
        let mut s = domino_fixture();
        s.board = s.solution.map(|v| if v == 1 { 1 } else { 0 });
        s.givens = s.board;
        s.notes = [0; 81];
        assert!(s.number_is_solved(1));
        assert!(!s.number_is_solved(0));
        assert!(!s.number_is_solved(2));
        let app = AppState(RwSignal::new(s));
        app.toggle_drop_mode();
        app.select_cell(0, 0); // Picking a complete digit cannot bypass the guard.
        assert_eq!(app.0.get().drop_number, Some(1));
        let before = app.0.get();
        app.select_cell(0, 1);
        assert_eq!(app.0.get().board, before.board);
        assert_eq!(app.0.get().history.len(), 0);
        assert_eq!(app.0.get().error_count, before.error_count);
        app.toggle_note_mode();
        app.select_cell(0, 1);
        assert_eq!(app.0.get().notes[1], 1);
        app.select_cell(0, 1);
        assert_eq!(app.0.get().notes[1], 0);
        app.toggle_note_mode();
        app.select_cell(0, 1);
        assert_eq!(app.0.get().board[1], 0);
        assert_eq!(app.0.get().history.len(), 2);
        app.0.update(|s| {
            s.board[0] = 0; // Eight correct ones plus one incorrect one is incomplete.
            s.givens[0] = 0;
            s.board[1] = 1;
        });
        assert!(!app.0.get().number_is_solved(1));
        app.select_cell(0, 0);
        assert_eq!(app.0.get().board[0], 1);
        assert!(app.0.get().number_is_solved(1));
        app.toggle_drop_pick_solved();
        app.select_cell(0, 2);
        assert_eq!(app.0.get().board[2], 0);
    }

    #[test]
    fn solved_digit_pick_preference_is_compatible_persistent_and_resettable() {
        let mut s = domino_fixture();
        s.timer_seconds = 123;
        s.error_count = 2;
        s.selected = Some((0, 0));
        s.history.push(Snapshot {
            board: s.board,
            notes: s.notes,
            cell_colors: s.cell_colors,
        });
        let mut legacy = serde_json::to_value(&s).unwrap();
        legacy.as_object_mut().unwrap().remove("drop_pick_solved");
        let loaded = decode_saved_state(&legacy.to_string()).unwrap();
        assert!(loaded.drop_pick_solved);
        assert_eq!(
            serde_json::to_value(&loaded).unwrap(),
            serde_json::to_value(&s).unwrap()
        );
        s.drop_pick_solved = false;
        let loaded = decode_saved_state(&serde_json::to_string(&s).unwrap()).unwrap();
        assert!(!loaded.drop_pick_solved);
        let app = AppState(RwSignal::new(loaded));
        app.reset_config();
        assert!(app.0.get().drop_pick_solved);
        assert_eq!(app.0.get().board, s.board);
        assert_eq!(app.0.get().timer_seconds, 123);
        assert_eq!(app.0.get().history.len(), 1);
        app.toggle_drop_pick_solved();
        app.new_game(Difficulty::Easy);
        assert!(!app.0.get().drop_pick_solved);
    }

    #[test]
    fn domino_timing_and_normalization() {
        let mut d = DominoSettings::default();
        assert_eq!(d.next_delay(600), 480);
        assert_eq!(d.next_delay(480), 384);
        assert_eq!(d.next_delay(100), 100);
        d.acceleration_percent = 0;
        assert_eq!(d.next_delay(600), 600);
        d.initial_delay_ms = 100;
        d.minimum_delay_ms = 900;
        d.acceleration_percent = 99;
        d.empty_cell_threshold = 99;
        d.normalize();
        assert_eq!(d.minimum_delay_ms, 100);
        assert_eq!(d.acceleration_percent, 50);
        assert_eq!(d.empty_cell_threshold, 81);
        assert_eq!(d.next_delay(101), 100);
    }

    #[test]
    fn domino_threshold_is_checked_after_correct_entries_in_both_modes() {
        for drop in [false, true] {
            let mut s = domino_fixture();
            s.domino.empty_cell_threshold = 2;
            assert!(!s.domino_eligible());
            let value = s.solution[0];
            let app = AppState(RwSignal::new(s));
            if drop {
                app.toggle_drop_mode();
                app.select_drop_number(value);
                app.select_cell(0, 0);
            } else {
                app.select_cell(0, 0);
                app.place_number(value);
            }
            let s = app.0.get();
            assert!(s.domino_eligible());
            assert_eq!(s.board.iter().filter(|&&v| v == 0).count(), 2);
            assert_eq!(s.history.len(), 1);
            assert_eq!(s.domino_gen, 1);
        }
        let mut s = domino_fixture();
        s.domino.empty_cell_threshold = 1;
        assert!(!s.enter_number(0, 0, s.solution[0]));
        s.domino.empty_cell_threshold = 0;
        assert!(s.domino_eligible());
        s = domino_fixture();
        assert!(!s.enter_number(0, 0, 9));
        s = domino_fixture();
        s.note_mode = true;
        assert!(!s.enter_number(0, 0, s.solution[0]));
        s = domino_fixture();
        s.paused = true;
        assert!(!s.enter_number(0, 0, s.solution[0]));
        assert_eq!(s.board[0], 0);
    }

    #[test]
    fn domino_steps_ignore_notes_clear_peers_and_group_history() {
        let mut s = domino_fixture();
        let before = s.board;
        let notes = s.notes;
        assert!(s.enter_number(0, 0, s.solution[0]));
        let generation = s.domino_gen;
        assert!(s.domino_step(generation));
        assert_eq!(s.just_filled, Some((0, 1)));
        assert_eq!(s.notes[1], 0);
        assert_eq!(s.notes[10] & (1 << (s.solution[1] - 1)), 0);
        assert!(s.domino_step(generation));
        assert!(s.won);
        assert!(!s.domino_step(generation));
        assert_eq!(s.history.len(), 1);
        let after = s.board;
        let after_notes = s.notes;
        s.undo();
        assert_eq!(s.board, before);
        assert_eq!(s.notes, notes);
        assert!(!s.won);
        assert!(!s.domino_step(generation));
        s.redo();
        assert_eq!(s.board, after);
        assert_eq!(s.notes, after_notes);
        assert!(s.won);
    }

    #[test]
    fn domino_scans_row_column_box_and_skips_unsafe_candidates() {
        // Block 1..8 around (0,0), with each unit providing distinct exclusions.
        let mut s = domino_fixture();
        s.board = [0; 81];
        s.solution = [9; 81];
        for (i, v) in [
            (3, 1),
            (4, 2),
            (5, 3),
            (27, 4),
            (36, 5),
            (45, 6),
            (10, 7),
            (20, 8),
        ] {
            s.board[i] = v;
        }
        assert_eq!(sudoku_engine::candidates(&s.board, 0, 0), vec![9]);
        s.notes[0] = 1; // Notes cannot make 1 eligible or exclude 9.
        assert!(s.domino_step(s.domino_gen));
        assert_eq!(s.board[0], 9);
        s.board[0] = 0;
        s.solution = [1; 81];
        assert!(!s.domino_step(s.domino_gen));
        assert_eq!(s.board[0], 0);
        s.board[6] = 9; // No candidate at the target.
        assert!(sudoku_engine::candidates(&s.board, 0, 0).is_empty());
        assert!(!s.domino_step(s.domino_gen));
        s.board = [0; 81]; // Multiple candidates; notes are not deductions.
        s.notes = [1; 81];
        assert!(!s.domino_step(s.domino_gen));
    }

    #[test]
    fn domino_recomputes_candidates_and_leaves_hidden_singles_alone() {
        let mut s = domino_fixture();
        let rows = [
            "530070000",
            "600195000",
            "098000060",
            "800060003",
            "400803001",
            "700020006",
            "060000280",
            "000419005",
            "000080079",
        ];
        let solution = [
            "534678912",
            "672195348",
            "198342567",
            "859761423",
            "426853791",
            "713924856",
            "961537284",
            "287419635",
            "345286179",
        ];
        for (r, row) in rows.iter().enumerate() {
            for (c, ch) in row.bytes().enumerate() {
                s.board[r * 9 + c] = ch - b'0';
            }
            for (c, ch) in solution[r].bytes().enumerate() {
                s.solution[r * 9 + c] = ch - b'0';
            }
        }
        let initial = s.board;
        let mut newly_single = false;
        while s.domino_step(s.domino_gen) {
            let (r, c) = s.just_filled.unwrap();
            newly_single |= sudoku_engine::candidates(&initial, r, c).len() > 1;
        }
        assert!(newly_single);
        assert_eq!(s.board, s.solution);

        // Create a hidden single 1 at (0,0): the other row cells exclude 1,
        // but the target still has several candidates and must remain empty.
        s.board = [0; 81];
        for i in [12, 15, 28, 29] {
            s.board[i] = 1;
        }
        assert!(sudoku_engine::candidates(&s.board, 0, 0).len() > 1);
        assert_eq!(
            (0..9)
                .filter(|&c| sudoku_engine::candidates(&s.board, 0, c).contains(&1))
                .collect::<Vec<_>>(),
            vec![0]
        );
        assert!(!s.domino_step(s.domino_gen));
        assert_eq!(s.board[0], 0);
    }

    #[test]
    fn domino_callbacks_are_invalidated_by_state_actions() {
        for action in 0..10 {
            let app = AppState(RwSignal::new(domino_fixture()));
            app.select_cell(0, 0);
            app.place_number(app.0.get().solution[0]);
            let generation = app.0.get().domino_gen;
            match action {
                0 => app.toggle_domino(),
                1 => app.reset_config(),
                2 => app.set_domino_settings(DominoSettings::default()),
                3 => app.toggle_pause(),
                4 => app.undo(),
                5 => app.redo(),
                6 => app.hint(),
                7 => app.auto_notes(),
                8 => app.place_number(0),
                _ => app.new_game(Difficulty::Easy),
            }
            let before = app.0.get().board;
            app.0.update(|s| assert!(!s.domino_step(generation)));
            assert_eq!(app.0.get().board, before);
        }
    }

    #[test]
    fn domino_preferences_are_compatible_persistent_and_resettable() {
        let mut s = domino_fixture();
        s.timer_seconds = 123;
        s.push_snapshot();
        s.domino = DominoSettings {
            initial_delay_ms: 1000,
            acceleration_percent: 0,
            minimum_delay_ms: 250,
            empty_cell_threshold: 12,
        };
        let mut json = serde_json::to_value(&s).unwrap();
        let restored = decode_saved_state(&json.to_string()).unwrap();
        assert_eq!(restored.domino, s.domino);
        assert_eq!(restored.domino_gen, 0);
        json.as_object_mut().unwrap().remove("domino");
        json["domino_gen"] = serde_json::json!(4294967295u32);
        let legacy = decode_saved_state(&json.to_string()).unwrap();
        assert_eq!(legacy.domino, DominoSettings::default());
        assert_eq!(legacy.domino_gen, 0);
        assert_eq!(legacy.board, s.board);
        assert_eq!(legacy.notes, s.notes);
        assert_eq!(legacy.timer_seconds, 123);
        assert_eq!(legacy.history[0].board, s.history[0].board);
        assert!(legacy.domino_enabled);
        json["domino"] = serde_json::json!({"initial_delay_ms": 1, "empty_cell_threshold": 999});
        let partial = decode_saved_state(&json.to_string()).unwrap();
        assert_eq!(partial.domino.initial_delay_ms, 100);
        assert_eq!(partial.domino.minimum_delay_ms, 100);
        assert_eq!(partial.domino.acceleration_percent, 20);
        assert_eq!(partial.domino.empty_cell_threshold, 81);
        let app = AppState(RwSignal::new(s.clone()));
        app.new_game(Difficulty::Easy);
        assert_eq!(app.0.get().domino, s.domino);
        let board = app.0.get().board;
        app.reset_config();
        assert_eq!(app.0.get().board, board);
        assert_eq!(app.0.get().domino, DominoSettings::default());
        assert!(app.0.get().domino_enabled);
    }

    #[test]
    fn config_defaults_preserve_explicit_saved_preferences() {
        let mut s = GameState::default();
        assert_eq!(s.highlights.selected_shading, 20);
        assert_eq!(s.highlights.matching_shading, 20);
        assert_eq!(s.highlights.available_shading, 100);
        assert_eq!(s.highlights.completed_contrast, 100);
        assert!(s.domino_enabled);
        assert_eq!(s.domino.empty_cell_threshold, 10);
        // The previous release's defaults are explicit saved preferences too.
        s.highlights = HighlightSettings {
            selected_shading: 100,
            matching_shading: 100,
            available_shading: 0,
            completed_contrast: 35,
            dots: false,
            stripes: false,
        };
        s.domino_enabled = false;
        s.domino.empty_cell_threshold = 0;
        s.push_snapshot();
        let mut saved = serde_json::to_value(&s).unwrap();
        let restored = decode_saved_state(&saved.to_string()).unwrap();
        assert_eq!(restored.highlights, s.highlights);
        assert!(!restored.domino_enabled);
        assert_eq!(restored.domino.empty_cell_threshold, 0);
        for field in ["highlights", "domino_enabled", "domino"] {
            saved.as_object_mut().unwrap().remove(field);
        }
        let restored = decode_saved_state(&saved.to_string()).unwrap();
        assert_eq!(restored.highlights, HighlightSettings::default());
        assert!(restored.domino_enabled);
        assert_eq!(restored.domino.empty_cell_threshold, 10);
        assert_eq!(restored.board, s.board);
        assert_eq!(restored.notes, s.notes);
        assert_eq!(restored.history[0].board, s.history[0].board);
    }

    #[test]
    fn highlight_preferences_round_trip_and_old_saves_keep_progress() {
        let mut s = GameState::default();
        s.highlights = HighlightSettings {
            selected_shading: 25,
            matching_shading: 70,
            available_shading: 40,
            completed_contrast: 65,
            dots: false,
            stripes: false,
        };
        s.push_snapshot();
        let mut saved = serde_json::to_value(&s).unwrap();
        let restored = decode_saved_state(&saved.to_string()).unwrap();
        assert_eq!(restored.highlights, s.highlights);
        let mut previous_save = saved.clone();
        previous_save["highlights"]
            .as_object_mut()
            .unwrap()
            .remove("completed_contrast");
        let migrated = decode_saved_state(&previous_save.to_string()).unwrap();
        assert_eq!(migrated.highlights.completed_contrast, 100);
        assert_eq!(
            migrated.highlights.selected_shading,
            s.highlights.selected_shading
        );
        assert_eq!(
            migrated.highlights.matching_shading,
            s.highlights.matching_shading
        );
        assert_eq!(migrated.board, s.board);
        assert_eq!(migrated.notes, s.notes);
        assert_eq!(migrated.history[0].board, s.history[0].board);
        let mut two_slider_save = saved.clone();
        two_slider_save["highlights"]
            .as_object_mut()
            .unwrap()
            .remove("available_shading");
        let migrated = decode_saved_state(&two_slider_save.to_string()).unwrap();
        assert_eq!(migrated.highlights.selected_shading, 25);
        assert_eq!(migrated.highlights.matching_shading, 70);
        assert_eq!(migrated.highlights.available_shading, 100);
        assert!(!migrated.highlights.dots && !migrated.highlights.stripes);
        assert_eq!(migrated.board, s.board);
        saved.as_object_mut().unwrap().remove("highlights");
        let legacy = decode_saved_state(&saved.to_string()).unwrap();
        assert_eq!(legacy.highlights, HighlightSettings::default());
        assert_eq!(legacy.board, s.board);
        assert_eq!(legacy.notes, s.notes);
        assert_eq!(legacy.history.len(), s.history.len());
        saved["highlights"] =
            serde_json::json!({"selected_shading": 200, "completed_contrast": 255});
        let partial = decode_saved_state(&saved.to_string()).unwrap();
        assert_eq!(partial.highlights.selected_shading, 100);
        assert_eq!(partial.highlights.matching_shading, 20);
        assert_eq!(partial.highlights.available_shading, 100);
        assert_eq!(partial.highlights.completed_contrast, 100);
    }

    #[test]
    fn highlight_preferences_survive_new_games_and_reset_without_changing_board() {
        let state = AppState::new();
        let preferences = HighlightSettings {
            selected_shading: 0,
            matching_shading: 45,
            available_shading: 60,
            completed_contrast: 80,
            dots: false,
            stripes: false,
        };
        state.0.update(|s| s.highlights = preferences);
        state.new_game(Difficulty::Easy);
        assert_eq!(state.0.get().highlights, preferences);
        let board = state.0.get().board;
        state.reset_config();
        assert_eq!(state.0.get().highlights, HighlightSettings::default());
        assert_eq!(state.0.get().board, board);
    }

    #[test]
    fn placement_preview_checks_every_occurrence_without_using_solution() {
        let mut s = GameState::default();
        s.board = [0; 81];
        s.board[0] = 5;
        s.board[80] = 5;
        s.selected = Some((0, 0));
        s.solution = [1; 81];
        assert_eq!(s.active_number(), Some(5));
        for (r, c) in [(0, 4), (4, 0), (1, 1), (7, 7), (8, 4)] {
            assert_eq!(s.placement_available(r, c), Some(false));
        }
        assert_eq!(s.placement_available(4, 4), Some(true));
        assert_eq!(s.placement_available(0, 0), None);
        s.solution = [9; 81];
        assert_eq!(s.placement_available(4, 4), Some(true));
        s.selected = Some((4, 4));
        assert_eq!(s.active_number(), None);
        assert_eq!(s.placement_available(4, 4), Some(false));
        assert_eq!(s.placement_available(7, 7), Some(true));
    }

    #[test]
    fn empty_selection_highlights_peers_and_available_empty_cells() {
        let mut s = GameState::default();
        s.board = [0; 81];
        s.selected = Some((4, 4));
        assert_eq!(s.active_number(), None);
        for (r, c) in [(4, 4), (4, 8), (8, 4), (3, 3)] {
            assert_eq!(s.placement_available(r, c), Some(false));
            assert_eq!(s.placement_blocker(r, c), Some("selected"));
        }
        assert_eq!(s.placement_available(0, 0), Some(true));
        assert_eq!(s.placement_blocker(0, 0), None);
        s.selected = None;
        assert_eq!(s.placement_available(0, 0), None);
    }

    #[test]
    fn placement_blockers_distinguish_selected_matching_and_overlap() {
        let mut s = GameState::default();
        s.board = [0; 81];
        s.board[0] = 5;
        s.board[42] = 5;
        s.selected = Some((0, 0));
        assert_eq!(s.placement_blocker(0, 4), Some("selected"));
        assert_eq!(s.placement_blocker(1, 1), Some("selected"));
        assert_eq!(s.placement_blocker(4, 4), Some("matching"));
        assert_eq!(s.placement_blocker(3, 8), Some("matching"));
        assert_eq!(s.placement_blocker(7, 6), Some("matching"));
        assert_eq!(s.placement_blocker(4, 0), Some("selected"));
        assert_eq!(s.placement_blocker(7, 4), None);
        assert_eq!(s.placement_blocker(0, 0), None);
        s.drop_mode = true;
        s.drop_number = Some(5);
        s.selected = Some((7, 4));
        assert_eq!(s.placement_blocker(0, 4), Some("matching"));
        assert_eq!(s.placement_blocker(4, 4), Some("matching"));
        s.drop_number = None;
        assert_eq!(s.placement_blocker(4, 4), None);
    }

    #[test]
    fn drop_preview_overrides_selection_and_clears_with_number() {
        let mut s = GameState::default();
        s.board = [0; 81];
        s.board[0] = 5;
        s.board[80] = 7;
        s.selected = Some((0, 0));
        s.drop_mode = true;
        assert_eq!(s.active_number(), None);
        s.drop_number = Some(7);
        assert_eq!(s.active_number(), Some(7));
        assert_eq!(s.placement_available(7, 7), Some(false));
        assert_eq!(s.placement_available(0, 4), Some(true));
        s.drop_number = None;
        assert_eq!(s.placement_available(7, 7), None);
        s.drop_mode = false;
        assert_eq!(s.active_number(), Some(5));
    }

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

    #[test]
    fn test_hint_picks_fewest_candidates() {
        let state = AppState::new();

        // Before hint, compute minimum candidate count among unsolved cells
        let s = state.0.get();
        let mut min_cands: Option<u32> = None;
        let mut cands_per_cell: [u32; 81] = [0; 81];
        for i in 0..81 {
            if s.board[i] != s.solution[i] {
                let r = i / 9;
                let c = i % 9;
                let cands = (1..=9)
                    .filter(|&v| crate::sudoku_engine::is_valid_move(&s.board, r, c, v))
                    .count() as u32;
                cands_per_cell[i] = cands;
                if cands > 0 {
                    min_cands = Some(min_cands.map_or(cands, |m| m.min(cands)));
                }
            }
        }
        drop(s);

        state.hint();

        // The filled cell must have had the minimum candidate count
        let s = state.0.get();
        let mut filled_idx = None;
        for i in 0..81 {
            if s.board[i] != 0 && s.board[i] == s.solution[i] && cands_per_cell[i] > 0 {
                // This cell was unsolved before (cands > 0) and is now correct
                filled_idx = Some(i);
                break;
            }
        }
        // Note: this may not find the exact cell if it was already filled before hint,
        // but hint guarantees it fills at least one new cell.
        if let Some(idx) = filled_idx {
            if let Some(min) = min_cands {
                assert_eq!(
                    cands_per_cell[idx], min,
                    "Hint must fill a cell with the fewest candidates (had {}, min was {})",
                    cands_per_cell[idx], min
                );
            }
        }
    }

    #[test]
    fn test_hint_deterministic_from_seed() {
        // Same board + same seed = same hint result
        let board = generate_board(Difficulty::Easy);
        let state1 = GameState {
            game_id: new_game_id(),
            board: board.cells,
            givens: board.cells,
            solution: board.solution,
            notes: [0u16; 81],
            cell_colors: [0; 81],
            difficulty: board.difficulty,
            requested_difficulty: Some(board.difficulty),
            rating: board.rating,
            seed: board.seed,
            hinted: [0u8; 81],
            error_count: 0,
            note_mode: false,
            drop_mode: false,
            drop_availability: DropAvailability::Both,
            paint_tool: None,
            drop_number: None,
            drop_pick_solved: true,
            undo_enabled: true,
            auto_notes_enabled: true,
            hint_enabled: true,
            domino_enabled: true,
            domino: DominoSettings::default(),
            domino_gen: 0,
            sound_type: SoundType::default(),
            effects: crate::effects::EffectsSettings::default(),
            highlights: HighlightSettings::default(),
            just_filled: None,
            selected: None,
            timer_seconds: 0,
            paused: false,
            won: false,
            celebrated: false,
            history: Vec::new(),
            redo_stack: Vec::new(),
        };
        let state2 = state1.clone();

        let app1 = AppState(RwSignal::new(state1));
        let app2 = AppState(RwSignal::new(state2));

        app1.hint();
        app2.hint();

        assert_eq!(
            app1.0.get().board,
            app2.0.get().board,
            "Same seed should produce same hint cell"
        );
        // Hint should NOT add to undo history
        assert!(app1.0.get().history.is_empty(), "Hint must not be undoable");
        // Hint should mark the cell
        let hinted_count = app1.0.get().hinted.iter().filter(|&&v| v != 0).count();
        assert_eq!(hinted_count, 1, "Hint should mark exactly one cell");
    }
    #[test]
    fn correct_player_entries_remain_editable_and_survive_history() {
        let state = AppState::new();
        let before = state.0.get();
        let i = (0..81).find(|&i| before.board[i] == 0).unwrap();
        let original_givens = before.givens;
        state.select_cell(i / 9, i % 9);
        state.place_number(before.solution[i]);
        assert!(!state.0.get().is_given(i / 9, i % 9));
        state.place_number(0);
        assert_eq!(state.0.get().board[i], 0);
        state.undo();
        assert_eq!(state.0.get().board[i], before.solution[i]);
        state.redo();
        assert_eq!(state.0.get().board[i], 0);
        assert_eq!(state.0.get().givens, original_givens);
        let loaded = decode_saved_state(&serde_json::to_string(&state.0.get()).unwrap()).unwrap();
        assert_eq!(loaded.givens, original_givens);
    }

    #[test]
    fn hints_lock_without_becoming_original_clues() {
        let state = AppState::new();
        state.hint();
        let s = state.0.get();
        let i = (0..81).find(|&i| s.hinted[i] != 0).unwrap();
        assert!(!s.is_given(i / 9, i % 9));
        state.select_cell(i / 9, i % 9);
        state.place_number(0);
        assert_eq!(state.0.get().board[i], s.solution[i]);
        state.toggle_drop_mode();
        state.select_drop_number(if s.solution[i] == 1 { 2 } else { 1 });
        state.select_cell(i / 9, i % 9);
        assert_eq!(state.0.get().board[i], s.solution[i]);
    }

    #[test]
    fn legacy_save_retains_locked_cells_and_tracks_new_entries() {
        let mut s = GameState::default();
        let i = (0..81).find(|&i| s.board[i] == 0).unwrap();
        s.board[i] = s.solution[i];
        s.notes[(i + 1) % 81] = 5;
        s.timer_seconds = 123;
        s.auto_notes_enabled = false;
        s.push_snapshot();
        let mut json = serde_json::to_value(&s).unwrap();
        json.as_object_mut().unwrap().remove("givens");
        let migrated = decode_saved_state(&json.to_string()).unwrap();
        assert_eq!(migrated.board, s.board);
        assert_eq!(migrated.notes, s.notes);
        assert_eq!(migrated.timer_seconds, 123);
        assert!(!migrated.auto_notes_enabled);
        assert_eq!(migrated.history.len(), 1);
        assert_eq!(migrated.history[0].board, s.history[0].board);
        assert!(migrated.is_locked(i / 9, i % 9));
        let app = AppState(RwSignal::new(migrated));
        let j = (0..81).find(|&j| s.board[j] == 0).unwrap();
        app.select_cell(j / 9, j % 9);
        app.place_number(s.solution[j]);
        assert!(!app.0.get().is_given(j / 9, j % 9));
        let saved = decode_saved_state(&serde_json::to_string(&app.0.get()).unwrap()).unwrap();
        assert!(!saved.is_given(j / 9, j % 9));
    }

    #[test]
    fn legacy_history_does_not_leave_erased_player_entries_locked() {
        let mut s = GameState::default();
        let original_givens = s.givens;
        let i = (0..81).find(|&i| s.board[i] == 0).unwrap();
        s.push_snapshot();
        s.board[i] = s.solution[i];
        let mut json = serde_json::to_value(&s).unwrap();
        json.as_object_mut().unwrap().remove("givens");
        let mut migrated = decode_saved_state(&json.to_string()).unwrap();
        assert!(migrated.is_locked(i / 9, i % 9));
        migrated.undo();
        assert_eq!(migrated.board[i], 0);
        assert!(!migrated.is_locked(i / 9, i % 9));
        assert_eq!(migrated.givens, original_givens);
        migrated.redo();
        assert_eq!(migrated.board[i], s.solution[i]);
        assert!(!migrated.is_locked(i / 9, i % 9));
        let app = AppState(RwSignal::new(migrated));
        app.select_cell(i / 9, i % 9);
        app.place_number(0);
        assert_eq!(app.0.get().board[i], 0);
    }

    #[test]
    fn malformed_given_array_is_rejected() {
        let mut json = serde_json::to_value(GameState::default()).unwrap();
        json["givens"] = serde_json::json!([1, 2]);
        assert!(decode_saved_state(&json.to_string()).is_err());
    }
    #[test]
    fn old_saves_keep_progress_and_new_ratings_round_trip() {
        let app = AppState::new();
        app.new_game(Difficulty::Master);
        let state = app.0.get();
        assert_eq!(state.requested_difficulty, Some(Difficulty::Master));
        assert_eq!(state.rating.as_ref().unwrap().difficulty, state.difficulty);
        let mut json = serde_json::to_value(&state).unwrap();
        let restored = decode_saved_state(&json.to_string()).unwrap();
        assert_eq!(restored.rating, state.rating);
        json.as_object_mut().unwrap().remove("rating");
        json.as_object_mut().unwrap().remove("requested_difficulty");
        let legacy = decode_saved_state(&json.to_string()).unwrap();
        assert_eq!(legacy.board, state.board);
        assert_eq!(legacy.givens, state.givens);
        assert_eq!(legacy.solution, state.solution);
        assert_eq!(legacy.difficulty, state.difficulty);
        assert_eq!(legacy.notes, state.notes);
        assert!(legacy.rating.is_none());
        assert!(legacy.requested_difficulty.is_none());
        app.hint();
        assert!(app.0.get().hinted.iter().any(|&v| v != 0));
        assert_eq!(
            app.0.get().rating,
            state.rating,
            "Moves must not regrade the original puzzle"
        );
        app.toggle_hint();
        let board = app.0.get().board;
        app.hint();
        assert_eq!(
            app.0.get().board,
            board,
            "Hint preference applies at every level"
        );
    }
}

pub fn try_save_state(state: &GameState) -> Result<(), Message> {
    #[cfg(target_arch = "wasm32")]
    {
        let storage = web_sys::window()
            .and_then(|w| w.local_storage().ok().flatten())
            .ok_or(Message::SaveFailedKept)?;
        let json = serde_json::to_string(state).map_err(|_| Message::SaveFailed)?;
        storage
            .set_item("sudoku_state", &json)
            .map_err(|_| Message::SaveFailedKept)?;
    }
    #[cfg(not(target_arch = "wasm32"))]
    let _ = state;
    Ok(())
}

pub fn new_game_id() -> String {
    #[cfg(target_arch = "wasm32")]
    {
        js_sys::eval("crypto.randomUUID()")
            .ok()
            .and_then(|v| v.as_string())
            .unwrap_or_else(|| format!("{}-{}", js_sys::Date::now(), js_sys::Math::random()))
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        use std::sync::atomic::{AtomicU64, Ordering};
        static NEXT: AtomicU64 = AtomicU64::new(1);
        format!("native-{}", NEXT.fetch_add(1, Ordering::Relaxed))
    }
}
