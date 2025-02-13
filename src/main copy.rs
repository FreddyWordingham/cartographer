use ndarray::{s, Array2};
use ndarray_images::Image;
use rand::{rng, Rng};
use std::collections::{HashMap, HashSet, VecDeque};

fn main() {
    println!("Hello, world!");

    let example_filepath = "input/rooms.png";
    let example: Array2<f32> = Image::load(example_filepath).unwrap();
    let num_tiles = [4, 4];
    let tile_size = [
        example.shape()[0] / num_tiles[0],
        example.shape()[1] / num_tiles[1],
    ];

    let target_num_tiles = [num_tiles[0] * 8, num_tiles[1] * 8];
    let output_data = wave_function_collapse(&example, tile_size, target_num_tiles);

    // Save or display output_data as needed.
    let output_filepath = "output/rooms_wfc.png";
    output_data.save(&output_filepath).unwrap();
}

fn wave_function_collapse(
    example: &Array2<f32>,
    tile_size: [usize; 2],
    target_num_tiles: [usize; 2],
) -> Array2<f32> {
    // --- STEP 1: Extract unique tiles and their frequencies ---
    let mut tiles = Vec::new();
    let mut frequencies = Vec::new();
    let mut tile_map: HashMap<String, usize> = HashMap::new();
    let tiles_per_row = 4; // from num_tiles above
    let tiles_per_col = 4;
    for ty in 0..tiles_per_col {
        for tx in 0..tiles_per_row {
            let tile = example
                .slice(s![
                    ty * tile_size[0]..(ty + 1) * tile_size[0],
                    tx * tile_size[1]..(tx + 1) * tile_size[1]
                ])
                .to_owned();
            let key = format!("{:?}", tile);
            if let Some(&idx) = tile_map.get(&key) {
                frequencies[idx] += 1;
            } else {
                let idx = tiles.len();
                tiles.push(tile);
                frequencies.push(1);
                tile_map.insert(key, idx);
            }
        }
    }

    // --- STEP 2: Build compatibility (edge matching) ---
    // Directions: 0=Up, 1=Right, 2=Down, 3=Left.
    let opposites = [2, 3, 0, 1];
    let num_unique = tiles.len();
    let mut compat: Vec<[Vec<usize>; 4]> =
        vec![[Vec::new(), Vec::new(), Vec::new(), Vec::new()]; num_unique];
    for i in 0..num_unique {
        for j in 0..num_unique {
            for d in 0..4 {
                if edges_match(&get_edge(&tiles[i], d), &get_edge(&tiles[j], opposites[d])) {
                    compat[i][d].push(j);
                }
            }
        }
    }

    // --- STEP 3: Initialize the output grid with all possibilities ---
    let (grid_h, grid_w) = (target_num_tiles[0], target_num_tiles[1]);
    let mut grid: Vec<Vec<HashSet<usize>>> = vec![vec![(0..num_unique).collect(); grid_w]; grid_h];

    // --- STEP 4: Collapse & propagate ---
    let mut rng = rng();
    while let Some((y, x)) = find_lowest_entropy(&grid) {
        // Weighted random pick from candidates.
        let candidates: Vec<usize> = grid[y][x].iter().cloned().collect();
        let total_weight: usize = candidates.iter().map(|&i| frequencies[i]).sum();
        let mut r = rng.random_range(0..total_weight);
        let mut chosen = candidates[0];
        for &c in &candidates {
            r = r.saturating_sub(frequencies[c]);
            if r == 0 {
                chosen = c;
                break;
            }
        }
        grid[y][x].clear();
        grid[y][x].insert(chosen);
        propagate(y, x, &mut grid, &compat);
    }

    // --- STEP 5: Build the output image by placing each tile ---
    let out_h = grid_h * tile_size[0];
    let out_w = grid_w * tile_size[1];
    let mut output = Array2::<f32>::zeros((out_h, out_w));
    for gy in 0..grid_h {
        for gx in 0..grid_w {
            // Safe unwrap: cell must be collapsed.
            let tile_index = *grid[gy][gx].iter().next().unwrap();
            let tile = &tiles[tile_index];
            let y0 = gy * tile_size[0];
            let x0 = gx * tile_size[1];
            output
                .slice_mut(s![y0..y0 + tile_size[0], x0..x0 + tile_size[1]])
                .assign(tile);
        }
    }
    output
}

fn get_edge(tile: &Array2<f32>, direction: usize) -> Vec<f32> {
    let (h, w) = (tile.shape()[0], tile.shape()[1]);
    match direction {
        0 => tile.slice(s![0, ..]).to_vec(),     // Up
        1 => tile.slice(s![.., w - 1]).to_vec(), // Right
        2 => tile.slice(s![h - 1, ..]).to_vec(), // Down
        3 => tile.slice(s![.., 0]).to_vec(),     // Left
        _ => panic!("Invalid direction"),
    }
}

fn edges_match(edge1: &Vec<f32>, edge2: &Vec<f32>) -> bool {
    if edge1.len() != edge2.len() {
        return false;
    }
    for (a, b) in edge1.iter().zip(edge2.iter()) {
        if (a - b).abs() > 1e-6 {
            return false;
        }
    }
    true
}

fn find_lowest_entropy(grid: &Vec<Vec<HashSet<usize>>>) -> Option<(usize, usize)> {
    let mut min = usize::MAX;
    let mut coord = None;
    for (y, row) in grid.iter().enumerate() {
        for (x, cell) in row.iter().enumerate() {
            let n = cell.len();
            if n > 1 && n < min {
                min = n;
                coord = Some((y, x));
            }
        }
    }
    coord
}

fn propagate(
    y: usize,
    x: usize,
    grid: &mut Vec<Vec<HashSet<usize>>>,
    compat: &Vec<[Vec<usize>; 4]>,
) {
    let grid_h = grid.len();
    let grid_w = grid[0].len();
    let mut queue = VecDeque::new();
    queue.push_back((y, x));

    while let Some((cy, cx)) = queue.pop_front() {
        let cell_candidates = grid[cy][cx].clone();
        for (d, (dy, dx)) in [(-1isize, 0isize), (0, 1), (1, 0), (0, -1)]
            .iter()
            .enumerate()
        {
            let ny = cy as isize + dy;
            let nx = cx as isize + dx;
            if ny < 0 || nx < 0 || ny >= grid_h as isize || nx >= grid_w as isize {
                continue;
            }
            let (ny, nx) = (ny as usize, nx as usize);
            let before = grid[ny][nx].len();
            let mut allowed = HashSet::new();
            // For each candidate in the current cell, add possible neighbors.
            for &t in &cell_candidates {
                allowed.extend(compat[t][d].iter());
            }
            // Intersection with neighbor's current possibilities.
            grid[ny][nx] = grid[ny][nx].intersection(&allowed).cloned().collect();
            if grid[ny][nx].is_empty() {
                panic!("Contradiction encountered!");
            }
            if grid[ny][nx].len() < before {
                queue.push_back((ny, nx));
            }
        }
    }
}
