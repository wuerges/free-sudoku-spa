//! Offline importer: cargo run --release --features puzzle-bank-tools --bin import_puzzle_bank -- SOURCE_DIR OUTPUT.csv
#![allow(dead_code)]
#[path = "../serde_helpers.rs"]
mod serde_helpers;
#[path = "../sudoku_engine.rs"]
mod sudoku_engine;
use std::{collections::HashSet, io::Write};
use sudoku_engine::{count_solutions, grade};

// Isomorphism-invariant graph fingerprint: cells connect to their row, column,
// box and (for clues) digit. Row/column nodes share a color so transposes,
// legal house permutations and digit relabeling all retain the fingerprint.
// Hash collisions can discard extra candidates, never admit equivalent ones.
fn fingerprint(grid: &[u8; 81]) -> Vec<u64> {
    let mut edges = vec![Vec::new(); 117];
    let mut colors = vec![0u64; 117];
    for i in 0..81 {
        colors[i] = u64::from(grid[i] != 0);
        let (r, c) = (i / 9, i % 9);
        let mut neighbors = vec![81 + r, 90 + c, 99 + r / 3 * 3 + c / 3];
        if grid[i] != 0 {
            neighbors.push(108 + grid[i] as usize - 1);
        }
        for j in neighbors {
            edges[i].push(j);
            edges[j].push(i);
        }
    }
    colors[81..99].fill(2);
    colors[99..108].fill(3);
    colors[108..].fill(4);
    for _ in 0..12 {
        colors = (0..117)
            .map(|i| {
                let mut neighbors: Vec<_> = edges[i].iter().map(|&j| colors[j]).collect();
                neighbors.sort_unstable();
                neighbors
                    .into_iter()
                    .fold(colors[i].wrapping_add(0x9e3779b97f4a7c15), |hash, v| {
                        (hash ^ v).wrapping_mul(0x100000001b3)
                    })
            })
            .collect();
    }
    colors.sort_unstable();
    colors
}
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<_> = std::env::args().collect();
    let mut output = std::io::BufWriter::new(std::fs::File::create(&args[2])?);
    writeln!(output, "difficulty,puzzle,solution")?;
    let mut counts = [0; 5];
    let mut seen = HashSet::new();
    for name in ["easy", "medium", "hard", "diabolical"] {
        for line in std::fs::read_to_string(format!("{}/{name}.txt", args[1]))?.lines() {
            let text = line.split_whitespace().nth(1).ok_or("Missing puzzle")?;
            if text.len() != 81 || !text.bytes().all(|b| b.is_ascii_digit()) {
                return Err("Invalid puzzle".into());
            }
            let cells: [u8; 81] = std::array::from_fn(|i| text.as_bytes()[i] - b'0');
            let grading = grade(&cells);
            let Some(rating) = grading.rating else {
                continue;
            };
            let level = rating.difficulty as usize;
            if counts[level] >= 1000 || !seen.insert(fingerprint(&cells)) {
                continue;
            }
            if count_solutions(&mut cells.clone(), 2) != 1 {
                return Err("Nonunique puzzle".into());
            }
            let solution: String = grading
                .completed
                .iter()
                .map(|v| char::from(b'0' + v))
                .collect();
            writeln!(output, "{:?},{text},{solution}", rating.difficulty)?;
            counts[level] += 1;
            if counts[level] % 100 == 0 {
                eprintln!("Accepted {counts:?}");
            }
            if counts == [1000; 5] {
                break;
            }
        }
        if counts == [1000; 5] {
            break;
        }
    }
    output.flush()?;
    if counts != [1000; 5] {
        return Err(format!("Insufficient rated puzzles: {counts:?}").into());
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn equivalent_puzzles_have_equal_fingerprints() {
        let text = include_str!("../puzzle_bank.csv")
            .lines()
            .nth(1)
            .unwrap()
            .split(',')
            .nth(1)
            .unwrap();
        let grid = std::array::from_fn(|i| text.as_bytes()[i] - b'0');
        let rows = [4, 3, 5, 8, 6, 7, 1, 2, 0];
        let cols = [2, 0, 1, 7, 8, 6, 5, 3, 4];
        let transformed = std::array::from_fn(|i| {
            let v = grid[rows[i % 9] * 9 + cols[i / 9]];
            if v == 0 {
                0
            } else {
                10 - v
            }
        });
        assert_eq!(fingerprint(&grid), fingerprint(&transformed));
        assert_ne!(fingerprint(&grid), fingerprint(&[0; 81]));
    }
}
