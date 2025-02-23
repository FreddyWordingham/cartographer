use bit_vec::BitVec;
use ndarray::Array2;
use serde::{Deserialize, Serialize};

use crate::{Direction, Rule};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Ruleset {
    pub rules: Vec<Rule>,
}

impl Ruleset {
    /// Construct a new Ruleset from an example index map.
    pub fn new(map: &Array2<usize>) -> Self {
        let max_index = *map.iter().max().unwrap();
        let num_tiles = max_index + 1;
        let mut rules: Vec<Rule> = (0..num_tiles).map(|_| Rule::new(num_tiles)).collect();

        let (height, width) = map.dim();
        for ((i, j), &tile) in map.indexed_iter() {
            if i > 0 {
                let neighbor_tile = map[[i - 1, j]];
                rules[tile].north.set(neighbor_tile, true);
            }
            if i < height - 1 {
                let neighbor_tile = map[[i + 1, j]];
                rules[tile].south.set(neighbor_tile, true);
            }
            if j > 0 {
                let neighbor_tile = map[[i, j - 1]];
                rules[tile].west.set(neighbor_tile, true);
            }
            if j < width - 1 {
                let neighbor_tile = map[[i, j + 1]];
                rules[tile].east.set(neighbor_tile, true);
            }
        }
        Self { rules }
    }

    pub fn load(filepath: &str) -> Self {
        let rules: Vec<Rule> = serde_yaml::from_str(&std::fs::read_to_string(filepath).unwrap())
            .expect("Failed to load ruleset");
        Self { rules }
    }

    pub fn save(&self, filepath: &str) {
        let rules_str = serde_yaml::to_string(&self.rules).expect("Failed to serialize ruleset");
        std::fs::write(filepath, rules_str).expect("Failed to save ruleset");
    }

    // Return allowed mask for tile `p` in a given direction.
    pub fn allowed_mask(&self, p: usize, direction: Direction) -> BitVec {
        match direction {
            Direction::North => self.rules[p].north.clone(),
            Direction::South => self.rules[p].south.clone(),
            Direction::East => self.rules[p].east.clone(),
            Direction::West => self.rules[p].west.clone(),
        }
    }

    pub fn len(&self) -> usize {
        self.rules.len()
    }
}
