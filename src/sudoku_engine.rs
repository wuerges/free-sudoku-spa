// ponytail: flat [u8; 81] is the simplest that works. 0=empty, 1-9=filled.
// ponytail: clue-count proxy for difficulty. Human-technique grading deferred — add when players complain puzzles are mis-rated.
#![allow(dead_code)]

use crate::serde_helpers::u8_81;
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, PartialEq, Eq, Debug, Serialize, Deserialize)]
pub enum Difficulty {
    Easy,
    Medium,
    Hard,
    Expert,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Board {
    #[serde(with = "u8_81")]
    pub cells: [u8; 81],
    #[serde(with = "u8_81")]
    pub solution: [u8; 81],
    pub difficulty: Difficulty,
}

impl Board {
    fn idx(r: usize, c: usize) -> usize {
        r * 9 + c
    }

    pub fn get(&self, r: usize, c: usize) -> u8 {
        self.cells[Self::idx(r, c)]
    }

    pub fn set(&mut self, r: usize, c: usize, v: u8) {
        self.cells[Self::idx(r, c)] = v;
    }

    pub fn is_given(&self, r: usize, c: usize) -> bool {
        self.solution[Self::idx(r, c)] != 0 && self.get(r, c) != 0
    }

    pub fn clue_count(&self) -> u8 {
        self.cells.iter().filter(|&&c| c != 0).count() as u8
    }
}

/// Generate a complete valid Sudoku grid via backtracking.
fn generate_full() -> [u8; 81] {
    let mut grid = [0u8; 81];
    let mut rng = Rand::new();
    solve(&mut grid, &mut rng);
    grid
}

/// Backtracking solver that fills the first empty cell with candidates in random order.
/// Returns true when the grid is fully filled.
fn solve(grid: &mut [u8; 81], rng: &mut Rand) -> bool {
    if let Some(idx) = (0..81).find(|&i| grid[i] == 0) {
        let row = idx / 9;
        let col = idx % 9;
        let mut cands = candidates(grid, row, col);
        shuffle(&mut cands, rng);
        for v in cands {
            grid[idx] = v;
            if solve(grid, rng) {
                return true;
            }
            grid[idx] = 0;
        }
        false
    } else {
        true
    }
}

/// Count solutions, stopping early if more than `limit` found.
fn count_solutions(grid: &mut [u8; 81], limit: u32) -> u32 {
    if let Some(idx) = (0..81).find(|&i| grid[i] == 0) {
        let row = idx / 9;
        let col = idx % 9;
        let mut count = 0;
        for v in candidates(grid, row, col) {
            grid[idx] = v;
            count += count_solutions(grid, limit - count);
            grid[idx] = 0;
            if count >= limit {
                return count;
            }
        }
        count
    } else {
        1
    }
}

/// Valid candidates for a cell (numbers not in same row/col/box).
fn candidates(grid: &[u8; 81], row: usize, col: usize) -> Vec<u8> {
    let mut used = [false; 10];
    for c in 0..9 {
        used[grid[row * 9 + c] as usize] = true;
        used[grid[c * 9 + col] as usize] = true;
    }
    let br = (row / 3) * 3;
    let bc = (col / 3) * 3;
    for r in br..br + 3 {
        for c in bc..bc + 3 {
            used[grid[r * 9 + c] as usize] = true;
        }
    }
    (1..=9).filter(|&v| !used[v as usize]).collect()
}

/// Check if placing `v` at (row, col) is valid (doesn't conflict with peers).
pub fn is_valid_move(grid: &[u8; 81], row: usize, col: usize, v: u8) -> bool {
    for c in 0..9 {
        if c != col && grid[row * 9 + c] == v {
            return false;
        }
        if c != row && grid[c * 9 + col] == v {
            return false;
        }
    }
    let br = (row / 3) * 3;
    let bc = (col / 3) * 3;
    for r in br..br + 3 {
        for c in bc..bc + 3 {
            if (r != row || c != col) && grid[r * 9 + c] == v {
                return false;
            }
        }
    }
    true
}

/// Find conflicting cells. Returns (r, c) pairs that conflict with the given cell.
pub fn conflicts(grid: &[u8; 81], row: usize, col: usize) -> Vec<(usize, usize)> {
    let v = grid[row * 9 + col];
    if v == 0 {
        return vec![];
    }
    let mut res = Vec::new();
    for c in 0..9 {
        if c != col && grid[row * 9 + c] == v {
            res.push((row, c));
        }
        if c != row && grid[c * 9 + col] == v {
            res.push((c, col));
        }
    }
    let br = (row / 3) * 3;
    let bc = (col / 3) * 3;
    for r in br..br + 3 {
        for c in bc..bc + 3 {
            if (r != row || c != col) && grid[r * 9 + c] == v {
                res.push((r, c));
            }
        }
    }
    res
}

/// All conflicts on the board.
pub fn all_conflicts(grid: &[u8; 81]) -> Vec<(usize, usize)> {
    let mut res = Vec::new();
    for idx in 0..81 {
        if grid[idx] != 0 {
            let row = idx / 9;
            let col = idx % 9;
            let cs = conflicts(grid, row, col);
            if !cs.is_empty() {
                res.push((row, col));
            }
        }
    }
    res
}

/// Generate a puzzle with the given target clue range and 180° rotational symmetry.
pub fn generate(target_clues: std::ops::RangeInclusive<u8>) -> Board {
    let solution = generate_full();
    let mut puzzle = solution;
    let mut rng = Rand::new();

    // Build list of symmetric cell pairs, shuffle
    let mut pairs: Vec<(usize, usize)> = (0..40)
        .map(|i| (i, 80 - i))
        .filter(|&(a, b)| a != b)
        .collect();
    shuffle(&mut pairs, &mut rng);

    for (a, b) in pairs {
        let saved_a = puzzle[a];
        let saved_b = puzzle[b];
        puzzle[a] = 0;
        puzzle[b] = 0;

        let mut test = puzzle;
        if count_solutions(&mut test, 2) != 1 {
            puzzle[a] = saved_a;
            puzzle[b] = saved_b;
        }

        if puzzle.iter().filter(|&&c| c != 0).count() as u8 <= *target_clues.start() {
            break;
        }
    }

    let clue_count = puzzle.iter().filter(|&&c| c != 0).count() as u8;
    let difficulty = if clue_count >= 40 {
        Difficulty::Easy
    } else if clue_count >= 32 {
        Difficulty::Medium
    } else if clue_count >= 26 {
        Difficulty::Hard
    } else {
        Difficulty::Expert
    };

    Board {
        cells: puzzle,
        solution,
        difficulty,
    }
}

/// Simple xorshift PRNG — no need for `rand` dep just for shuffle.
struct Rand(u64);

impl Rand {
    fn new() -> Self {
        // ponytail: seed from a fixed value, deterministic. Replace with proper entropy if needed.
        Rand(0xDEAD_BEEF_CAFE_BABE)
    }
    fn next(&mut self) -> u64 {
        self.0 ^= self.0 << 13;
        self.0 ^= self.0 >> 7;
        self.0 ^= self.0 << 17;
        self.0
    }
}

fn shuffle<T>(slice: &mut [T], rng: &mut Rand) {
    for i in (1..slice.len()).rev() {
        let j = (rng.next() as usize) % (i + 1);
        slice.swap(i, j);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_full_valid() {
        let grid = generate_full();
        assert!(grid.iter().all(|&c| (1..=9).contains(&c)));
        // Check all rows, cols, boxes are valid
        for i in 0..9 {
            let mut row = [0u8; 9];
            let mut col = [0u8; 9];
            for j in 0..9 {
                row[j] = grid[i * 9 + j];
                col[j] = grid[j * 9 + i];
            }
            row.sort();
            col.sort();
            assert_eq!(row, [1, 2, 3, 4, 5, 6, 7, 8, 9]);
            assert_eq!(col, [1, 2, 3, 4, 5, 6, 7, 8, 9]);
        }
        for br in (0..3).map(|x| x * 3) {
            for bc in (0..3).map(|x| x * 3) {
                let mut box_cells = [0u8; 9];
                let mut k = 0;
                for r in br..br + 3 {
                    for c in bc..bc + 3 {
                        box_cells[k] = grid[r * 9 + c];
                        k += 1;
                    }
                }
                box_cells.sort();
                assert_eq!(box_cells, [1, 2, 3, 4, 5, 6, 7, 8, 9]);
            }
        }
    }

    #[test]
    fn test_generate_puzzle_unique() {
        let board = generate(45..=50);
        let mut test = board.cells;
        assert_eq!(count_solutions(&mut test, 2), 1);
    }

    #[test]
    fn test_is_valid_move() {
        let mut grid = [0u8; 81];
        grid[0] = 5;
        assert!(!is_valid_move(&grid, 0, 1, 5)); // same row
        assert!(!is_valid_move(&grid, 1, 0, 5)); // same col
        assert!(!is_valid_move(&grid, 1, 1, 5)); // same box
        assert!(is_valid_move(&grid, 0, 1, 3)); // ok
    }
}
