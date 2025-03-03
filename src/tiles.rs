use photo::{ImageRGBA, Transformation};
use std::collections::HashSet;

struct Tile {
    image: ImageRGBA<u8>,
    transformations: HashSet<Transformation>,
}

impl Tile {
    pub fn new(image: ImageRGBA<u8>) -> Self {
        let mut transformations = HashSet::new();
        transformations.insert(Transformation::Identity);

        Tile {
            image,
            transformations,
        }
    }

    pub fn add_transformation(&mut self, transformation: Transformation) {
        self.transformations.push(transformation);
    }
}

pub struct Tiles {
    tile_size: [usize; 2],
    tile_counts: Vec<(Tile, usize)>,
}

impl Tiles {
    pub fn new(tile_size: [usize; 2]) -> Self {
        debug_assert!(tile_size[0] > 0);
        debug_assert!(tile_size[1] > 0);

        Tiles {
            tile_size,
            tile_counts: Vec::new(),
        }
    }

    pub fn ingest(&mut self, map: &ImageRGBA<u8>) {
        let height = map.height();
        let width = map.width();

        debug_assert!(
            height >= self.tile_size[0],
            "Map height is too small for tile size."
        );
        debug_assert!(
            width >= self.tile_size[1],
            "Map width is too small for tile size."
        );

        for tile in map.data.windows([self.tile_size[0], self.tile_size[1], 4]) {
            let mut found = false;
            for (existing_tile, count) in &mut self.tile_counts {
                if existing_tile.image.data == tile {
                    *count += 1;
                    found = true;
                    break;
                }
            }
            if !found {
                self.tile_counts
                    .push((Tile::new(ImageRGBA::new(tile.to_owned())), 1));
            }
        }
    }
}
