use bit_vec::BitVec;
use ndarray::Array2;
use rand::{Rng, seq::IteratorRandom};
use std::{cmp::Ordering, collections::BinaryHeap};

use crate::{Direction, Ruleset};

#[derive(Eq, PartialEq)]
struct Cell {
    entropy: usize,
    row: usize,
    col: usize,
}

impl Ord for Cell {
    fn cmp(&self, other: &Self) -> Ordering {
        // Lower entropy gets higher priority.
        other.entropy.cmp(&self.entropy)
    }
}

impl PartialOrd for Cell {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

pub struct WaveFunction {
    possible_tiles: Array2<BitVec>,
    entropies: Array2<usize>,
    neighbors: Array2<Vec<(usize, usize, Direction)>>,
    rows: usize,
    cols: usize,
    num_tiles: usize,
    rules: Ruleset,
}

impl WaveFunction {
    pub fn new(resolution: [usize; 2], rules: Ruleset) -> Self {
        let (rows, cols) = (resolution[0], resolution[1]);
        let num_tiles = rules.len();
        let initial = BitVec::from_elem(num_tiles, true);
        let possible_tiles = Array2::from_shape_fn((rows, cols), |_| initial.clone());
        let entropies = Array2::from_shape_fn((rows, cols), |_| num_tiles);
        let neighbors = Array2::from_shape_fn((rows, cols), |(r, c)| {
            let mut nbrs = Vec::new();
            if r > 0 {
                nbrs.push((r - 1, c, Direction::North));
            }
            if r < rows - 1 {
                nbrs.push((r + 1, c, Direction::South));
            }
            if c > 0 {
                nbrs.push((r, c - 1, Direction::West));
            }
            if c < cols - 1 {
                nbrs.push((r, c + 1, Direction::East));
            }
            nbrs
        });
        Self {
            possible_tiles,
            entropies,
            neighbors,
            rows,
            cols,
            num_tiles,
            rules,
        }
    }

    // Count true bits in a BitVec.
    fn count_possibilities(bv: &BitVec) -> usize {
        bv.iter().filter(|&b| b).count()
    }

    // Get indices of possibilities for a cell.
    fn get_possibilities(&self, i: usize, j: usize) -> Vec<usize> {
        self.possible_tiles[[i, j]]
            .iter()
            .enumerate()
            .filter(|&(_, b)| b)
            .map(|(idx, _)| idx)
            .collect()
    }

    // Bitwise union (OR) of two BitVecs.
    fn bitvec_union(a: &BitVec, b: &BitVec) -> BitVec {
        BitVec::from_fn(a.len(), |i| {
            a.get(i).unwrap_or(false) || b.get(i).unwrap_or(false)
        })
    }

    // Bitwise intersection (AND) of two BitVecs.
    fn bitvec_intersection(a: &BitVec, b: &BitVec) -> BitVec {
        BitVec::from_fn(a.len(), |i| {
            a.get(i).unwrap_or(false) && b.get(i).unwrap_or(false)
        })
    }

    // Accumulate allowed bits from neighbor's possibilities using Ruleset.
    fn allowed_mask_from_neighbor(&self, r: usize, c: usize, direction: Direction) -> BitVec {
        self.get_possibilities(r, c)
            .iter()
            .fold(BitVec::from_elem(self.num_tiles, false), |acc, &p| {
                Self::bitvec_union(&acc, &self.rules.allowed_mask(p, direction))
            })
    }

    // Iteratively collapse the wave.
    pub fn collapse<R: Rng>(&mut self, rng: &mut R) -> Option<Array2<usize>> {
        let mut heap = BinaryHeap::new();
        // Seed the heap with all unsolved cells.
        for i in 0..self.rows {
            for j in 0..self.cols {
                let count = Self::count_possibilities(&self.possible_tiles[[i, j]]);
                if count > 1 {
                    heap.push(Cell {
                        entropy: count,
                        row: i,
                        col: j,
                    });
                }
            }
        }

        while !self.is_solved() {
            if heap.is_empty() {
                return None;
            }
            let cell = heap.pop().unwrap();
            let current_entropy =
                Self::count_possibilities(&self.possible_tiles[[cell.row, cell.col]]);
            if current_entropy != cell.entropy || current_entropy <= 1 {
                continue;
            }
            let possibilities = self.get_possibilities(cell.row, cell.col);
            let &choice = possibilities.iter().choose(rng).unwrap();
            let mut new_vec = BitVec::from_elem(self.num_tiles, false);
            new_vec.set(choice, true);
            self.possible_tiles[[cell.row, cell.col]] = new_vec;
            self.entropies[[cell.row, cell.col]] = 1;

            if !self.propagate(cell.row, cell.col, &mut heap) {
                return None;
            }
        }
        Some(self.to_map())
    }

    fn is_solved(&self) -> bool {
        self.entropies.iter().all(|&entropy| entropy == 1)
    }

    fn to_map(&self) -> Array2<usize> {
        Array2::from_shape_fn((self.rows, self.cols), |(i, j)| {
            self.possible_tiles[[i, j]]
                .iter()
                .position(|b| b)
                .unwrap_or(0)
        })
    }

    // Propagate constraints iteratively.
    fn propagate(&mut self, row: usize, col: usize, heap: &mut BinaryHeap<Cell>) -> bool {
        let mut dirty = vec![(row, col)];
        while let Some((r, c)) = dirty.pop() {
            for &(nr, nc, direction) in &self.neighbors[[r, c]] {
                let current = &self.possible_tiles[[nr, nc]];
                let allowed = self.allowed_mask_from_neighbor(r, c, direction);
                let new_mask = Self::bitvec_intersection(current, &allowed);
                if new_mask != *current {
                    if new_mask.iter().all(|b| !b) {
                        return false;
                    }
                    self.possible_tiles[[nr, nc]] = new_mask.clone();
                    let new_entropy = Self::count_possibilities(&new_mask);
                    self.entropies[[nr, nc]] = new_entropy;
                    heap.push(Cell {
                        entropy: new_entropy,
                        row: nr,
                        col: nc,
                    });
                    dirty.push((nr, nc));
                }
            }
        }
        true
    }
}
