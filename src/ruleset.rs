use bitvec::prelude::BitVec;
use photo::Transformation;

use crate::TileSet;

struct Rule {
    pub tile_index: usize,
    pub orientation: Transformation,
    pub neighbours: [BitVec; 4],
}

pub struct RuleSet {
    tileset: TileSet,
    rules: Vec<Rule>,
}

impl RuleSet {
    pub fn new(tileset: TileSet) -> Self {
        let mut rules = Vec::new();
        // Create a rule for each tile in the tileset

        RuleSet { tileset, rules }
    }
}
