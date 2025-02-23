use ndarray::Axis;
use photo::ImageRGBA;
use std::fmt::Display;

pub fn print_images_in_row(images: &[&ImageRGBA<u8>], gap: usize) {
    if images.is_empty() {
        return;
    }
    // Determine the maximum height among all images.
    let max_height = images.iter().map(|img| img.data.dim().0).max().unwrap();

    // For each row across the maximum height.
    for row in 0..max_height {
        for (i, image) in images.iter().enumerate() {
            let (img_height, img_width, _) = image.data.dim();
            if row < img_height {
                // Print the row's pixels.
                let row_data = image.data.index_axis(Axis(0), row);
                for pixel in row_data.outer_iter() {
                    let r = pixel[0];
                    let g = pixel[1];
                    let b = pixel[2];
                    let a = pixel[3];
                    // Each pixel is rendered as two spaces with background color.
                    print!("\x1b[48;2;{r};{g};{b};{a}m  \x1b[0m");
                }
            } else {
                // If this image doesn't have a row here, print blank spaces equal to its width.
                print!("{}", " ".repeat(img_width * 2));
            }
            // Print the gap between images (except after the last one).
            if i < images.len() - 1 {
                print!("{}", " ".repeat(gap));
            }
        }
        println!();
    }
}

pub fn print_images_with_captions<T: Display>(
    images_with_captions: &[(ImageRGBA<u8>, T)],
    gap: usize,
) {
    if images_with_captions.is_empty() {
        return;
    }
    // Collect image references.
    let images: Vec<&ImageRGBA<u8>> = images_with_captions.iter().map(|(img, _)| img).collect();
    print_images_in_row(&images, gap);

    // Now, print each caption centered beneath its image.
    for (i, (img, caption)) in images_with_captions.iter().enumerate() {
        // Each pixel renders as two spaces, so the printed width is:
        let printed_width = img.data.dim().1 * 2;
        let caption_str = caption.to_string();
        let caption_len = caption_str.len();
        let left_padding = if printed_width > caption_len {
            (printed_width - caption_len) / 2
        } else {
            0
        };
        print!("{}{}", " ".repeat(left_padding), caption_str);
        let right_padding = if printed_width > caption_len {
            printed_width - caption_len - left_padding
        } else {
            0
        };
        print!("{}", " ".repeat(right_padding));
        if i < images_with_captions.len() - 1 {
            print!("{}", " ".repeat(gap));
        }
    }
    println!();
}
