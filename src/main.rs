use ndarray::{s, Array3};
use photo::ImageRGBA;
use rand::{
    distr::{weighted::WeightedIndex, Distribution},
    rng,
};
use std::collections::HashMap;

fn main() {
    let input_file = "input/cube.png";
    let input = ImageRGBA::<u8>::load(input_file).unwrap();
    println!("{}", input);

    let mut tileset = TileSet::new(input);
    tileset.compute_rules();

    // Generate an image with a grid of 10x10 tiles.
    let output = tileset.generate(10, 10);
    println!("{}", output);
}

#[derive(Copy, Clone, Debug)]
pub enum Direction {
    Right,
    Bottom,
    Left,
    Top,
}

impl Direction {
    pub fn index(self) -> usize {
        match self {
            Direction::Right => 0,
            Direction::Bottom => 1,
            Direction::Left => 2,
            Direction::Top => 3,
        }
    }
    pub fn opposite(self) -> Direction {
        match self {
            Direction::Right => Direction::Left,
            Direction::Bottom => Direction::Top,
            Direction::Left => Direction::Right,
            Direction::Top => Direction::Bottom,
        }
    }
}

pub struct Tile {
    image: ImageRGBA<u8>,
    frequency: u8,
    // Rules are indexed by Direction::index()
    rules: [Vec<usize>; 4],
}

impl Tile {
    pub fn new(image: ImageRGBA<u8>, frequency: u8, rules: [Vec<usize>; 4]) -> Self {
        Self {
            image,
            frequency,
            rules,
        }
    }
}

pub struct TileSet {
    pub tiles: Vec<Tile>,
}

impl TileSet {
    pub fn new(image: ImageRGBA<u8>) -> Self {
        let all_tiles = Self::get_tiles(&image, 3);
        let tiles = Self::reduce_tiles(all_tiles);
        let mut tileset = Vec::new();
        for (tile, count) in tiles {
            tileset.push(Tile::new(tile, count, [vec![], vec![], vec![], vec![]]));
        }
        Self { tiles: tileset }
    }

    fn get_tiles(image: &ImageRGBA<u8>, tile_size: usize) -> Vec<ImageRGBA<u8>> {
        let (_, _, channels) = image.data.dim();
        let window_shape = [tile_size, tile_size, channels];
        let mut tiles = Vec::new();

        for window in image.data.windows(window_shape).into_iter() {
            tiles.push(ImageRGBA::new(window.to_owned()));
        }
        tiles
    }

    fn reduce_tiles(tiles: Vec<ImageRGBA<u8>>) -> Vec<(ImageRGBA<u8>, u8)> {
        let mut counts: HashMap<Vec<u8>, (ImageRGBA<u8>, u8)> = HashMap::new();

        for tile in tiles {
            // Flatten the tile data to create a hashable key.
            let key: Vec<u8> = tile.data.iter().copied().collect();
            counts
                .entry(key)
                .and_modify(|(_, count)| *count += 1)
                .or_insert((tile, 1));
        }

        counts.into_iter().map(|(_, v)| v).collect()
    }

    pub fn compute_rules(&mut self) {
        // Precompute each tile's edges.
        let edges: Vec<[Vec<u8>; 4]> = self
            .tiles
            .iter()
            .map(|tile| get_edges(&tile.image))
            .collect();

        for i in 0..self.tiles.len() {
            for j in 0..self.tiles.len() {
                for &dir in &[
                    Direction::Right,
                    Direction::Bottom,
                    Direction::Left,
                    Direction::Top,
                ] {
                    let opp = dir.opposite();
                    if edges[i][dir.index()] == edges[j][opp.index()] {
                        self.tiles[i].rules[dir.index()].push(j);
                    }
                }
            }
        }
    }

    /// Generate a new image using the wave function collapse algorithm.
    /// `grid_width` and `grid_height` specify the number of tiles in each dimension.
    pub fn generate(&self, grid_width: usize, grid_height: usize) -> ImageRGBA<u8> {
        let num_tiles = self.tiles.len();
        // Initialize grid: each cell starts with all tile possibilities.
        let mut grid = vec![vec![(0..num_tiles).collect::<Vec<usize>>(); grid_width]; grid_height];

        // Propagation function: update neighbors based on constraints.
        fn propagate(
            grid: &mut Vec<Vec<Vec<usize>>>,
            tiles: &Vec<Tile>,
            grid_width: usize,
            grid_height: usize,
            start_x: usize,
            start_y: usize,
        ) -> bool {
            let mut queue = vec![(start_x, start_y)];
            while let Some((cx, cy)) = queue.pop() {
                // For each neighbor, enforce constraints.
                for (dx, dy, dir) in &[
                    (1, 0, Direction::Right),
                    (-1, 0, Direction::Left),
                    (0, 1, Direction::Bottom),
                    (0, -1, Direction::Top),
                ] {
                    let nx = cx as isize + dx;
                    let ny = cy as isize + dy;
                    if nx < 0 || ny < 0 || nx >= grid_width as isize || ny >= grid_height as isize {
                        continue;
                    }
                    let nx = nx as usize;
                    let ny = ny as usize;
                    let before = grid[ny][nx].len();
                    // Filter neighbor's possibilities.
                    grid[ny][nx] = grid[ny][nx]
                        .iter()
                        .cloned()
                        .filter(|&poss| {
                            // For each possibility in neighbor cell, ensure there's a compatible possibility
                            // in the current cell.
                            grid[cy][cx]
                                .iter()
                                .any(|&p| tiles[p].rules[dir.index()].contains(&poss))
                        })
                        .collect();
                    if grid[ny][nx].is_empty() {
                        // Contradiction!
                        return false;
                    }
                    if grid[ny][nx].len() < before {
                        queue.push((nx, ny));
                    }
                }
            }
            true
        }

        let mut rng = rng();

        // Main collapse loop.
        loop {
            // Find the cell with the smallest entropy (more than one possibility).
            let mut min_entropy = usize::MAX;
            let mut cell = None;
            for y in 0..grid_height {
                for x in 0..grid_width {
                    let count = grid[y][x].len();
                    if count > 1 && count < min_entropy {
                        min_entropy = count;
                        cell = Some((x, y));
                    }
                }
            }
            if cell.is_none() {
                break; // All cells are collapsed.
            }
            let (x, y) = cell.unwrap();
            let possibilities = grid[y][x].clone();
            // Weighted random selection based on tile frequency.
            let weights: Vec<u32> = possibilities
                .iter()
                .map(|&i| self.tiles[i].frequency as u32)
                .collect();
            let dist = WeightedIndex::new(&weights).unwrap();
            let chosen = possibilities[dist.sample(&mut rng)];
            grid[y][x] = vec![chosen];

            // Propagate constraints.
            if !propagate(&mut grid, &self.tiles, grid_width, grid_height, x, y) {
                // On contradiction, reinitialize and restart.
                grid = vec![vec![(0..num_tiles).collect::<Vec<usize>>(); grid_width]; grid_height];
            }
        }

        // Assemble the final image.
        let (tile_h, tile_w, channels) = self.tiles[0].image.data.dim();
        let out_h = grid_height * tile_h;
        let out_w = grid_width * tile_w;
        let mut output_data = Array3::<u8>::zeros((out_h, out_w, channels));

        for (gy, row) in grid.iter().enumerate() {
            for (gx, cell) in row.iter().enumerate() {
                let tile_index = cell[0]; // Collapsed cell.
                let tile_img = &self.tiles[tile_index].image.data;
                let start_y = gy * tile_h;
                let start_x = gx * tile_w;
                let mut slice = output_data.slice_mut(s![
                    start_y..start_y + tile_h,
                    start_x..start_x + tile_w,
                    ..
                ]);
                slice.assign(tile_img);
            }
        }
        ImageRGBA::new(output_data)
    }
}

// Helper to extract the four edges of a tile.
// Order: [Right, Bottom, Left, Top]
// We swap bottom and top to account for the vertical flip.
fn get_edges(tile: &ImageRGBA<u8>) -> [Vec<u8>; 4] {
    let data = &tile.data;
    let (h, w, _channels) = data.dim();

    let right: Vec<u8> = (0..h)
        .flat_map(|i| {
            data.slice(s![i, w - 1, ..])
                .iter()
                .copied()
                .collect::<Vec<u8>>()
        })
        .collect();

    // Bottom edge comes from row 0 (due to vertical flip).
    let bottom: Vec<u8> = (0..w)
        .flat_map(|j| {
            data.slice(s![0, j, ..])
                .iter()
                .copied()
                .collect::<Vec<u8>>()
        })
        .collect();

    let left: Vec<u8> = (0..h)
        .flat_map(|i| {
            data.slice(s![i, 0, ..])
                .iter()
                .copied()
                .collect::<Vec<u8>>()
        })
        .collect();

    // Top edge comes from row h-1.
    let top: Vec<u8> = (0..w)
        .flat_map(|j| {
            data.slice(s![h - 1, j, ..])
                .iter()
                .copied()
                .collect::<Vec<u8>>()
        })
        .collect();

    [right, bottom, left, top]
}
