use ndarray::{Array2, Array3, Axis};
use ndarray_images::Image;

fn main() {
    println!("Hello, world!");

    let example_filepath = "input/rooms.png";
    let mut example: Array2<f32> = Image::load(example_filepath).unwrap();
    let resolution = [example.shape()[0], example.shape()[1]];
    let num_tiles = [4, 4];
    let tile_size = [
        example.shape()[0] / num_tiles[0],
        example.shape()[1] / num_tiles[1],
    ];

    println!("Tile size: {:?}", tile_size);

    // Draw lines
    for i in 0..resolution[0] {
        for j in 0..resolution[1] {
            if (i % tile_size[0] == 0) || (j % tile_size[1] == 0) {
                example[[i, j]] = 0.5;
            }
        }
    }

    println!("Image shape: {:?}", example.shape());

    let output_filepath = "output/rooms_lines.png";
    example.save(&output_filepath).unwrap();
}
