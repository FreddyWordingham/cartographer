use bitvec::prelude::BitVec;
use ndarray::s;
use photo::ImageRGBA;

use crate::{Pattern, Tile, TileSet};

pub struct PatternSet {
    pub pattern_size: [usize; 2],
    pub patterns: Vec<Pattern>,
}

impl PatternSet {
    pub fn new(pattern_size: [usize; 2]) -> Self {
        debug_assert!(pattern_size[0] > 0);
        debug_assert!(pattern_size[1] > 0);

        PatternSet {
            pattern_size,
            patterns: Vec::new(),
        }
    }

    pub fn ingest(mut self, map: &ImageRGBA<u8>) -> Self {
        let height = map.height();
        let width = map.width();

        debug_assert!(
            height >= self.pattern_size[0],
            "Map height is too small for pattern size."
        );
        debug_assert!(
            width >= self.pattern_size[1],
            "Map width is too small for pattern size."
        );

        // Iterate over all possible pattern windows in the map image
        for image in map
            .data
            .windows([self.pattern_size[0], self.pattern_size[1], 4])
        {
            let mut found = false;
            for pattern in &mut self.patterns {
                if let Some(trans) = pattern.equal_under_transformation(&image) {
                    pattern.add_transformation(trans);
                    pattern.frequency += 1;
                    found = true;
                    break;
                }
            }
            if !found {
                let pattern_image = ImageRGBA::new(image.to_owned());
                self.patterns.push(Pattern::new(pattern_image, 1));
            }
        }

        self
    }

    pub fn num_tiles(&self) -> usize {
        self.patterns.iter().map(|p| p.transformations.len()).sum()
    }

    pub fn tile_images(&self) -> Vec<ImageRGBA<u8>> {
        let mut images = Vec::with_capacity(self.num_tiles());

        for pattern in &self.patterns {
            for transformation in pattern.transformations.clone() {
                images.push(pattern.image.transform(transformation));
            }
        }

        images
    }

    pub fn build(self) -> TileSet {
        // Create a vector of tuples: (pattern index, transformation, tile image)
        let mut tile_infos = Vec::new();
        for (pattern_index, pattern) in self.patterns.iter().enumerate() {
            for &transformation in &pattern.transformations {
                let image = pattern.image.transform(transformation);
                tile_infos.push((pattern_index, transformation, image));
            }
        }

        let num_tiles = tile_infos.len();
        let mut tiles = Vec::with_capacity(num_tiles);

        // For each tile, compute neighbour compatibility in all four directions.
        for i in 0..num_tiles {
            let (_, transformation, ref image_i) = tile_infos[i];
            let mut neighbour_north = BitVec::repeat(false, num_tiles);
            let mut neighbour_east = BitVec::repeat(false, num_tiles);
            let mut neighbour_south = BitVec::repeat(false, num_tiles);
            let mut neighbour_west = BitVec::repeat(false, num_tiles);

            for j in 0..num_tiles {
                let (_, _, ref image_j) = tile_infos[j];
                if check_north(image_i, image_j) {
                    neighbour_north.set(j, true);
                }
                if check_east(image_i, image_j) {
                    neighbour_east.set(j, true);
                }
                if check_south(image_i, image_j) {
                    neighbour_south.set(j, true);
                }
                if check_west(image_i, image_j) {
                    neighbour_west.set(j, true);
                }
            }
            let neighbours = [
                neighbour_north,
                neighbour_east,
                neighbour_south,
                neighbour_west,
            ];
            let (pattern_index, _, _) = tile_infos[i];
            tiles.push(Tile {
                pattern_index,
                transformation,
                neighbours,
            });
        }

        TileSet {
            pattern_size: self.pattern_size,
            patterns: self.patterns,
            tiles,
        }
    }
}

fn check_east(centre_image: &ImageRGBA<u8>, right_image: &ImageRGBA<u8>) -> bool {
    let width = centre_image.width();
    let centre = centre_image.data.slice(s![.., 1..width, ..]);
    let right = right_image.data.slice(s![.., 0..width - 1, ..]);
    centre == right
}

fn check_west(centre_image: &ImageRGBA<u8>, left_image: &ImageRGBA<u8>) -> bool {
    let width = centre_image.width();
    let centre = centre_image.data.slice(s![.., 0..width - 1, ..]);
    let left = left_image.data.slice(s![.., 1..width, ..]);
    centre == left
}

fn check_north(centre_image: &ImageRGBA<u8>, top_image: &ImageRGBA<u8>) -> bool {
    let height = centre_image.height();
    let centre = centre_image.data.slice(s![0..(height - 1), .., ..]);
    let top = top_image.data.slice(s![1..height, .., ..]);
    centre == top
}

fn check_south(centre_image: &ImageRGBA<u8>, bottom_image: &ImageRGBA<u8>) -> bool {
    let height = centre_image.height();
    let centre = centre_image.data.slice(s![1..height, .., ..]);
    let bottom = bottom_image.data.slice(s![0..(height - 1), .., ..]);
    centre == bottom
}
