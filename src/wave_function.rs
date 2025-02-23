use ndarray::{Array2, Array3};
use rand::prelude::SliceRandom;
use std::collections::{HashSet, VecDeque};

use crate::{Direction, Ruleset};

// Get valid neighbors with associated direction (from current cell’s coordinates).
fn get_neighbors(r: usize, c: usize, rows: usize, cols: usize) -> Vec<(usize, usize, Direction)> {
    let mut neighbors = Vec::new();
    if r > 0 {
        neighbors.push((r - 1, c, Direction::North));
    }
    if r < rows - 1 {
        neighbors.push((r + 1, c, Direction::South));
    }
    if c > 0 {
        neighbors.push((r, c - 1, Direction::West));
    }
    if c < cols - 1 {
        neighbors.push((r, c + 1, Direction::East));
    }
    neighbors
}

pub struct WaveFunction {
    possible_tiles: Array3<bool>, // (rows, cols, num_tiles)
    rules: Ruleset,
    rows: usize,
    cols: usize,
    num_tiles: usize,
}

impl WaveFunction {
    pub fn new(resolution: [usize; 2], rules: Ruleset) -> Self {
        let (rows, cols) = (resolution[0], resolution[1]);
        let num_tiles = rules.rules.len();
        let possible_tiles = Array3::from_elem((rows, cols, num_tiles), true);
        Self {
            possible_tiles,
            rules,
            rows,
            cols,
            num_tiles,
        }
    }

    // Main collapse function using backtracking.
    pub fn collapse(&mut self) -> Option<Array2<usize>> {
        if self.is_solved() {
            return Some(self.to_map());
        }

        // Select cell with minimal entropy (>1 possibility).
        let (r, c) = self.find_min_entropy_cell()?;
        // Gather valid possibilities and shuffle them.
        let mut possibilities: Vec<usize> = (0..self.num_tiles)
            .filter(|&p| self.possible_tiles[[r, c, p]])
            .collect();
        possibilities.shuffle(&mut rand::rng());

        for choice in possibilities {
            // Backup current state for backtracking.
            let backup = self.possible_tiles.clone();
            // Collapse cell (r,c) to the chosen possibility.
            for p in 0..self.num_tiles {
                self.possible_tiles[[r, c, p]] = p == choice;
            }
            let mut dirty = VecDeque::new();
            dirty.push_back((r, c));

            if self.propagate(&mut dirty) {
                if let Some(map) = self.collapse() {
                    return Some(map);
                }
            }
            // Backtrack.
            self.possible_tiles = backup;
        }
        None
    }

    fn is_solved(&self) -> bool {
        (0..self.rows).all(|i| (0..self.cols).all(|j| self.count_possibilities(i, j) == 1))
    }

    fn to_map(&self) -> Array2<usize> {
        let mut map = Array2::zeros((self.rows, self.cols));
        for i in 0..self.rows {
            for j in 0..self.cols {
                for p in 0..self.num_tiles {
                    if self.possible_tiles[[i, j, p]] {
                        map[[i, j]] = p;
                        break;
                    }
                }
            }
        }
        map
    }

    fn count_possibilities(&self, i: usize, j: usize) -> usize {
        (0..self.num_tiles)
            .filter(|&p| self.possible_tiles[[i, j, p]])
            .count()
    }

    fn find_min_entropy_cell(&self) -> Option<(usize, usize)> {
        let mut min_entropy = usize::MAX;
        let mut cell = None;
        for i in 0..self.rows {
            for j in 0..self.cols {
                let count = self.count_possibilities(i, j);
                if count > 1 && count < min_entropy {
                    min_entropy = count;
                    cell = Some((i, j));
                }
            }
        }
        cell
    }

    // Propagate constraints using a dirty list.
    fn propagate(&mut self, dirty: &mut VecDeque<(usize, usize)>) -> bool {
        while let Some((r, c)) = dirty.pop_front() {
            for (nr, nc, dir) in get_neighbors(r, c, self.rows, self.cols) {
                if self.update_cell(nr, nc, r, c, dir) {
                    if self.count_possibilities(nr, nc) == 0 {
                        return false;
                    }
                    dirty.push_back((nr, nc));
                }
            }
        }
        true
    }

    // Update neighbor cell (nr, nc) based on constraints from (r, c).
    fn update_cell(
        &mut self,
        nr: usize,
        nc: usize,
        r: usize,
        c: usize,
        direction: Direction,
    ) -> bool {
        let mut changed = false;
        // Build union of allowed neighbor types from cell (r, c).
        let mut allowed = HashSet::new();
        for p in 0..self.num_tiles {
            if self.possible_tiles[[r, c, p]] {
                let rule = &self.rules.rules[p];
                match direction {
                    Direction::North => allowed.extend(rule.north.iter().copied()),
                    Direction::South => allowed.extend(rule.south.iter().copied()),
                    Direction::East => allowed.extend(rule.east.iter().copied()),
                    Direction::West => allowed.extend(rule.west.iter().copied()),
                }
            }
        }
        // Remove possibilities in (nr, nc) that aren't allowed.
        for poss in 0..self.num_tiles {
            if self.possible_tiles[[nr, nc, poss]] && !allowed.contains(&poss) {
                self.possible_tiles[[nr, nc, poss]] = false;
                changed = true;
            }
        }
        changed
    }
}
