use crossterm::event::{Event, KeyCode, KeyEvent, read};
use indicatif::ProgressBar;
use ndarray::{Array2, Array3, Axis, s};
use photo::ImageRGBA;
use rand::{SeedableRng, prelude::SliceRandom};
use serde::{Deserialize, Serialize};
use std::{
    cmp::Reverse,
    collections::BinaryHeap,
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
    thread,
};

const TILE_RESOLUTION: [usize; 2] = [3, 3];

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Rule {
    north: Vec<usize>,
    east: Vec<usize>,
    south: Vec<usize>,
    west: Vec<usize>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
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
        self.count_map().mapv(|v| (v as f64).ln())
    }

    fn propagate(&mut self, rules: &RuleSet, mut stack: Vec<(usize, usize)>) -> bool {
        let (rows, cols) = self.possibilities.dim();
        while let Some((ci, cj)) = stack.pop() {
            if self.possibilities[[ci, cj]].len() != 1 {
                continue;
            }
            let t = self.possibilities[[ci, cj]][0];
            let neighbors = [
                (ci.wrapping_sub(1), cj, &rules.rules[t].north),
                (ci, cj + 1, &rules.rules[t].east),
                (ci + 1, cj, &rules.rules[t].south),
                (ci, cj.wrapping_sub(1), &rules.rules[t].west),
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

    fn count_collapsed(&self) -> u64 {
        self.possibilities.iter().filter(|v| v.len() == 1).count() as u64
    }

    /// Modified collapse function using the priority queue.
    fn backtracking_collapse<R: rand::Rng>(
        &mut self,
        rules: &RuleSet,
        rng: &mut R,
        progress: &indicatif::ProgressBar,
        cancel_flag: &std::sync::atomic::AtomicBool,
    ) -> Option<ndarray::Array2<usize>> {
        if cancel_flag.load(std::sync::atomic::Ordering::SeqCst) {
            return None;
        }
        progress.set_position(self.count_collapsed());

        if self.possibilities.iter().all(|v| v.len() == 1) {
            return Some(self.possibilities.mapv(|v| v[0]));
        }

        if let Some((i, j)) = self.choose_min_entropy_cell() {
            let mut candidates = self.possibilities[[i, j]].clone();
            candidates.shuffle(rng);
            for candidate in candidates {
                let backup = self.possibilities.clone();
                self.possibilities[[i, j]] = vec![candidate];
                progress.set_position(self.count_collapsed());
                if self.propagate(rules, vec![(i, j)]) {
                    if let Some(solution) =
                        self.backtracking_collapse(rules, rng, progress, cancel_flag)
                    {
                        return Some(solution);
                    }
                }
                self.possibilities = backup;
                progress.set_position(self.count_collapsed());
                if cancel_flag.load(std::sync::atomic::Ordering::SeqCst) {
                    return None;
                }
            }
        }
        None
    }

    /// Returns the coordinates of the cell with minimum entropy (>1 possibility).
    fn choose_min_entropy_cell(&self) -> Option<(usize, usize)> {
        let (rows, cols) = self.possibilities.dim();
        let mut heap = BinaryHeap::new();
        for i in 0..rows {
            for j in 0..cols {
                let count = self.possibilities[[i, j]].len();
                if count > 1 {
                    heap.push(Reverse((count, i, j)));
                }
            }
        }
        heap.pop().map(|Reverse((_count, i, j))| (i, j))
    }
}

fn main() {
    // Read command line arguments.
    let args: Vec<String> = std::env::args().collect();
    println!("{:?}", args.len());
    if args.len() != 3 {
        eprintln!("Usage: {} <input_image> <output_resolution>", args[0]);
        std::process::exit(1);
    }
    let input_image_filepath = &args[1];
    let map_resolution = {
        /// In the form "widthxheight".
        let s = &args[2];
        let mut parts = s.split('x');
        let width = parts.next().unwrap().parse::<usize>().unwrap();
        let height = parts.next().unwrap().parse::<usize>().unwrap();
        [height, width]
    };

    // Spawn a thread to listen for the spacebar.
    let cancel_flag = Arc::new(AtomicBool::new(false));
    {
        let cancel_flag = cancel_flag.clone();
        thread::spawn(move || {
            loop {
                if let Event::Key(KeyEvent {
                    code: KeyCode::Char(' '),
                    ..
                }) = read().unwrap()
                {
                    cancel_flag.store(true, Ordering::SeqCst);
                }
            }
        });
    }

    let input_image =
        ImageRGBA::<u8>::load(input_image_filepath).expect("Failed to load input image");
    println!("{}", input_image);

    println!("---\nTiles:");
    let tiles = remove_duplicates(collect_tiles(&input_image, TILE_RESOLUTION));
    for tile in &tiles {
        println!("{}", tile);
    }

    let rules = create_rules(&tiles);
    // Write rules to a file.
    let rules_json = serde_yaml::to_string(&rules).unwrap();
    std::fs::write("output/rules.yaml", rules_json).unwrap();

    let punched_tiles = punch_tiles(tiles);
    for seed in 0..100 {
        println!("---\n\n\nSeed: {}", seed);
        // Reset cancellation for each seed.
        cancel_flag.store(false, Ordering::SeqCst);
        let mut rng = rand::rngs::StdRng::seed_from_u64(seed);
        let mut wave_function = WaveFunction::new(map_resolution, rules.len());
        let progress = ProgressBar::new((map_resolution[0] * map_resolution[1]) as u64);
        if let Some(map) =
            wave_function.backtracking_collapse(&rules, &mut rng, &progress, &cancel_flag)
        {
            let mut output = Array3::zeros([map_resolution[0], map_resolution[1], 4]);
            fill_output(&mut output, &map, &punched_tiles);
            let output_image = ImageRGBA::new(output);
            progress.finish();
            println!("{}", output_image);
            output_image
                .save(&format!("output/output_{:03}.png", seed))
                .unwrap();
        } else {
            progress.finish();
            println!("Seed {} cancelled.", seed);
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

fn collect_tiles(image: &ImageRGBA<u8>, tile_resolution: [usize; 2]) -> Vec<ImageRGBA<u8>> {
    let mut tiles = Vec::with_capacity(
        (image.height() - tile_resolution[0] + 1) * (image.width() - tile_resolution[1] + 1),
    );
    for tile in image
        .data
        .windows([tile_resolution[0], tile_resolution[1], 4])
    {
        tiles.push(ImageRGBA::new(tile.to_owned()));
    }
    tiles
}

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

#[allow(dead_code)]
fn punch_tiles(tiles: Vec<ImageRGBA<u8>>) -> Vec<ImageRGBA<u8>> {
    tiles
        .into_iter()
        .map(|tile| ImageRGBA::new(extract_center(&tile.data, 1)))
        .collect()
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
