use bitvec::prelude::*;
use ndarray::{Array2, s};
use photo::ImageRGBA;
use rand::{prelude::*, rng};
use std::f64;

use crate::TileSet;

pub struct WaveFunction<'a> {
    tile_set: &'a TileSet,
    states: Array2<BitVec>,
    rng: ThreadRng,
}

impl<'a> WaveFunction<'a> {
    pub fn new(tile_set: &'a TileSet, resolution: [usize; 2]) -> Self {
        debug_assert!(resolution[0] > 0 && resolution[1] > 0);
        Self {
            tile_set,
            states: Array2::from_elem(resolution, BitVec::repeat(true, tile_set.tiles.len())),
            rng: rng(),
        }
    }

    /// Returns true if the given pixel is considered a wildcard.
    fn is_wildcard(pixel: ndarray::ArrayView1<u8>, wildcard_colors: &[[u8; 4]]) -> bool {
        if pixel[3] == 0 {
            return true;
        }
        for &wc in wildcard_colors {
            if pixel[0] == wc[0] && pixel[1] == wc[1] && pixel[2] == wc[2] && pixel[3] == wc[3] {
                return true;
            }
        }
        false
    }

    /// Constructs a new WaveFunction from a tile set, an initialization image, and wildcard colours.
    /// Returns None if any cell has no valid candidate tiles.
    pub fn new_from_image(
        tile_set: &'a TileSet,
        init_image: &ImageRGBA<u8>,
        wildcard_colors: &[[u8; 4]],
    ) -> Option<Self> {
        // Precompute candidate (transformed) tile images.
        let tile_images: Vec<_> = tile_set
            .tiles
            .iter()
            .map(|tile| {
                tile_set.patterns[tile.pattern_index]
                    .image
                    .clone()
                    .transform(tile.transformation)
            })
            .collect();

        let (rows, cols) = (init_image.height(), init_image.width());

        // Each init image pixel becomes a cell.
        let mut states =
            Array2::from_shape_fn((rows, cols), |_| BitVec::repeat(true, tile_set.tiles.len()));

        for i in 0..rows {
            for j in 0..cols {
                let init_col = init_image.data.slice(s![i, j, ..]);
                if Self::is_wildcard(init_col, wildcard_colors) {
                    continue;
                } else {
                    // Create the mask of valid candidate tiles for the cell.
                    let mut mask = BitVec::repeat(false, tile_set.tiles.len());
                    for (k, tile_image) in tile_images.iter().enumerate() {
                        // Check the centre colour.
                        let tile_centre_col = tile_image.data.slice(s![1, 1, ..]);
                        if tile_centre_col == init_col {
                            mask.set(k, true);
                        }
                    }
                    if mask.count_ones() == 0 {
                        eprintln!("No candidate tile found for cell ({}, {})", i, j);
                        return None;
                    }
                    states[(i, j)] = mask;
                }
            }
        }

        Some(Self {
            tile_set,
            states,
            rng: rand::rng(),
        })
    }

    /// Checks if every cell is collapsed (exactly one possibility).
    fn is_fully_collapsed(&self) -> bool {
        self.states.iter().all(|cell| cell.count_ones() == 1)
    }

    /// Compute the Shannon entropy of a cell based on the frequencies of possible tiles.
    fn compute_entropy(&self, cell: &BitVec) -> f64 {
        let mut poss = Vec::new();
        for (i, bit) in cell.iter().enumerate() {
            if *bit {
                poss.push(i);
            }
        }
        if poss.len() <= 1 {
            return 0.0;
        }
        let mut total = 0.0;
        let mut weights = Vec::new();
        for &i in &poss {
            // Use the frequency of the underlying pattern.
            let pattern_idx = self.tile_set.tiles[i].pattern_index;
            let weight = self.tile_set.patterns[pattern_idx].frequency as f64;
            total += weight;
            weights.push(weight);
        }
        let mut entropy = 0.0;
        for w in weights {
            let p = w / total;
            entropy -= p * p.ln();
        }
        entropy
    }

    /// Select the cell (i,j) with minimal nonzero entropy.
    fn select_cell_with_min_entropy(&self) -> Option<(usize, usize)> {
        let (rows, cols) = self.states.dim();
        let mut min_entropy = f64::INFINITY;
        let mut chosen = None;
        for i in 0..rows {
            for j in 0..cols {
                let cell = &self.states[(i, j)];
                if cell.count_ones() > 1 {
                    let entropy = self.compute_entropy(cell);
                    if entropy < min_entropy {
                        min_entropy = entropy;
                        chosen = Some((i, j));
                    }
                }
            }
        }
        chosen
    }

    /// Update neighbor cell at (i, j) given a source cell’s possibilities and a relative direction.
    /// `direction` is the direction from the source cell to this neighbor (0: North, 1: East, 2: South, 3: West).
    /// Returns true if the neighbor cell’s possibilities were revised.
    fn update_cell(
        &mut self,
        i: usize,
        j: usize,
        direction: usize,
        source_possible: &BitVec,
    ) -> bool {
        let cell = &mut self.states[(i, j)];
        let original = cell.clone();
        for b in 0..cell.len() {
            if cell[b] {
                let mut supported = false;
                // Check if any possibility in the source cell supports `b`.
                for a in 0..source_possible.len() {
                    if source_possible[a] && self.tile_set.tiles[a].neighbours[direction][b] {
                        supported = true;
                        break;
                    }
                }
                if !supported {
                    cell.set(b, false);
                }
            }
        }
        original != *cell
    }

    /// Propagate constraints from all cells until no further updates.
    /// Returns false if a contradiction (an empty cell) is found.
    fn propagate(&mut self) -> bool {
        let (rows, cols) = self.states.dim();
        let mut queue = Vec::new();
        // Start with all cells (a simple initialization).
        for i in 0..rows {
            for j in 0..cols {
                queue.push((i, j));
            }
        }
        while let Some((i, j)) = queue.pop() {
            let current = self.states[(i, j)].clone();
            // North neighbor: relative direction = 0.
            if i > 0 {
                if self.update_cell(i - 1, j, 0, &current) {
                    queue.push((i - 1, j));
                }
                if self.states[(i - 1, j)].count_ones() == 0 {
                    return false;
                }
            }
            // East neighbor: direction = 1.
            if j + 1 < cols {
                if self.update_cell(i, j + 1, 1, &current) {
                    queue.push((i, j + 1));
                }
                if self.states[(i, j + 1)].count_ones() == 0 {
                    return false;
                }
            }
            // South neighbor: direction = 2.
            if i + 1 < rows {
                if self.update_cell(i + 1, j, 2, &current) {
                    queue.push((i + 1, j));
                }
                if self.states[(i + 1, j)].count_ones() == 0 {
                    return false;
                }
            }
            // West neighbor: direction = 3.
            if j > 0 {
                if self.update_cell(i, j - 1, 3, &current) {
                    queue.push((i, j - 1));
                }
                if self.states[(i, j - 1)].count_ones() == 0 {
                    return false;
                }
            }
        }
        true
    }

    /// Collapse the cell with the lowest entropy using backtracking.
    /// Returns true if the entire grid is successfully collapsed.
    pub fn collapse(&mut self) -> bool {
        if self.is_fully_collapsed() {
            return true;
        }
        let (i, j) = match self.select_cell_with_min_entropy() {
            Some(coord) => coord,
            None => return true,
        };
        // Collect the current possibilities for the chosen cell.
        let possibilities: Vec<usize> = (0..self.tile_set.tiles.len())
            .filter(|&k| self.states[(i, j)][k])
            .collect();
        if possibilities.is_empty() {
            return false;
        }
        // Backup current state for backtracking.
        let backup = self.states.clone();
        // Compute weighted probabilities based on tile frequency.
        let mut total = 0.0;
        let mut weights = Vec::new();
        for &choice in &possibilities {
            let pattern_idx = self.tile_set.tiles[choice].pattern_index;
            let w = self.tile_set.patterns[pattern_idx].frequency as f64;
            total += w;
            weights.push(w);
        }
        // Choose a possibility randomly weighted.
        let mut r: f64 = self.rng.random_range(0.0..total);
        let mut chosen = possibilities[0];
        for (&choice, &w) in possibilities.iter().zip(weights.iter()) {
            r -= w;
            if r <= 0.0 {
                chosen = choice;
                break;
            }
        }
        // Collapse the cell to the chosen possibility.
        self.states[(i, j)].fill(false);
        self.states[(i, j)].set(chosen, true);
        // Propagate constraints.
        if self.propagate() && self.collapse() {
            return true;
        }
        // Backtracking: restore state and eliminate the chosen possibility.
        self.states = backup;
        self.states[(i, j)].set(chosen, false);
        if self.propagate() && self.collapse() {
            return true;
        }
        false
    }

    /// Returns the final state as a 2D array of tile indices.
    pub fn state(&self) -> Array2<usize> {
        let (rows, cols) = self.states.dim();
        let mut result = Array2::zeros((rows, cols));
        for i in 0..rows {
            for j in 0..cols {
                for k in 0..self.tile_set.tiles.len() {
                    if self.states[(i, j)][k] {
                        result[(i, j)] = k;
                        break;
                    }
                }
            }
        }
        result
    }
}
