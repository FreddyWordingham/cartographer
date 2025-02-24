use ndarray::Array2;
use photo::ImageRGBA;
use rand::SeedableRng;
use wave_function_collapse::{Ruleset, WaveFunction, map_tiles, print_images_with_captions};

const INPUT_DIR: &str = "input";
const OUTPUT_DIR: &str = "output";
const TILE_SIZE: [usize; 2] = [5, 5];
const OUTPUT_MAP_SIZE: [usize; 2] = [64, 64];
const FIRST_SEED: u64 = 0;
const ATTEMPTS: u64 = 100;

#[allow(dead_code)]
fn generate_rules(map: &Array2<usize>) -> Ruleset {
    let rules = Ruleset::new(&map);
    let rules_filepath = format!("{}/rules.yaml", OUTPUT_DIR);
    rules.save(&rules_filepath);
    rules
}

#[allow(dead_code)]
fn load_rules() -> Ruleset {
    let rules_filepath = format!("{}/rules.yaml", OUTPUT_DIR);
    let rules = Ruleset::load(&rules_filepath);
    rules
}

fn main() {
    let image_name = "tileset2.png";
    let filepath = format!("{}/{}", INPUT_DIR, image_name);
    let image = ImageRGBA::<u8>::load(filepath).expect("Failed to load image");
    // println!("{}", image);

    let image_tiles = image.tiles(TILE_SIZE);
    let unique_tiles = image.unique_tiles(TILE_SIZE);
    let sections = unique_tiles.len() / 8;
    for i in 0..sections {
        let start = i * 8;
        let end = (i + 1) * 8;
        print_images_with_captions(&unique_tiles[start..end], 1);
    }

    let tile_mapping = map_tiles(&image_tiles, &unique_tiles);

    let rules = generate_rules(&tile_mapping);
    // let rules = load_rules();
    let mut wave_function = WaveFunction::new(OUTPUT_MAP_SIZE, rules);

    let mut out_map = None;
    for seed in FIRST_SEED..FIRST_SEED + ATTEMPTS {
        let mut rng = rand::rngs::StdRng::seed_from_u64(seed);
        if let Some(map) = wave_function.collapse(&mut rng) {
            out_map = Some(map);
            println!("Collapsed wave function with seed {}", seed);
            break;
        } else {
            println!("Failed to collapse wave function with seed {}", seed);
        }
    }

    if let Some(map) = out_map {
        let output = ImageRGBA::new_from_mapping(
            &map,
            unique_tiles
                .into_iter()
                .map(|(tile, _)| tile)
                .collect::<Vec<_>>()
                .as_slice(),
        );

        let output_filepath = format!("{}/output.png", OUTPUT_DIR);
        // println!("{}", output);
        output.save(&output_filepath).expect("Failed to save image");
    } else {
        println!("Failed to collapse wave function");
    }
}
