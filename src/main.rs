use ndarray::{Array2, Array3, ArrayView1, ArrayViewMut1, s};
use photo::ImageRGBA;
use rand::Rng;
use std::path::PathBuf;

const INPUT_DIR: &str = "input";
const INPUT_IMAGE_FILENAME: &str = "tileset2.png";

const TILE_RESOLUTION: [usize; 2] = [3, 3];
const MAP_RESOLUTION: [usize; 2] = [5, 5];

struct Rule {
    north: Vec<usize>,
    east: Vec<usize>,
    south: Vec<usize>,
    west: Vec<usize>,
}

struct RuleSet {
    rules: Vec<Rule>,
}

impl RuleSet {
    fn len(&self) -> usize {
        self.rules.len()
    }
}

struct WaveFunction {
    possibilities: Array2<Vec<usize>>,
}

impl WaveFunction {
    fn new(resolution: [usize; 2], rules: &RuleSet) -> Self {
        let mut wave_function = Self::new_empty(resolution, rules.len());
        // wave_function.collapse();
        wave_function
    }

    fn new_empty(resolution: [usize; 2], num_rules: usize) -> Self {
        let p: Vec<_> = (0..num_rules).collect();
        Self {
            possibilities: Array2::from_elem(resolution, p.clone()),
        }
    }

    fn count_map(&self) -> Array2<usize> {
        self.possibilities.mapv(|v| v.len())
    }

    fn collapse_cell_probabilities(&mut self, coord: [usize; 2], rules: &RuleSet) {
        let (row, col) = (coord[0], coord[1]);
        let (height, width) = self.possibilities.dim();

        // Retain only those rule indices that have valid neighbors
        self.possibilities[[row, col]].retain(|&rule_idx| {
            let rule = &rules.rules[rule_idx];

            // Check north neighbor if exists
            if row > 0 {
                let neighbor = &self.possibilities[[row - 1, col]];
                if !rule
                    .north
                    .iter()
                    .any(|&allowed| neighbor.contains(&allowed))
                {
                    return false;
                }
            }

            // Check east neighbor if exists
            if col + 1 < width {
                let neighbor = &self.possibilities[[row, col + 1]];
                if !rule.east.iter().any(|&allowed| neighbor.contains(&allowed)) {
                    return false;
                }
            }

            // Check south neighbor if exists
            if row + 1 < height {
                let neighbor = &self.possibilities[[row + 1, col]];
                if !rule
                    .south
                    .iter()
                    .any(|&allowed| neighbor.contains(&allowed))
                {
                    return false;
                }
            }

            // Check west neighbor if exists
            if col > 0 {
                let neighbor = &self.possibilities[[row, col - 1]];
                if !rule.west.iter().any(|&allowed| neighbor.contains(&allowed)) {
                    return false;
                }
            }
            true
        });

        fn collapse_cell(&mut self, coord: [usize; 2], rules: &RuleSet) {
            let (row, col) = (coord[0], coord[1]);
            // If not yet collapsed, pick one possibility (could use randomness)
            if self.possibilities[[row, col]].len() > 1 {
                let chosen = self.possibilities[[row, col]][0];
                self.possibilities[[row, col]] = vec![chosen];
            }

            let (height, width) = self.possibilities.dim();
            let mut stack = vec![coord];

            // Propagate changes recursively using a stack
            while let Some([r, c]) = stack.pop() {
                // For each neighboring cell, update its possibilities
                let neighbors = [
                    if r > 0 { Some([r - 1, c]) } else { None },
                    if r + 1 < height {
                        Some([r + 1, c])
                    } else {
                        None
                    },
                    if c > 0 { Some([r, c - 1]) } else { None },
                    if c + 1 < width {
                        Some([r, c + 1])
                    } else {
                        None
                    },
                ];

                for neighbor in neighbors.iter().flatten() {
                    let (nr, nc) = (neighbor[0], neighbor[1]);
                    let prev_count = self.possibilities[[nr, nc]].len();
                    self.collapse_cell_probabilities(*neighbor, rules);
                    let new_count = self.possibilities[[nr, nc]].len();
                    if new_count != prev_count {
                        stack.push(*neighbor);
                    }
                }
            }
        }
    }
}

fn main() {
    println!("Hello, world!");

    let input_image_filepath = PathBuf::from(INPUT_DIR).join(INPUT_IMAGE_FILENAME);
    let input_image =
        ImageRGBA::<u8>::load(input_image_filepath).expect("Failed to load input image");
    println!("{}", input_image);

    println!("---\nTiles:");
    let tiles = remove_duplicates(collect_tiles(&input_image, TILE_RESOLUTION));
    for tile in &tiles {
        println!("{}", tile);
    }

    let rules = create_rules(&tiles);
    for (n, rule) in rules.rules.iter().enumerate() {
        println!("Rule {}:", n);
        println!("  North: {:?}", rule.north);
        println!("  East: {:?}", rule.east);
        println!("  South: {:?}", rule.south);
        println!("  West: {:?}", rule.west);
    }

    let wave_function = WaveFunction::new(MAP_RESOLUTION, &rules);
    // let mut rng = rand::rng();
    let map = wave_function.count_map();
    println!("{:?}", map);
}

/// Slide window over input image to collect all possible tiles.
fn collect_tiles(image: &ImageRGBA<u8>, tile_resolution: [usize; 2]) -> Vec<ImageRGBA<u8>> {
    let mut tiles = Vec::with_capacity(
        (image.height() - tile_resolution[0] + 1) * (image.width() - tile_resolution[1] + 1),
    );

    for tile in image
        .data
        .windows([tile_resolution[0], tile_resolution[1], 4])
    {
        let tile = ImageRGBA::new(tile.to_owned());
        tiles.push(tile);
    }

    tiles
}

/// Remove duplicate tiles from the list.
fn remove_duplicates(tiles: Vec<ImageRGBA<u8>>) -> Vec<ImageRGBA<u8>> {
    let mut unique_tiles = Vec::with_capacity(tiles.len());
    let mut unique_tiles_set = std::collections::HashSet::new();

    for tile in tiles {
        if unique_tiles_set.insert(tile.data.clone()) {
            unique_tiles.push(tile);
        }
    }

    unique_tiles
}

/// Punch tiles to remove the edges of the tile.
#[allow(dead_code)]
fn punch_tiles(tiles: Vec<ImageRGBA<u8>>) -> Vec<ImageRGBA<u8>> {
    let mut punched_tiles = Vec::with_capacity(tiles.len());

    for tile in tiles {
        punched_tiles.push(ImageRGBA::new(extract_center(&tile.data, 1)));
    }

    punched_tiles
}

fn extract_center(image: &Array3<u8>, border: usize) -> Array3<u8> {
    let (height, width, _) = image.dim();
    image
        .slice(s![border..height - border, border..width - border, ..])
        .to_owned()
}

fn create_rules(tiles: &[ImageRGBA<u8>]) -> RuleSet {
    let mut rules = Vec::with_capacity(tiles.len());

    for q_tile in tiles {
        let mut rule = Rule {
            north: vec![],
            east: vec![],
            south: vec![],
            west: vec![],
        };
        for (n, tile) in tiles.iter().enumerate() {
            if check_east(q_tile, tile) {
                rule.east.push(n);
            }
            if check_west(q_tile, tile) {
                rule.west.push(n);
            }
            if check_north(q_tile, tile) {
                rule.north.push(n);
            }
            if check_south(q_tile, tile) {
                rule.south.push(n);
            }
        }
        rules.push(rule);
    }

    RuleSet { rules }
}

fn check_east(centre_tile: &ImageRGBA<u8>, right_tile: &ImageRGBA<u8>) -> bool {
    debug_assert!(centre_tile.data.dim() == right_tile.data.dim());
    let width = centre_tile.width();
    let centre = centre_tile.data.slice(s![.., 1..width, ..]);
    let right = right_tile.data.slice(s![.., 0..width - 1, ..]);
    centre == right
}

fn check_west(centre_tile: &ImageRGBA<u8>, left_tile: &ImageRGBA<u8>) -> bool {
    debug_assert!(centre_tile.data.dim() == left_tile.data.dim());
    let width = centre_tile.width();
    let centre = centre_tile.data.slice(s![.., 0..width - 1, ..]);
    let left = left_tile.data.slice(s![.., 1..width, ..]);
    centre == left
}

fn check_north(centre_tile: &ImageRGBA<u8>, top_tile: &ImageRGBA<u8>) -> bool {
    debug_assert!(centre_tile.data.dim() == top_tile.data.dim());
    let height = centre_tile.height();
    let centre = centre_tile.data.slice(s![0..(height - 1), .., ..]);
    let top = top_tile.data.slice(s![1..height, .., ..]);
    centre == top
}

fn check_south(centre_tile: &ImageRGBA<u8>, bottom_tile: &ImageRGBA<u8>) -> bool {
    debug_assert!(centre_tile.data.dim() == bottom_tile.data.dim());
    let height = centre_tile.height();
    let centre = centre_tile.data.slice(s![1..height, .., ..]);
    let bottom = bottom_tile.data.slice(s![0..(height - 1), .., ..]);
    centre == bottom
}
