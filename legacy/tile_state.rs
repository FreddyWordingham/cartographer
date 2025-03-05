use photo::{ALL_TRANSFORMATIONS, ImageRGBA};

use crate::{ALL_DIRECTIONS, PatternSet};

pub struct TileState {
    pub index: u32,
    pub orientation: u8,
    pub pattern_set: &PatternSet,
}

impl TileState {
    pub fn new(index: u32, orientation: u8, pattern_set: &PatternSet) -> Self {
        debug_assert!(orientation < ALL_DIRECTIONS.len() as u8);
        TileState {
            index,
            orientation,
            pattern_set,
        }
    }

    pub fn image(&self) -> ImageRGBA<u8> {
        let pattern = self.pattern_set.patterns[self.index as usize];
        if pattern
            .transformations
            .contains(&ALL_TRANSFORMATIONS[self.orientation as usize])
        {
            pattern
                .image
                .transform(ALL_TRANSFORMATIONS[self.orientation as usize])
        } else {
            panic!("Pattern does not contain the required transformation.")
        }
    }
}
