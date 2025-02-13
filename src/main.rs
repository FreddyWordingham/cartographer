use ndarray::{Array2, Array3};
use ndarray_images::Image;
use rand::{rng, Rng};
use std::{
    collections::{HashMap, HashSet, VecDeque},
    fs,
};

#[derive(Clone)]
struct Pattern {
    data: Vec<u8>, // row-major, size = tile_h * tile_w * channels
    freq: usize,
}

#[derive(Clone, Copy, Debug)]
enum Dir {
    Up = 0,
    UpRight = 1,
    Right = 2,
    DownRight = 3,
    Down = 4,
    DownLeft = 5,
    Left = 6,
    UpLeft = 7,
}

fn opposite(d: Dir) -> Dir {
    match d {
        Dir::Up => Dir::Down,
        Dir::UpRight => Dir::DownLeft,
        Dir::Right => Dir::Left,
        Dir::DownRight => Dir::UpLeft,
        Dir::Down => Dir::Up,
        Dir::DownLeft => Dir::UpRight,
        Dir::Left => Dir::Right,
        Dir::UpLeft => Dir::DownRight,
    }
}

// Rotate a square tile 90° clockwise.
fn rotate_90(tile: &[u8], size: usize, ch: usize) -> Vec<u8> {
    let mut new_tile = vec![0; tile.len()];
    for r in 0..size {
        for c in 0..size {
            for k in 0..ch {
                new_tile[(r * size + c) * ch + k] = tile[((size - 1 - c) * size + r) * ch + k];
            }
        }
    }
    new_tile
}

// Mirror a square tile horizontally.
fn mirror(tile: &[u8], size: usize, ch: usize) -> Vec<u8> {
    let mut new_tile = vec![0; tile.len()];
    for r in 0..size {
        for c in 0..size {
            for k in 0..ch {
                new_tile[(r * size + c) * ch + k] = tile[(r * size + (size - 1 - c)) * ch + k];
            }
        }
    }
    new_tile
}

// Generate all 8 variants (identity, rotations, mirrored variants) and deduplicate.
fn generate_variants(tile: &Vec<u8>, size: usize, ch: usize) -> Vec<Vec<u8>> {
    let mut variants = Vec::new();
    variants.push(tile.clone());
    let r90 = rotate_90(tile, size, ch);
    variants.push(r90.clone());
    let r180 = rotate_90(&r90, size, ch);
    variants.push(r180.clone());
    let r270 = rotate_90(&r180, size, ch);
    variants.push(r270.clone());
    let m = mirror(tile, size, ch);
    variants.push(m.clone());
    let m90 = rotate_90(&m, size, ch);
    variants.push(m90.clone());
    let m180 = rotate_90(&m90, size, ch);
    variants.push(m180.clone());
    let m270 = rotate_90(&m180, size, ch);
    variants.push(m270.clone());
    let mut set = HashSet::new();
    variants
        .into_iter()
        .filter(|v| set.insert(v.clone()))
        .collect()
}

// Propagate constraints over 8 directions. Returns false if a contradiction occurs.
fn propagate(
    grid: &mut Vec<Vec<HashSet<usize>>>,
    allowed: &Vec<[HashSet<usize>; 8]>,
    grid_h: usize,
    grid_w: usize,
) -> bool {
    let mut queue: VecDeque<(usize, usize)> = VecDeque::new();
    for r in 0..grid_h {
        for c in 0..grid_w {
            if grid[r][c].len() == 1 {
                queue.push_back((r, c));
            }
        }
    }
    let directions = [
        (-1, 0, Dir::Up),
        (-1, 1, Dir::UpRight),
        (0, 1, Dir::Right),
        (1, 1, Dir::DownRight),
        (1, 0, Dir::Down),
        (1, -1, Dir::DownLeft),
        (0, -1, Dir::Left),
        (-1, -1, Dir::UpLeft),
    ];
    while let Some((r, c)) = queue.pop_front() {
        let current = grid[r][c].clone();
        for (dr, dc, d) in directions.iter().cloned() {
            let nr = r as isize + dr;
            let nc = c as isize + dc;
            if nr < 0 || nr >= grid_h as isize || nc < 0 || nc >= grid_w as isize {
                continue;
            }
            let nr = nr as usize;
            let nc = nc as usize;
            let before = grid[nr][nc].len();
            let opp = opposite(d);
            grid[nr][nc].retain(|&q| {
                current.iter().any(|&p| {
                    allowed[p][d as usize].contains(&q) || allowed[q][opp as usize].contains(&p)
                })
            });
            if grid[nr][nc].is_empty() {
                return false;
            }
            if grid[nr][nc].len() < before {
                queue.push_back((nr, nc));
            }
        }
    }
    true
}

// Backtracking solver using an entropy parameter.
fn solve(
    grid: Vec<Vec<HashSet<usize>>>,
    allowed: &Vec<[HashSet<usize>; 8]>,
    grid_h: usize,
    grid_w: usize,
    entropy: f32,
) -> Option<Vec<Vec<HashSet<usize>>>> {
    let mut grid = grid;
    if !propagate(&mut grid, allowed, grid_h, grid_w) {
        return None;
    }
    if grid
        .iter()
        .all(|row| row.iter().all(|cell| cell.len() == 1))
    {
        return Some(grid);
    }
    let mut candidates = Vec::new();
    let mut min_possible = usize::MAX;
    for r in 0..grid_h {
        for c in 0..grid_w {
            let count = grid[r][c].len();
            if count > 1 {
                candidates.push((r, c, count));
                if count < min_possible {
                    min_possible = count;
                }
            }
        }
    }
    let mut total_weight = 0.0;
    for &(_r, _c, count) in &candidates {
        total_weight += f32::exp(-((count - min_possible) as f32) / entropy);
    }
    let mut rng = rng();
    let mut choice = rng.random_range(0.0..total_weight);
    let mut selected_candidate = None;
    for &(r, c, count) in &candidates {
        let weight = f32::exp(-((count - min_possible) as f32) / entropy);
        if choice < weight {
            selected_candidate = Some((r, c));
            break;
        } else {
            choice -= weight;
        }
    }
    let (r, c) = selected_candidate.expect("No candidate cell selected");
    for &option in grid[r][c].iter() {
        let mut grid_clone = grid.clone();
        grid_clone[r][c] = vec![option].into_iter().collect();
        if let Some(solution) = solve(grid_clone, allowed, grid_h, grid_w, entropy) {
            return Some(solution);
        }
    }
    None
}

pub fn wave_function_collapse_backtracking(
    example: &Array3<u8>,
    tile_size: [usize; 2],
    slide: [usize; 2],
    target_resolution: [usize; 2],
    entropy: f32,
) -> Array3<u8> {
    let (tile_h, tile_w) = (tile_size[0], tile_size[1]);
    let (slide_h, slide_w) = (slide[0], slide[1]);
    let (ex_h, ex_w, ch) = {
        let dims = example.dim();
        (dims.0, dims.1, dims.2)
    };

    fs::create_dir_all("output").unwrap();

    // 1. Extract overlapping colored patterns with variants.
    let mut pattern_map: HashMap<Vec<u8>, usize> = HashMap::new();
    let mut patterns: Vec<Pattern> = Vec::new();
    let mut sample_count = 0;
    for i in (0..=ex_h - tile_h).step_by(slide_h) {
        for j in (0..=ex_w - tile_w).step_by(slide_w) {
            sample_count += 1;
            let mut tile = Vec::with_capacity(tile_h * tile_w * ch);
            for di in 0..tile_h {
                for dj in 0..tile_w {
                    for c in 0..ch {
                        tile.push(example[[i + di, j + dj, c]]);
                    }
                }
            }
            let variants = generate_variants(&tile, tile_w, ch);
            for variant in variants {
                if let Some(&idx) = pattern_map.get(&variant) {
                    patterns[idx].freq += 1;
                } else {
                    let idx = patterns.len();
                    pattern_map.insert(variant.clone(), idx);
                    patterns.push(Pattern {
                        data: variant,
                        freq: 1,
                    });
                }
            }
        }
    }
    println!("Number of samples taken: {}", sample_count);
    for (i, pattern) in patterns.iter().enumerate() {
        println!("Tile {} occurred {} times", i, pattern.freq);
        let tile_array: Array3<u8> =
            Array3::from_shape_vec((tile_h, tile_w, ch), pattern.data.clone()).unwrap();
        let tile_array_f: Array3<f32> = tile_array.mapv(|x| (x as f32) / 255.0);
        let path = format!("output/tile_{}.png", i);
        tile_array_f.save(&path).unwrap();
    }
    let num_patterns = patterns.len();

    // 2. Build adjacency rules for 8 directions.
    let mut allowed: Vec<[HashSet<usize>; 8]> = vec![
        [
            HashSet::new(),
            HashSet::new(),
            HashSet::new(),
            HashSet::new(),
            HashSet::new(),
            HashSet::new(),
            HashSet::new(),
            HashSet::new(),
        ];
        num_patterns
    ];
    for i in 0..num_patterns {
        for j in 0..num_patterns {
            // Up: pattern i's top row equals pattern j's bottom row.
            let mut ok = true;
            for c in 0..tile_w {
                if patterns[i].data[c] != patterns[j].data[((tile_h - 1) * tile_w + c) * ch] {
                    ok = false;
                    break;
                }
            }
            if ok {
                allowed[i][Dir::Up as usize].insert(j);
                allowed[j][Dir::Down as usize].insert(i);
            }
            // Right: pattern i's right column equals pattern j's left column.
            ok = true;
            for r in 0..tile_h {
                let idx_i = (r * tile_w + (tile_w - 1)) * ch;
                let idx_j = (r * tile_w + 0) * ch;
                for c in 0..ch {
                    if patterns[i].data[idx_i + c] != patterns[j].data[idx_j + c] {
                        ok = false;
                        break;
                    }
                }
                if !ok {
                    break;
                }
            }
            if ok {
                allowed[i][Dir::Right as usize].insert(j);
                allowed[j][Dir::Left as usize].insert(i);
            }
            // Down: pattern i's bottom row equals pattern j's top row.
            ok = true;
            for c in 0..tile_w {
                let idx_i = ((tile_h - 1) * tile_w + c) * ch;
                let idx_j = (0 * tile_w + c) * ch;
                for d in 0..ch {
                    if patterns[i].data[idx_i + d] != patterns[j].data[idx_j + d] {
                        ok = false;
                        break;
                    }
                }
                if !ok {
                    break;
                }
            }
            if ok {
                allowed[i][Dir::Down as usize].insert(j);
                allowed[j][Dir::Up as usize].insert(i);
            }
            // Left: pattern i's left column equals pattern j's right column.
            ok = true;
            for r in 0..tile_h {
                let idx_i = (r * tile_w + 0) * ch;
                let idx_j = (r * tile_w + (tile_w - 1)) * ch;
                for c in 0..ch {
                    if patterns[i].data[idx_i + c] != patterns[j].data[idx_j + c] {
                        ok = false;
                        break;
                    }
                }
                if !ok {
                    break;
                }
            }
            if ok {
                allowed[i][Dir::Left as usize].insert(j);
                allowed[j][Dir::Right as usize].insert(i);
            }
            // Diagonals:
            // UpRight: pattern i's top-right equals pattern j's bottom-left.
            if patterns[i].data[(tile_w - 1) * ch]
                == patterns[j].data[((tile_h - 1) * tile_w + 0) * ch]
            {
                allowed[i][Dir::UpRight as usize].insert(j);
                allowed[j][Dir::DownLeft as usize].insert(i);
            }
            // DownRight: pattern i's bottom-right equals pattern j's top-left.
            if patterns[i].data[((tile_h - 1) * tile_w + (tile_w - 1)) * ch] == patterns[j].data[0]
            {
                allowed[i][Dir::DownRight as usize].insert(j);
                allowed[j][Dir::UpLeft as usize].insert(i);
            }
            // DownLeft: pattern i's bottom-left equals pattern j's top-right.
            if patterns[i].data[(tile_h - 1) * tile_w * ch] == patterns[j].data[(tile_w - 1) * ch] {
                allowed[i][Dir::DownLeft as usize].insert(j);
                allowed[j][Dir::UpRight as usize].insert(i);
            }
            // UpLeft: pattern i's top-left equals pattern j's bottom-right.
            if patterns[i].data[0] == patterns[j].data[((tile_h - 1) * tile_w + (tile_w - 1)) * ch]
            {
                allowed[i][Dir::UpLeft as usize].insert(j);
                allowed[j][Dir::DownRight as usize].insert(i);
            }
        }
    }

    // 3. Set up the grid for pattern placement.
    let grid_h = target_resolution[0] - tile_h + 1;
    let grid_w = target_resolution[1] - tile_w + 1;
    let all: HashSet<usize> = (0..num_patterns).collect();
    let grid: Vec<Vec<HashSet<usize>>> = vec![vec![all; grid_w]; grid_h];

    // 4. Solve the grid using backtracking.
    let solved_grid = solve(grid, &allowed, grid_h, grid_w, entropy)
        .expect("No solution found with backtracking!");

    // 5. Reconstruct the final image by averaging overlapping contributions.
    let (out_h, out_w) = (target_resolution[0], target_resolution[1]);
    let mut acc = Array3::<f32>::zeros((out_h, out_w, ch));
    let mut count = Array2::<u32>::zeros((out_h, out_w));
    for r in 0..grid_h {
        for c in 0..grid_w {
            let &p = solved_grid[r][c].iter().next().unwrap();
            for di in 0..tile_h {
                for dj in 0..tile_w {
                    let y = r + di;
                    let x = c + dj;
                    let idx = (di * tile_w + dj) * ch;
                    for cc in 0..ch {
                        acc[[y, x, cc]] += patterns[p].data[idx + cc] as f32;
                    }
                    count[[y, x]] += 1;
                }
            }
        }
    }
    let mut output = Array3::<u8>::zeros((out_h, out_w, ch));
    for y in 0..out_h {
        for x in 0..out_w {
            for cc in 0..ch {
                let avg = acc[[y, x, cc]] / (count[[y, x]] as f32);
                output[[y, x, cc]] = avg.round().clamp(0.0, 255.0) as u8;
            }
        }
    }
    output
}

fn main() {
    println!("Hello, world!");
    let example_filepath = "input/island.png";
    let example: Array3<f32> = Image::load(example_filepath).unwrap();
    let example: Array3<u8> = example.mapv(|x| (x * 255.0) as u8);
    let tile_size = [2, 2];
    let slide = [1, 1];
    let target_resolution = [40, 40];
    let entropy = 0.2;
    let output_data =
        wave_function_collapse_backtracking(&example, tile_size, slide, target_resolution, entropy);
    let output_filepath = "output/simple_wfc_backtracking.png";
    let output_data: Array3<f32> = output_data.mapv(|x| (x as f32) / 255.0);
    println!("Image shape: {:?}", output_data.shape());
    output_data.save(&output_filepath).unwrap();
}
