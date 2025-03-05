use crate::{Pattern, Tile};

pub struct TileSet {
    pub pattern_size: [usize; 2],
    pub patterns: Vec<Pattern>,
    pub tiles: Vec<Tile>,
}
