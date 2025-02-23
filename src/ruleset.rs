use indexmap::IndexSet;
use ndarray::Array2;
use serde::{Deserialize, Serialize};
use std::fs::File;

#[derive(Debug, Serialize, Deserialize)]
pub struct Ruleset {
    pub rules: Vec<Rule>,
}

impl Ruleset {
    /// Construct a new Ruleset from an example index map.
    pub fn new(map: &Array2<usize>) -> Self {
        let max_index = map.iter().max().unwrap();
        let mut rules: Vec<Rule> = vec![Rule::default(); max_index + 1];

        let (height, width) = map.dim();

        // Iterate over map tiles and find valid indices for each side.
        for (coord, &tile) in map.indexed_iter() {
            if coord.0 > 0 {
                rules[tile].north.insert(map[[coord.0 - 1, coord.1]]);
            }
            if coord.0 < height - 1 {
                rules[tile].south.insert(map[[coord.0 + 1, coord.1]]);
            }
            if coord.1 > 0 {
                rules[tile].west.insert(map[[coord.0, coord.1 - 1]]);
            }
            if coord.1 < width - 1 {
                rules[tile].east.insert(map[[coord.0, coord.1 + 1]]);
            }
        }

        Self { rules }
    }
}

impl Ruleset {
    pub fn load(filepath: &str) -> Self {
        let file = File::create(filepath).expect("Failed to create file");
        serde_yaml::from_reader(file).expect("Failed to read rules")
    }

    pub fn save(&self, filepath: &str) {
        let file = File::create(filepath).expect("Failed to create file");
        serde_yaml::to_writer(file, self).expect("Failed to write rules");
    }
}

/// The allowed indices for each side of a tile.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Rule {
    pub north: IndexSet<usize>,
    pub south: IndexSet<usize>,
    pub west: IndexSet<usize>,
    pub east: IndexSet<usize>,
}
