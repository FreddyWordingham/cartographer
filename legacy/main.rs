use cartographer::{PatternSet, TileSet};
use photo::ImageRGBA;

const TILE_SIZE: [usize; 2] = [3, 3];

/// Read command line arguments.
fn read_inputs() -> (String, [usize; 2]) {
    let args: Vec<String> = std::env::args().collect();
    println!("{:?}", args.len());
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
    let (example_image_filepath, _output_map_resolution) = read_inputs();

    let example_map =
        ImageRGBA::<u8>::load(example_image_filepath).expect("Failed to load example map image.");
    println!("{}", example_map);

    let mut pattern_set = PatternSet::new(TILE_SIZE);
    pattern_set.ingest(&example_map);
    println!("{}", &pattern_set);

    let tile_set = TileSet::new(pattern_set);
}
