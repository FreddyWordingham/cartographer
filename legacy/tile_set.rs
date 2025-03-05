use bitvec::prelude::BitVec;
use photo::Transformation;

use crate::{Pattern, PatternSet};

struct Tile {
    pub tile_index: usize,
    pub orientation: Transformation,
    pub neighbours: [BitVec; 4],
}

impl Tile {}

pub struct TileSet {
    pattern_size: [usize; 2],
    patterns: Vec<Pattern>,
    tiles: Vec<Tile>,
}

impl TileSet {
    pub fn new(pattern_set: PatternSet) -> Self {
        let num_tiles = pattern_set.len();
        let mut tiles = Vec::with_capacity(num_tiles);

        for tile in pattern_set.iter()

        tiles.shrink_to_fit();
        TileSet {
            pattern_size: pattern_set.pattern_size,
            patterns: pattern_set.patterns,
            tiles,
        }
    }
}
