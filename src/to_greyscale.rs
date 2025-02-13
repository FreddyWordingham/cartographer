use ndarray::{Array2, Array3, Axis};
use ndarray_images::Image;

fn main() {
    println!("Hello, world!");

    let example_filepath = "input/rooms.png";
    let example: Array3<f32> = Image::load(example_filepath).unwrap();

    println!("Image shape: {:?}", example.shape());

    let greyscale: Array2<f32> = example.index_axis(Axis(2), 0).to_owned();
    let output_filepath = "output/rooms.png";
    greyscale.save(&output_filepath).unwrap();
}
