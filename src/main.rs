use ndarray::{Array2, Array3, Axis, s};
use photo::ImageRGBA;
use rand::{
    Rng, SeedableRng,
    prelude::{IndexedRandom, SliceRandom},
};
use std::path::PathBuf;

const INPUT_DIR: &str = "input";
const INPUT_IMAGE_FILENAME: &str = "tileset.png";

const TILE_RESOLUTION: [usize; 2] = [3, 3];
const MAP_RESOLUTION: [usize; 2] = [20, 20];

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
    fn new(resolution: [usize; 2], num_rules: usize) -> Self {
        let p: Vec<_> = (0..num_rules).collect();
        Self {
            possibilities: Array2::from_elem(resolution, p.clone()),
        }
    }

    #[allow(dead_code)]
    fn count_map(&self) -> Array2<usize> {
        self.possibilities.mapv(|v| v.len())
    }

    #[allow(dead_code)]
    fn entropy_map(&self) -> Array2<f64> {
        self.count_map().mapv(|v| (v as f64).ln() as f64)
    }

    // Propagate constraints starting from the initial changed positions.
    fn propagate(&mut self, rules: &RuleSet, mut stack: Vec<(usize, usize)>) -> bool {
        let (rows, cols) = self.possibilities.dim();
        while let Some((ci, cj)) = stack.pop() {
            if self.possibilities[[ci, cj]].len() != 1 {
                continue;
            }
            let t = self.possibilities[[ci, cj]][0];
            let neighbors = [
                (ci.wrapping_sub(1), cj, &rules.rules[t].north), // North
                (ci, cj + 1, &rules.rules[t].east),              // East
                (ci + 1, cj, &rules.rules[t].south),             // South
                (ci, cj.wrapping_sub(1), &rules.rules[t].west),  // West
            ];
            for &(ni, nj, allowed) in &neighbors {
                if ni >= rows || nj >= cols {
                    continue;
                }
                let neighbor = &mut self.possibilities[[ni, nj]];
                let before = neighbor.len();
                neighbor.retain(|&candidate| allowed.contains(&candidate));
                if neighbor.is_empty() {
                    return false; // Contradiction
                }
                if neighbor.len() < before && neighbor.len() == 1 {
                    stack.push((ni, nj));
                }
            }
        }
        true
    }

    // Backtracking collapse method.
    fn backtracking_collapse<R: Rng>(
        &mut self,
        rules: &RuleSet,
        rng: &mut R,
    ) -> Option<Array2<usize>> {
        // Fully collapsed: return the solution.
        if self.possibilities.iter().all(|v| v.len() == 1) {
            return Some(self.possibilities.mapv(|v| v[0]));
        }

        let (rows, cols) = self.possibilities.dim();
        // Find all cells with the lowest entropy (>1 possibility).
        let mut min_entropy = usize::MAX;
        let mut min_positions = Vec::new();
        for i in 0..rows {
            for j in 0..cols {
                let len = self.possibilities[[i, j]].len();
                if len > 1 {
                    if len < min_entropy {
                        min_entropy = len;
                        min_positions.clear();
                        min_positions.push((i, j));
                    } else if len == min_entropy {
                        min_positions.push((i, j));
                    }
                }
            }
        }

        // Randomly select one of the lowest entropy cells.
        let &(i, j) = min_positions.choose(rng).unwrap();
        let mut candidates = self.possibilities[[i, j]].clone();
        // Shuffle candidates for additional stochasticity.
        candidates.shuffle(rng);

        for candidate in candidates {
            // Backup current state.
            let backup = self.possibilities.clone();
            self.possibilities[[i, j]] = vec![candidate];
            if self.propagate(rules, vec![(i, j)]) {
                if let Some(solution) = self.backtracking_collapse(rules, rng) {
                    return Some(solution);
                }
            }
            // Restore state on contradiction.
            self.possibilities = backup;
        }
        None
    }
}

fn main() {
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

    let punched_tiles = punch_tiles(tiles);
    for seed in 0..100 {
        let mut rng = rand::rngs::StdRng::seed_from_u64(seed);
        let mut wave_function = WaveFunction::new(MAP_RESOLUTION, rules.len());
        if let Some(map) = wave_function.backtracking_collapse(&rules, &mut rng) {
            let mut output = Array3::zeros([MAP_RESOLUTION[0], MAP_RESOLUTION[1], 4]);
            fill_output(&mut output, &map, &punched_tiles);
            let output_image = ImageRGBA::new(output);
            println!("{}", output_image);
            output_image
                .save(&format!("output/output_{:03}.png", seed))
                .unwrap();
        }
    }
}

fn fill_output(output: &mut Array3<u8>, map: &Array2<usize>, punched_tiles: &Vec<ImageRGBA<u8>>) {
    for ((i, j), &tile_index) in map.indexed_iter() {
        let tile_data = &punched_tiles[tile_index].data;
        let tile_slice = tile_data
            .index_axis(Axis(0), 0)
            .index_axis(Axis(0), 0)
            .to_owned();
        output.slice_mut(s![i, j, ..]).assign(&tile_slice);
    }
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
