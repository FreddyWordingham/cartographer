use bitvec::prelude::BitVec;
use photo::Transformation;

pub struct Tile {
    pub pattern_index: usize,
    pub transformation: Transformation,
    pub neighbours: [BitVec; 4],
}

impl Tile {}
