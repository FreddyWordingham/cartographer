use ndarray::Array2;
use photo::ImageRGBA;
use wave_function_collapse::{Ruleset, WaveFunction, map_tiles, print_images_with_captions};

const INPUT_DIR: &str = "input";
const OUTPUT_DIR: &str = "output";
const TILE_SIZE: [usize; 2] = [3, 3];
const OUTPUT_MAP_SIZE: [usize; 2] = [55, 50];

#[allow(dead_code)]
fn generate_rules(map: &Array2<usize>) -> Ruleset {
    let rules = Ruleset::new(&map);
    for (n, rule) in rules.rules.iter().enumerate() {
        println!("Rule {}", n);
        println!("North: {:?}", rule.north);
        println!("South: {:?}", rule.south);
        println!("East: {:?}", rule.east);
        println!("West: {:?}", rule.west);
        println!();
    }
    let rules_filepath = format!("{}/rules.yaml", OUTPUT_DIR);
    rules.save(&rules_filepath);
    rules
}

#[allow(dead_code)]
fn load_rules(filepath: &str) -> Ruleset {
    let rules = Ruleset::load(filepath);
    for (n, rule) in rules.rules.iter().enumerate() {
        println!("Rule {}", n);
        println!("North: {:?}", rule.north);
        println!("South: {:?}", rule.south);
        println!("East: {:?}", rule.east);
        println!("West: {:?}", rule.west);
        println!();
    }
    rules
}

fn main() {
    let image_name = "tileset.png";
    let filepath = format!("{}/{}", INPUT_DIR, image_name);
    let image = ImageRGBA::<u8>::load(filepath).expect("Failed to load image");
    println!("{}", image);

    let image_tiles = image.tiles(TILE_SIZE);
    let unique_tiles = image.unique_tiles(TILE_SIZE);
    print_images_with_captions(unique_tiles.as_slice(), 1);

    let tile_mapping = map_tiles(&image_tiles, &unique_tiles);
    println!("{:?}", tile_mapping);

    let rules = generate_rules(&tile_mapping);
    let mut wave_function = WaveFunction::new(OUTPUT_MAP_SIZE, rules);

    let out_map = wave_function
        .collapse()
        .expect("Failed to collapse wave function");
    let output = ImageRGBA::new_from_mapping(
        &out_map,
        unique_tiles
            .into_iter()
            .map(|(tile, _)| tile)
            .collect::<Vec<_>>()
            .as_slice(),
    );
    // println!("{}", output);
    let output_filepath = format!("{}/output.png", OUTPUT_DIR);
    output.save(&output_filepath).expect("Failed to save image");
}
