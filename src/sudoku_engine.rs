// ponytail: flat [u8; 81], 0=empty, 1-9=filled. ceiling: 9x9 only. upgrade: Grid<const N: usize> if variants needed.

use crate::serde_helpers::u8_81;
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug, Serialize, Deserialize)]
pub enum Difficulty {
    Easy,
    Medium,
    Hard,
    Expert,
    Master,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Board {
    #[serde(with = "u8_81")]
    pub cells: [u8; 81],
    #[serde(with = "u8_81")]
    pub solution: [u8; 81],
    pub difficulty: Difficulty,
    pub seed: u64,
    #[serde(default)]
    pub rating: Option<Rating>,
}

/// Versioned app-specific grading; these bands are not Sudoku Explainer scores.
pub const GRADER_VERSION: u8 = 1;
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Technique {
    NakedSingle,
    HiddenSingle,
    LockedCandidates,
    NakedSubset,
    HiddenSubset,
    XWing,
    XYWing,
    SimpleColoring,
    ShortChain,
    XChain,
}
impl Technique {
    pub fn difficulty(self) -> Difficulty {
        match self {
            Self::NakedSingle | Self::HiddenSingle => Difficulty::Easy,
            Self::LockedCandidates => Difficulty::Medium,
            Self::NakedSubset | Self::HiddenSubset => Difficulty::Hard,
            Self::XWing | Self::XYWing | Self::SimpleColoring | Self::ShortChain => {
                Difficulty::Expert
            }
            Self::XChain => Difficulty::Master,
        }
    }
    pub fn label(self) -> &'static str {
        match self {
            Self::NakedSingle => "Candidato único",
            Self::HiddenSingle => "Único na unidade",
            Self::LockedCandidates => "Candidatos bloqueados",
            Self::NakedSubset => "Pares/trios nus",
            Self::HiddenSubset => "Pares/trios ocultos",
            Self::XWing => "X-Wing",
            Self::XYWing => "XY-Wing",
            Self::SimpleColoring => "Coloração simples",
            Self::ShortChain => "Cadeia curta",
            Self::XChain => "Cadeia alternada",
        }
    }
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Rating {
    pub grader_version: u8,
    pub difficulty: Difficulty,
    pub strongest: Technique,
    pub step_counts: [u16; 10],
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LogicalStep {
    pub technique: Technique,
    pub placements: Vec<(usize, u8)>,
    pub eliminations: Vec<(usize, u16)>,
}
#[derive(Clone, Debug)]
pub struct Grading {
    pub rating: Option<Rating>,
    pub trace: Vec<LogicalStep>,
    pub completed: [u8; 81],
}
fn peers(a: usize, b: usize) -> bool {
    a != b && (a / 9 == b / 9 || a % 9 == b % 9 || (a / 27 == b / 27 && a % 9 / 3 == b % 9 / 3))
}
fn units() -> [[usize; 9]; 27] {
    std::array::from_fn(|u| {
        std::array::from_fn(|i| {
            if u < 9 {
                u * 9 + i
            } else if u < 18 {
                i * 9 + u - 9
            } else {
                let b = u - 18;
                (b / 3 * 3 + i / 3) * 9 + b % 3 * 3 + i % 3
            }
        })
    })
}
fn candidate_masks(grid: &[u8; 81]) -> [u16; 81] {
    std::array::from_fn(|i| {
        if grid[i] == 0 {
            candidates(grid, i / 9, i % 9)
                .iter()
                .fold(0, |mask, v| mask | (1 << (v - 1)))
        } else {
            0
        }
    })
}
fn elimination(
    technique: Technique,
    masks: &[u16; 81],
    cells: impl Iterator<Item = usize>,
    bits: u16,
) -> Option<LogicalStep> {
    let eliminations: Vec<_> = cells
        .filter_map(|i| {
            let removed = masks[i] & bits;
            (removed != 0).then_some((i, removed))
        })
        .collect();
    (!eliminations.is_empty()).then_some(LogicalStep {
        technique,
        placements: vec![],
        eliminations,
    })
}
fn singles(masks: &[u16; 81], houses: &[[usize; 9]; 27]) -> Option<LogicalStep> {
    if let Some(i) = (0..81).find(|&i| masks[i].count_ones() == 1) {
        return Some(LogicalStep {
            technique: Technique::NakedSingle,
            placements: vec![(i, masks[i].trailing_zeros() as u8 + 1)],
            eliminations: vec![],
        });
    }
    for house in houses {
        for v in 1..=9 {
            let cells: Vec<_> = house
                .iter()
                .copied()
                .filter(|&i| masks[i] & (1 << (v - 1)) != 0)
                .collect();
            if cells.len() == 1 {
                return Some(LogicalStep {
                    technique: Technique::HiddenSingle,
                    placements: vec![(cells[0], v)],
                    eliminations: vec![],
                });
            }
        }
    }
    None
}
fn locked(masks: &[u16; 81], houses: &[[usize; 9]; 27]) -> Option<LogicalStep> {
    for (a, source) in houses.iter().enumerate() {
        for v in 0..9 {
            let bit = 1 << v;
            let cells: Vec<_> = source
                .iter()
                .copied()
                .filter(|&i| masks[i] & bit != 0)
                .collect();
            if cells.len() < 2 {
                continue;
            }
            for (b, destination) in houses.iter().enumerate() {
                if a != b && cells.iter().all(|i| destination.contains(i)) {
                    if let Some(step) = elimination(
                        Technique::LockedCandidates,
                        masks,
                        destination.iter().copied().filter(|i| !source.contains(i)),
                        bit,
                    ) {
                        return Some(step);
                    }
                }
            }
        }
    }
    None
}
fn subsets(masks: &[u16; 81], houses: &[[usize; 9]; 27], hidden: bool) -> Option<LogicalStep> {
    for size in 2..=3 {
        for house in houses {
            for selection in 1u16..512 {
                if selection.count_ones() != size {
                    continue;
                }
                if hidden {
                    let positions = house.iter().enumerate().fold(0u16, |p, (j, &i)| {
                        if masks[i] & selection != 0 {
                            p | (1 << j)
                        } else {
                            p
                        }
                    });
                    if positions.count_ones() != size
                        || (0..9).any(|v| {
                            selection & (1 << v) != 0
                                && !house.iter().any(|&i| masks[i] & (1 << v) != 0)
                        })
                    {
                        continue;
                    }
                    if let Some(step) = elimination(
                        Technique::HiddenSubset,
                        masks,
                        house
                            .iter()
                            .enumerate()
                            .filter_map(|(j, &i)| (positions & (1 << j) != 0).then_some(i)),
                        !selection,
                    ) {
                        return Some(step);
                    }
                } else {
                    let selected: Vec<_> = house
                        .iter()
                        .enumerate()
                        .filter_map(|(j, &i)| (selection & (1 << j) != 0).then_some(i))
                        .collect();
                    if selected
                        .iter()
                        .any(|&i| masks[i].count_ones() < 2 || masks[i].count_ones() > size)
                    {
                        continue;
                    }
                    let union = selected.iter().fold(0, |a, &i| a | masks[i]);
                    if union.count_ones() != size {
                        continue;
                    }
                    if let Some(step) = elimination(
                        Technique::NakedSubset,
                        masks,
                        house.iter().copied().filter(|i| !selected.contains(i)),
                        union,
                    ) {
                        return Some(step);
                    }
                }
            }
        }
    }
    None
}
fn x_wing(masks: &[u16; 81]) -> Option<LogicalStep> {
    for transpose in [false, true] {
        let index = |row, col| {
            if transpose {
                col * 9 + row
            } else {
                row * 9 + col
            }
        };
        for v in 0..9 {
            let bit = 1 << v;
            let positions: [u16; 9] = std::array::from_fn(|r| {
                (0..9).fold(0, |p, c| {
                    if masks[index(r, c)] & bit != 0 {
                        p | (1 << c)
                    } else {
                        p
                    }
                })
            });
            for a in 0..9 {
                for b in a + 1..9 {
                    if positions[a].count_ones() == 2 && positions[a] == positions[b] {
                        let cells = (0..81).filter_map(|i| {
                            let r = i / 9;
                            let c = i % 9;
                            (r != a && r != b && positions[a] & (1 << c) != 0)
                                .then_some(index(r, c))
                        });
                        if let Some(step) = elimination(Technique::XWing, masks, cells, bit) {
                            return Some(step);
                        }
                    }
                }
            }
        }
    }
    None
}
fn xy_wing(masks: &[u16; 81]) -> Option<LogicalStep> {
    for pivot in 0..81 {
        if masks[pivot].count_ones() != 2 {
            continue;
        }
        let wings: Vec<_> = (0..81)
            .filter(|&i| {
                peers(pivot, i)
                    && masks[i].count_ones() == 2
                    && (masks[pivot] & masks[i]).count_ones() == 1
            })
            .collect();
        for (a, &left) in wings.iter().enumerate() {
            for &right in &wings[a + 1..] {
                let shared = masks[left] & masks[right];
                if shared.count_ones() == 1
                    && shared & masks[pivot] == 0
                    && (masks[left] | masks[right] | masks[pivot]).count_ones() == 3
                {
                    if let Some(step) = elimination(
                        Technique::XYWing,
                        masks,
                        (0..81).filter(|&i| i != pivot && peers(i, left) && peers(i, right)),
                        shared,
                    ) {
                        return Some(step);
                    }
                }
            }
        }
    }
    None
}
fn coloring(masks: &[u16; 81], houses: &[[usize; 9]; 27]) -> Option<LogicalStep> {
    for v in 0..9 {
        let bit = 1 << v;
        let mut links: [Vec<usize>; 81] = std::array::from_fn(|_| vec![]);
        for house in houses {
            let nodes: Vec<_> = house
                .iter()
                .copied()
                .filter(|&i| masks[i] & bit != 0)
                .collect();
            if nodes.len() == 2 {
                links[nodes[0]].push(nodes[1]);
                links[nodes[1]].push(nodes[0]);
            }
        }
        let mut visited = [false; 81];
        for root in 0..81 {
            if visited[root] || links[root].is_empty() {
                continue;
            }
            let mut colors = [None; 81];
            colors[root] = Some(false);
            let mut queue = vec![root];
            let mut cursor = 0;
            let mut consistent = true;
            while cursor < queue.len() {
                let i = queue[cursor];
                cursor += 1;
                visited[i] = true;
                for &j in &links[i] {
                    let next = !colors[i].unwrap();
                    if let Some(existing) = colors[j] {
                        if existing != next {
                            consistent = false;
                        }
                    } else {
                        colors[j] = Some(next);
                        queue.push(j);
                    }
                }
            }
            if !consistent {
                continue;
            }
            for color in [false, true] {
                let nodes: Vec<_> = queue
                    .iter()
                    .copied()
                    .filter(|&i| colors[i] == Some(color))
                    .collect();
                if nodes
                    .iter()
                    .enumerate()
                    .any(|(a, &i)| nodes[a + 1..].iter().any(|&j| peers(i, j)))
                {
                    if let Some(step) =
                        elimination(Technique::SimpleColoring, masks, nodes.into_iter(), bit)
                    {
                        return Some(step);
                    }
                }
            }
            let cells = (0..81).filter(|&i| {
                colors[i].is_none()
                    && [false, true].iter().all(|&color| {
                        queue
                            .iter()
                            .any(|&j| colors[j] == Some(color) && peers(i, j))
                    })
            });
            if let Some(step) = elimination(Technique::SimpleColoring, masks, cells, bit) {
                return Some(step);
            }
        }
    }
    None
}
/// Alternating strong/weak single-digit chains. Both endpoints cannot be false.
/// Three links are a short chain; longer chains are searched only after those.
fn x_chain(masks: &[u16; 81], houses: &[[usize; 9]; 27], long: bool) -> Option<LogicalStep> {
    for v in 0..9 {
        let bit = 1 << v;
        let nodes: Vec<_> = (0..81).filter(|&i| masks[i] & bit != 0).collect();
        let mut strong: [Vec<usize>; 81] = std::array::from_fn(|_| vec![]);
        for house in houses {
            let candidates: Vec<_> = house
                .iter()
                .copied()
                .filter(|&i| masks[i] & bit != 0)
                .collect();
            if candidates.len() == 2 {
                strong[candidates[0]].push(candidates[1]);
                strong[candidates[1]].push(candidates[0]);
            }
        }
        let mut budget = 10_000;
        for &start in &nodes {
            let mut path = vec![start];
            if let Some(step) = chain_walk(
                masks,
                bit,
                &nodes,
                &strong,
                &mut path,
                false,
                long,
                &mut budget,
            ) {
                return Some(step);
            }
            if budget == 0 {
                break;
            }
        }
    }
    None
}
#[allow(clippy::too_many_arguments)]
fn chain_walk(
    masks: &[u16; 81],
    bit: u16,
    nodes: &[usize],
    strong: &[Vec<usize>; 81],
    path: &mut Vec<usize>,
    weak: bool,
    long: bool,
    budget: &mut u32,
) -> Option<LogicalStep> {
    if *budget == 0 {
        return None;
    }
    *budget -= 1;
    let first = path[0];
    let last = *path.last().unwrap();
    let links = path.len() - 1;
    if weak && ((!long && links == 3) || (long && links >= 5)) {
        let technique = if long {
            Technique::XChain
        } else {
            Technique::ShortChain
        };
        if let Some(step) = elimination(
            technique,
            masks,
            nodes
                .iter()
                .copied()
                .filter(|i| !path.contains(i) && peers(*i, first) && peers(*i, last)),
            bit,
        ) {
            return Some(step);
        }
    }
    if links >= if long { 7 } else { 3 } {
        return None;
    }
    let next: Vec<_> = if weak {
        nodes.iter().copied().filter(|&i| peers(i, last)).collect()
    } else {
        strong[last].clone()
    };
    for i in next {
        if path.contains(&i) {
            continue;
        }
        path.push(i);
        let result = chain_walk(masks, bit, nodes, strong, path, !weak, long, budget);
        path.pop();
        if result.is_some() {
            return result;
        }
        if *budget == 0 {
            break;
        }
    }
    None
}

/// A deterministic logical solve trace. Unsupported/stalled puzzles stay unrated.
/// Candidate removals are retained between steps; the solution is never consulted.
pub fn grade(puzzle: &[u8; 81]) -> Grading {
    let mut result = Grading {
        rating: None,
        trace: vec![],
        completed: *puzzle,
    };
    if puzzle.iter().any(|&v| v > 9)
        || (0..81).any(|i| puzzle[i] != 0 && !is_valid_move(puzzle, i / 9, i % 9, puzzle[i]))
    {
        return result;
    }
    let houses = units();
    let mut masks = candidate_masks(puzzle);
    let mut strongest = Technique::NakedSingle;
    let mut counts = [0u16; 10];
    // Every step fills a cell or removes at least one of at most 729 candidates.
    for _ in 0..810 {
        if (0..81).any(|i| result.completed[i] == 0 && masks[i] == 0) {
            return result;
        }
        if result.completed.iter().all(|&v| v != 0) {
            if (0..81).all(|i| is_valid_move(&result.completed, i / 9, i % 9, result.completed[i]))
            {
                result.rating = Some(Rating {
                    grader_version: GRADER_VERSION,
                    difficulty: strongest.difficulty(),
                    strongest,
                    step_counts: counts,
                });
            }
            return result;
        }
        let step = singles(&masks, &houses)
            .or_else(|| locked(&masks, &houses))
            .or_else(|| subsets(&masks, &houses, false))
            .or_else(|| subsets(&masks, &houses, true))
            .or_else(|| x_wing(&masks))
            .or_else(|| xy_wing(&masks))
            .or_else(|| coloring(&masks, &houses))
            .or_else(|| x_chain(&masks, &houses, false))
            .or_else(|| x_chain(&masks, &houses, true));
        let Some(step) = step else {
            return result;
        };
        if (step.technique as usize) > (strongest as usize) {
            strongest = step.technique;
        }
        counts[step.technique as usize] += 1;
        for &(i, v) in &step.placements {
            result.completed[i] = v;
            masks[i] = 0;
            for (j, mask) in masks.iter_mut().enumerate() {
                if peers(i, j) {
                    *mask &= !(1 << (v - 1));
                }
            }
        }
        for &(i, bits) in &step.eliminations {
            masks[i] &= !bits;
        }
        result.trace.push(step);
    }
    result
}

/// MRV search is for completion/uniqueness only, never human difficulty.
fn search_cell(grid: &[u8; 81]) -> Option<(usize, Vec<u8>)> {
    let mut best: Option<(usize, Vec<u8>)> = None;
    for i in 0..81 {
        if grid[i] != 0 {
            continue;
        }
        let values = candidates(grid, i / 9, i % 9);
        if values.is_empty() {
            return Some((i, values));
        }
        if best.as_ref().is_none_or(|(_, v)| values.len() < v.len()) {
            best = Some((i, values));
            if best.as_ref().unwrap().1.len() == 1 {
                break;
            }
        }
    }
    best
}
fn solve(grid: &mut [u8; 81], rng: &mut Rand, budget: &mut u32) -> bool {
    if *budget == 0 {
        return false;
    }
    *budget -= 1;
    if let Some((idx, mut values)) = search_cell(grid) {
        shuffle(&mut values, rng);
        for v in values {
            grid[idx] = v;
            if solve(grid, rng, budget) {
                return true;
            }
            grid[idx] = 0;
        }
        false
    } else {
        true
    }
}
fn count_bounded(grid: &mut [u8; 81], limit: u32, budget: &mut u32) -> Option<u32> {
    if *budget == 0 {
        return None;
    }
    *budget -= 1;
    if let Some((idx, values)) = search_cell(grid) {
        let mut count = 0;
        for v in values {
            grid[idx] = v;
            let result = count_bounded(grid, limit - count, budget);
            grid[idx] = 0;
            count += result?;
            if count >= limit {
                return Some(count);
            }
        }
        Some(count)
    } else {
        Some(1)
    }
}
#[cfg(test)]
fn count_solutions(grid: &mut [u8; 81], limit: u32) -> u32 {
    count_bounded(grid, limit, &mut 1_000_000).expect("Test uniqueness budget exhausted")
}

/// Valid candidates for a cell (numbers not in same row/col/box).
pub fn candidates(grid: &[u8; 81], row: usize, col: usize) -> Vec<u8> {
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

const SEARCH_BUDGET: u32 = 20_000;
const GENERATION_ATTEMPTS: usize = 2;

/// Bounds are secondary density constraints. Failure to reach them is explicit.
fn generate_clues(
    range: std::ops::RangeInclusive<u8>,
    rng: &mut Rand,
) -> Option<([u8; 81], [u8; 81])> {
    let min = *range.start();
    let max = *range.end();
    if min < 17 || max > 81 || min > max {
        return None;
    }
    let target = min + (rng.next() % (max - min + 1) as u64) as u8;
    let mut solution = [0; 81];
    let mut fill_budget = SEARCH_BUDGET;
    if !solve(&mut solution, rng, &mut fill_budget) {
        return None;
    }
    let mut puzzle = solution;
    let mut count = 81;
    let mut pairs: Vec<_> = (0..=40).map(|i| (i, 80 - i)).collect();
    shuffle(&mut pairs, rng);
    for (a, b) in pairs {
        if count == target {
            break;
        }
        let removed = if a == b { 1 } else { 2 };
        if count - removed < target {
            continue;
        }
        let saved = (puzzle[a], puzzle[b]);
        puzzle[a] = 0;
        puzzle[b] = 0;
        let mut uniqueness_budget = SEARCH_BUDGET;
        if count_bounded(&mut puzzle.clone(), 2, &mut uniqueness_budget) == Some(1) {
            count -= removed;
        } else {
            puzzle[a] = saved.0;
            puzzle[b] = saved.1;
        }
    }
    range.contains(&count).then_some((puzzle, solution))
}
fn parse_puzzle(s: &str) -> [u8; 81] {
    let mut grid = [0; 81];
    assert_eq!(s.len(), 81, "Invalid bundled puzzle length");
    for (i, b) in s.bytes().enumerate() {
        assert!(b.is_ascii_digit());
        grid[i] = b - b'0';
    }
    grid
}
fn bank(difficulty: Difficulty) -> Vec<([u8; 81], [u8; 81])> {
    let name = format!("{difficulty:?}");
    include_str!("puzzle_bank.csv")
        .lines()
        .skip(1)
        .filter_map(|line| {
            let fields: Vec<_> = line.split(',').collect();
            (fields[0] == name).then(|| (parse_puzzle(fields[1]), parse_puzzle(fields[2])))
        })
        .collect()
}
fn transform(grid: &[u8; 81], rng: &mut Rand) -> [u8; 81] {
    let mut digits = [1, 2, 3, 4, 5, 6, 7, 8, 9];
    shuffle(&mut digits, rng);
    let rotation = rng.next() % 4;
    let reflection = rng.next() % 2 == 1;
    std::array::from_fn(|i| {
        let (mut row, mut col) = (i / 9, i % 9);
        for _ in 0..rotation {
            (row, col) = (col, 8 - row);
        }
        if reflection {
            col = 8 - col;
        }
        let v = grid[row * 9 + col];
        if v == 0 {
            0
        } else {
            digits[v as usize - 1]
        }
    })
}
fn rated_board(
    cells: [u8; 81],
    solution: [u8; 81],
    difficulty: Difficulty,
    seed: u64,
) -> Option<Board> {
    let grading = grade(&cells);
    let rating = grading.rating?;
    if rating.difficulty != difficulty || grading.completed != solution {
        return None;
    }
    Some(Board {
        cells,
        solution,
        difficulty: rating.difficulty,
        seed,
        rating: Some(rating),
    })
}
/// Replay is deterministic for generator/grader v1 and this bundled corpus.
/// All fallback puzzles are regraded; easier or unsupported puzzles are rejected.
pub fn generate_seeded(difficulty: Difficulty, seed: u64) -> Board {
    let seed = if seed == 0 {
        0xDEAD_BEEF_CAFE_BABE
    } else {
        seed
    };
    let mut rng = Rand(seed);
    if difficulty <= Difficulty::Hard {
        let range = if difficulty == Difficulty::Easy {
            34..=46
        } else {
            24..=36
        };
        for _ in 0..GENERATION_ATTEMPTS {
            if let Some((cells, solution)) = generate_clues(range.clone(), &mut rng) {
                if let Some(board) = rated_board(cells, solution, difficulty, seed) {
                    return board;
                }
            }
        }
    }
    let corpus = bank(difficulty);
    let index = rng.next() as usize % corpus.len();
    let (cells, solution) = corpus[index];
    for _ in 0..GENERATION_ATTEMPTS {
        // Apply the same random transform to puzzle and solution.
        let mut solution_rng = Rand(rng.0);
        let transformed_cells = transform(&cells, &mut rng);
        let transformed_solution = transform(&solution, &mut solution_rng);
        if let Some(board) = rated_board(transformed_cells, transformed_solution, difficulty, seed)
        {
            return board;
        }
    }
    rated_board(cells, solution, difficulty, seed)
        .expect("Bundled puzzle must retain its verified rating")
}
pub fn generate(difficulty: Difficulty) -> Board {
    generate_seeded(difficulty, Rand::new().0)
}

/// Simple xorshift PRNG — no need for `rand` dep just for shuffle.
struct Rand(u64);

impl Rand {
    fn new() -> Self {
        // ponytail: xorshift seeded from Math.random(). ceiling: not cryptographically random. upgrade: getrandom crate if seed quality matters.
        #[cfg(target_arch = "wasm32")]
        let seed = (js_sys::Math::random() * (u64::MAX as f64)) as u64;
        #[cfg(not(target_arch = "wasm32"))]
        let seed = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_nanos() as u64)
            .unwrap_or(0xDEAD_BEEF_CAFE_BABE);
        Rand(if seed == 0 {
            0xDEAD_BEEF_CAFE_BABE
        } else {
            seed
        })
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
        let mut grid = [0; 81];
        let mut budget = SEARCH_BUDGET;
        assert!(solve(&mut grid, &mut Rand(42), &mut budget));
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
        let board = generate(Difficulty::Easy);
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

    fn assert_generated_invariants(board: &Board) {
        assert!(board.cells.iter().all(|&v| v <= 9));
        assert!(board.solution.iter().all(|&v| (1..=9).contains(&v)));
        for i in 0..81 {
            assert!(board.cells[i] == 0 || board.cells[i] == board.solution[i]);
            assert_eq!(board.cells[i] == 0, board.cells[80 - i] == 0);
            let mut without_cell = board.solution;
            without_cell[i] = 0;
            assert!(is_valid_move(
                &without_cell,
                i / 9,
                i % 9,
                board.solution[i]
            ));
        }
        let mut puzzle = board.cells;
        assert_eq!(count_solutions(&mut puzzle, 2), 1);
        assert_eq!(
            puzzle, board.cells,
            "Solution counting must restore the grid"
        );
    }

    #[test]
    fn generated_puzzles_preserve_rules_and_uniqueness_at_every_setting() {
        for difficulty in [
            Difficulty::Easy,
            Difficulty::Medium,
            Difficulty::Hard,
            Difficulty::Expert,
            Difficulty::Master,
        ] {
            for _ in 0..3 {
                assert_generated_invariants(&generate(difficulty));
            }
        }
    }

    /// Diagnostic only: requested labels are not assertions of human difficulty.
    /// Run in release mode; timings measure generation, excluding invariant checks.
    #[test]
    #[ignore = "manual generation audit; emits CSV with machine-dependent timings"]
    fn audit_difficulty_settings() {
        println!("AUDIT,requested,run,clues,engine_label,seed,generation_ms,puzzle,solution");
        for difficulty in [
            Difficulty::Easy,
            Difficulty::Medium,
            Difficulty::Hard,
            Difficulty::Expert,
            Difficulty::Master,
        ] {
            for run in 0..20 {
                let started = std::time::Instant::now();
                let board = generate(difficulty);
                let elapsed = started.elapsed().as_secs_f64() * 1000.0;
                assert_generated_invariants(&board);
                println!(
                    "AUDIT,{difficulty:?},{run},{},{:?},{},{elapsed:.3},{},{}",
                    board.cells.iter().filter(|&&v| v != 0).count(),
                    board.difficulty,
                    board.seed,
                    board
                        .cells
                        .iter()
                        .map(|v| char::from(b'0' + v))
                        .collect::<String>(),
                    board
                        .solution
                        .iter()
                        .map(|v| char::from(b'0' + v))
                        .collect::<String>(),
                );
            }
        }
    }
    fn assert_trace_sound(board: &Board, grading: &Grading) {
        for step in &grading.trace {
            for &(i, v) in &step.placements {
                assert_eq!(v, board.solution[i]);
            }
            for &(i, bits) in &step.eliminations {
                assert_eq!(
                    bits & (1 << (board.solution[i] - 1)),
                    0,
                    "Unsound {:?} at {i}",
                    step.technique
                );
            }
        }
        if grading.rating.is_some() {
            assert_eq!(grading.completed, board.solution);
        }
    }

    #[test]
    fn logical_trace_is_sound_on_recorded_corpus() {
        for line in include_str!("../docs/difficulty-native.csv")
            .lines()
            .skip(1)
        {
            let columns: Vec<_> = line.split(',').collect();
            let board = Board {
                cells: parse_grid(columns[8]),
                solution: parse_grid(columns[9]),
                difficulty: Difficulty::Easy,
                seed: 1,
                rating: None,
            };
            assert_trace_sound(&board, &grade(&board.cells));
        }
        assert!(grade(&[0; 81]).rating.is_none());
        assert!(grade(&[1; 81]).rating.is_none());
    }
    fn parse_grid(s: &str) -> [u8; 81] {
        let mut grid = [0; 81];
        assert_eq!(s.len(), 81);
        for (i, b) in s.bytes().enumerate() {
            grid[i] = b - b'0';
        }
        grid
    }

    #[test]
    #[ignore = "offline self-generated grading corpus search"]
    fn collect_grading_fixtures() {
        let mut found = [0; 5];
        for attempt in 0..20000 {
            let Some((cells, solution)) = generate_clues(22..=34, &mut Rand::new()) else {
                continue;
            };
            let board = Board {
                cells,
                solution,
                difficulty: Difficulty::Easy,
                seed: 1,
                rating: None,
            };
            let grading = grade(&board.cells);
            assert_trace_sound(&board, &grading);
            if let Some(rating) = grading.rating {
                let level = rating.difficulty as usize;
                if found[level] < 8 {
                    println!(
                        "FIXTURE,{:?},{},{},{:?}",
                        rating.difficulty,
                        board
                            .cells
                            .iter()
                            .map(|v| char::from(b'0' + v))
                            .collect::<String>(),
                        board
                            .solution
                            .iter()
                            .map(|v| char::from(b'0' + v))
                            .collect::<String>(),
                        rating.strongest
                    );
                    found[level] += 1;
                }
            }
            if found.iter().all(|&n| n >= 8) {
                break;
            }
            if attempt % 1000 == 0 {
                eprintln!("Fixture search {attempt}: {found:?}");
            }
        }
        assert!(
            found.iter().all(|&n| n >= 8),
            "Missing fixture levels: {found:?}"
        );
    }
    #[test]
    fn bundled_puzzles_have_verified_unique_technique_ratings() {
        for difficulty in [
            Difficulty::Easy,
            Difficulty::Medium,
            Difficulty::Hard,
            Difficulty::Expert,
            Difficulty::Master,
        ] {
            let corpus = bank(difficulty);
            assert!(corpus.len() >= 2);
            for (cells, solution) in corpus {
                let board =
                    rated_board(cells, solution, difficulty, 1).expect("Catalog rating mismatch");
                assert_generated_invariants(&board);
                let grading = grade(&cells);
                assert_trace_sound(&board, &grading);
                if difficulty == Difficulty::Master {
                    assert_eq!(grading.rating.unwrap().strongest, Technique::XChain);
                }
            }
        }
    }
    #[test]
    fn every_selected_level_replays_and_matches_its_rating() {
        for difficulty in [
            Difficulty::Easy,
            Difficulty::Medium,
            Difficulty::Hard,
            Difficulty::Expert,
            Difficulty::Master,
        ] {
            for seed in [0, 1, 42, 123456789, u64::MAX] {
                let a = generate_seeded(difficulty, seed);
                let b = generate_seeded(difficulty, seed);
                assert_eq!(a.cells, b.cells);
                assert_eq!(a.solution, b.solution);
                assert_eq!(a.rating, b.rating);
                assert_eq!(a.difficulty, difficulty);
                assert_eq!(a.rating.as_ref().unwrap().difficulty, difficulty);
                assert_eq!(grade(&a.cells).rating, a.rating);
                assert_generated_invariants(&a);
            }
        }
    }
    #[test]
    fn clue_bounds_center_and_search_exhaustion_are_explicit() {
        assert!(generate_clues(16..=20, &mut Rand(1)).is_none());
        assert!(generate_clues(50..=40, &mut Rand(1)).is_none());
        assert!(generate_clues(40..=82, &mut Rand(1)).is_none());
        let mut removed_center = false;
        for seed in 1..=20 {
            for range in [40..=40, 32..=38, 26..=31, 17..=19] {
                if let Some((cells, _)) = generate_clues(range.clone(), &mut Rand(seed)) {
                    assert!(range.contains(&(cells.iter().filter(|&&v| v != 0).count() as u8)));
                    if range == (40..=40) {
                        assert_eq!(cells[40], 0);
                        removed_center = true;
                    }
                }
            }
        }
        assert!(removed_center);
        let mut puzzle = [0; 81];
        let original = puzzle;
        assert_eq!(count_bounded(&mut puzzle, 2, &mut 1), None);
        assert_eq!(puzzle, original, "Exhausted search must restore its input");
    }
    #[test]
    fn deduction_patterns_and_near_misses() {
        let houses = units();
        let mut masks = [511; 81];
        masks[0] = 3;
        masks[1] = 3;
        let pair = subsets(&masks, &houses, false).unwrap();
        assert!(pair.eliminations.contains(&(2, 3)));
        masks[1] = 7;
        assert!(!subsets(&masks, &houses, false).is_some_and(|s| s.eliminations.contains(&(2, 3))));
        let mut masks = [511; 81];
        for mask in &mut masks[2..9] {
            *mask &= !1;
        }
        assert!(locked(&masks, &houses)
            .unwrap()
            .eliminations
            .contains(&(9, 1)));
        let mut masks = [511; 81];
        for row in [0, 3] {
            for col in 0..9 {
                if col != 1 && col != 5 {
                    masks[row * 9 + col] &= !1;
                }
            }
        }
        assert!(x_wing(&masks).unwrap().eliminations.contains(&(10, 1)));
        let mut masks = [511; 81];
        masks[0] = 3;
        masks[4] = 5;
        masks[27] = 6;
        assert!(xy_wing(&masks).unwrap().eliminations.contains(&(31, 4)));
        masks[27] = 7;
        assert!(xy_wing(&masks).is_none());
    }
}
