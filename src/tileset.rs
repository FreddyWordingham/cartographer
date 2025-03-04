use ndarray::s;
use photo::ImageRGBA;

use crate::Tile;

pub struct TileSet {
    pub tile_size: [usize; 2],
    pub tile_counts: Vec<(Tile, usize)>,
}

impl TileSet {
    pub fn new(tile_size: [usize; 2]) -> Self {
        debug_assert!(tile_size[0] > 0);
        debug_assert!(tile_size[1] > 0);

        TileSet {
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

        // Iterate over all possible tile windows in the map image
        for tile_view in map.data.windows([self.tile_size[0], self.tile_size[1], 4]) {
            let mut found = false;
            for (existing_tile, count) in &mut self.tile_counts {
                if let Some(trans) = existing_tile.equal_under_transformation(&tile_view) {
                    existing_tile.add_transformation(trans);
                    *count += 1;
                    found = true;
                    break;
                }
            }
            if !found {
                let tile_image = ImageRGBA::new(tile_view.to_owned());
                self.tile_counts.push((Tile::new(tile_image), 1));
            }
        }
    }
}

fn check_east(centre_tile: &ImageRGBA<u8>, right_tile: &ImageRGBA<u8>) -> bool {
    let width = centre_tile.width();
    let centre = centre_tile.data.slice(s![.., 1..width, ..]);
    let right = right_tile.data.slice(s![.., 0..width - 1, ..]);
    centre == right
}

fn check_west(centre_tile: &ImageRGBA<u8>, left_tile: &ImageRGBA<u8>) -> bool {
    let width = centre_tile.width();
    let centre = centre_tile.data.slice(s![.., 0..width - 1, ..]);
    let left = left_tile.data.slice(s![.., 1..width, ..]);
    centre == left
}

fn check_north(centre_tile: &ImageRGBA<u8>, top_tile: &ImageRGBA<u8>) -> bool {
    let height = centre_tile.height();
    let centre = centre_tile.data.slice(s![0..(height - 1), .., ..]);
    let top = top_tile.data.slice(s![1..height, .., ..]);
    centre == top
}

fn check_south(centre_tile: &ImageRGBA<u8>, bottom_tile: &ImageRGBA<u8>) -> bool {
    let height = centre_tile.height();
    let centre = centre_tile.data.slice(s![1..height, .., ..]);
    let bottom = bottom_tile.data.slice(s![0..(height - 1), .., ..]);
    centre == bottom
}
