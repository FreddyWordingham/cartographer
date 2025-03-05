use cartographer::{PatternSet, TileSet, WaveFunction};
use ndarray::{Array, Array2, s};
use photo::ImageRGBA;

const TILE_SIZE: [usize; 2] = [3, 3];

/// Read command line arguments.
fn read_inputs() -> (String, [usize; 2]) {
    let args: Vec<String> = std::env::args().collect();
    if args.len() != 3 {
        eprintln!("Usage: {} <input_image> <output_resolution>", args[0]);
        std::process::exit(1);
    }
    let example_image_filepath = &args[1];
    let map_resolution = {
        // In the form "widthxheight".
        let s = &args[2];
        let mut parts = s.split('x');
        let width = parts.next().unwrap().parse::<usize>().unwrap();
        let height = parts.next().unwrap().parse::<usize>().unwrap();
        [height, width]
    };

    (example_image_filepath.to_string(), map_resolution)
}

fn main() {
    let (example_image_filepath, output_map_resolution) = read_inputs();

    let example_map =
        ImageRGBA::<u8>::load(example_image_filepath).expect("Failed to load example map image.");
    println!("{}", example_map);

    let tile_set = PatternSet::new(TILE_SIZE).ingest(&example_map).build();
    println!("Number of unique patterns: {}", tile_set.patterns.len());
    println!("Number of tiles: {}", tile_set.tiles.len());

    // let mut wave_function = WaveFunction::new(&tile_set, output_map_resolution);
    let init_image = ImageRGBA::<u8>::load("input/bw.png").expect("Failed to load init image.");
    println!("{}", init_image);
    let mut wave_function =
        WaveFunction::new_from_image(&tile_set, &init_image, &[[255u8, 0u8, 0u8, 255u8]])
            .expect("Failed to create wave function");

    wave_function.collapse();

    let map: Array2<usize> = wave_function.state();
    let map_image = collapsed_state_to_image(&tile_set, &map);
    println!("{}", map_image);
    map_image
        .save("output/1.png")
        .expect("Failed to save output image.");
}

pub fn collapsed_state_to_image(
    tile_set: &TileSet,
    state: &ndarray::Array2<usize>,
) -> ImageRGBA<u8> {
    let (rows, cols) = state.dim();
    // Each output pixel has 4 channels (RGBA)
    let mut data = Vec::with_capacity(rows * cols * 4);

    for i in 0..rows {
        for j in 0..cols {
            let tile_index = state[(i, j)];
            let tile = &tile_set.tiles[tile_index];
            // Recompute the tile image from its pattern and transformation.
            let tile_image = tile_set.patterns[tile.pattern_index]
                .image
                // .transform(tile.transformation)
                .clone() // We don't need to transform on a 3x3 tile
                ;
            // Extract the middle pixel (an array view of 4 elements).
            let pixel = tile_image.data.slice(s![1, 1, ..]);
            data.extend_from_slice(pixel.as_slice().unwrap());
        }
    }

    // Build the final image array from the collected pixel data.
    let image_array =
        Array::from_shape_vec((rows, cols, 4), data).expect("Failed to create image array");
    ImageRGBA::new(image_array)
}
