//! Archive snapshots, restoration and retention policy.
use crate::{
    sharing::SharedGame,
    state::{new_game_id, GameState, Snapshot},
};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct HistoryEntry {
    pub version: u8,
    pub id: String,
    pub game: SharedGame,
    pub history: Vec<Snapshot>,
    pub redo_stack: Vec<Snapshot>,
    pub pinned: bool,
    pub order: u64,
    pub last_played_ms: f64,
}
impl HistoryEntry {
    pub fn capture(state: &GameState) -> Self {
        Self {
            version: 1,
            id: state.game_id.clone(),
            game: SharedGame::capture(state, false),
            history: state.history.clone(),
            redo_stack: state.redo_stack.clone(),
            pinned: false,
            order: 0,
            last_played_ms: 0.0,
        }
    }
    pub fn valid(&self) -> bool {
        self.version == 1
            && !self.id.is_empty()
            && self.id.len() <= 128
            && self.last_played_ms.is_finite()
            && self.game.validate().is_ok()
            && self.history.iter().chain(&self.redo_stack).all(|s| {
                s.board.iter().all(|&n| n <= 9)
                    && s.notes.iter().all(|&n| n <= 511)
                    && s.cell_colors.iter().all(|&n| n <= 4)
            })
    }
    pub fn restore(&self, local: &GameState, initial: bool) -> GameState {
        let mut state = local.clone();
        let game = if initial {
            self.game.initial_position()
        } else {
            self.game.clone()
        };
        state.import_shared(&game);
        state.game_id = if initial {
            new_game_id()
        } else {
            self.id.clone()
        };
        if !initial {
            state.history = self.history.clone();
            state.redo_stack = self.redo_stack.clone();
        }
        state
    }
}

pub fn evictions(entries: &[HistoryEntry], active: &str) -> Vec<String> {
    let mut past: Vec<_> = entries
        .iter()
        .filter(|e| e.id != active && !e.pinned)
        .collect();
    past.sort_by(|a, b| b.order.cmp(&a.order).then_with(|| a.id.cmp(&b.id)));
    past.into_iter().skip(10).map(|e| e.id.clone()).collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    fn entry(id: usize, pinned: bool) -> HistoryEntry {
        let mut entry = HistoryEntry::capture(&GameState::default());
        entry.id = format!("game-{id}");
        entry.order = id as u64;
        entry.pinned = pinned;
        entry
    }
    #[test]
    fn retention_excludes_active_and_every_pin() {
        let mut entries = vec![];
        for id in 0..25 {
            entries.push(entry(id, id < 12));
        }
        let removed = evictions(&entries, "game-24");
        assert_eq!(removed, vec!["game-13", "game-12"]);
        assert_eq!(
            entries
                .iter()
                .filter(|e| !e.pinned && e.id != "game-24" && !removed.contains(&e.id))
                .count(),
            10
        );
        assert!(!removed.contains(&"game-24".to_string()));
        entries[0].pinned = false;
        assert!(evictions(&entries, "game-24").contains(&"game-0".into()));
    }
    #[test]
    fn resume_keeps_identity_history_and_local_preferences_restart_is_separate() {
        let mut source = GameState::default();
        source.timer_seconds = 321;
        source.error_count = 7;
        source.history.push(Snapshot {
            board: source.board,
            notes: source.notes,
            cell_colors: source.cell_colors,
        });
        source.redo_stack = source.history.clone();
        source.cell_colors[0] = 4;
        let entry = HistoryEntry::capture(&source);
        let mut local = GameState::default();
        local.highlights.completed_contrast = 37;
        local.undo_enabled = false;
        let resume = entry.restore(&local, false);
        assert_eq!(resume.game_id, source.game_id);
        assert_eq!(resume.timer_seconds, 321);
        assert_eq!(resume.history.len(), 1);
        assert_eq!(resume.redo_stack.len(), 1);
        assert!(!resume.undo_enabled);
        assert_eq!(resume.highlights.completed_contrast, 37);
        assert!(resume.paused && resume.selected.is_none());
        let initial = entry.restore(&local, true);
        assert_ne!(initial.game_id, source.game_id);
        assert_eq!(initial.board, source.givens);
        assert_eq!(initial.cell_colors, [0; 81]);
        assert_eq!(initial.timer_seconds, 0);
        assert!(initial.history.is_empty() && initial.redo_stack.is_empty());
        assert_eq!(entry.game.seconds, 321);
    }
    #[test]
    fn legacy_save_accepts_missing_identity_and_completed_archives_do_not_celebrate() {
        let mut state = GameState::default();
        state.board = state.solution;
        state.won = true;
        let mut json = serde_json::to_value(&state).unwrap();
        json.as_object_mut().unwrap().remove("game_id");
        let loaded: GameState = serde_json::from_value(json).unwrap();
        assert!(loaded.game_id.is_empty());
        assert_eq!(loaded.board, state.board);
        let entry = HistoryEntry::capture(&state);
        let restored = entry.restore(&GameState::default(), false);
        assert!(restored.won && restored.celebrated && restored.paused);
        let mut broken = entry.clone();
        broken.version = 2;
        assert!(!broken.valid());
        let mut broken = entry;
        broken.history.push(Snapshot {
            board: [10; 81],
            notes: [0; 81],
            cell_colors: [0; 81],
        });
        assert!(!broken.valid());
    }
}
