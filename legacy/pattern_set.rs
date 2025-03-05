use ndarray::s;
use photo::ImageRGBA;

use crate::{Direction, Pattern, TileState};

pub struct PatternSet {
    pub pattern_size: [usize; 2],
    pub patterns: Vec<Pattern>,
}

impl PatternSet {
    pub fn new(pattern_size: [usize; 2]) -> Self {
        debug_assert!(pattern_size[0] > 0);
        debug_assert!(pattern_size[1] > 0);

        PatternSet {
            pattern_size,
            patterns: Vec::new(),
        }
    }

    pub fn ingest(&mut self, map: &ImageRGBA<u8>) {
        let height = map.height();
        let width = map.width();

        debug_assert!(
            height >= self.pattern_size[0],
            "Map height is too small for pattern size."
        );
        debug_assert!(
            width >= self.pattern_size[1],
            "Map width is too small for pattern size."
        );

        // Iterate over all possible pattern windows in the map image
        for pattern_view in map
            .data
            .windows([self.pattern_size[0], self.pattern_size[1], 4])
        {
            let mut found = false;
            for pattern in &mut self.patterns {
                if let Some(trans) = pattern.equal_under_transformation(&pattern_view) {
                    pattern.add_transformation(trans);
                    pattern.frequency += 1;
                    found = true;
                    break;
                }
            }
            if !found {
                let pattern_image = ImageRGBA::new(pattern_view.to_owned());
                self.patterns.push(Pattern::new(pattern_image, 1));
            }
        }
    }

    /// Return the number of unique patterns-orientations.
    pub fn len(&self) -> usize {
        self.patterns.iter().map(|p| p.transformations.len()).sum()
    }

    pub fn valid_neighbours(
        &self,
        centre: &TileState,
        neighbour: &TileState,
        direction: Direction,
    ) -> bool {
        match direction {
            Direction::North => {
                let centre_image = centre.image(&self);
                let neighbour_image = neighbour.image(&self);
                check_north(&centre_image, &neighbour_image)
            }
            Direction::East => {
                let centre_image = centre.image(&self);
                let neighbour_image = neighbour.image(&self);
                check_east(&centre_image, &neighbour_image)
            }
            Direction::South => {
                let centre_image = centre.image(&self);
                let neighbour_image = neighbour.image(&self);
                check_south(&centre_image, &neighbour_image)
            }
            Direction::West => {
                let centre_image = centre.image(&self);
                let neighbour_image = neighbour.image(&self);
                check_west(&centre_image, &neighbour_image)
            }
        }
    }
}

impl std::fmt::Display for PatternSet {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for (n, pattern) in self.patterns.iter().enumerate() {
            let transformations_str: String = pattern
                .transformations
                .iter()
                .map(|t| format!("{}", t))
                .collect::<Vec<String>>()
                .join(", ");
            writeln!(
                f,
                "Tile {} frequency {:?} transformations {}:\n{}",
                n, pattern.frequency, transformations_str, pattern.image
            )?;
        }
        Ok(())
    }
}

fn check_east(centre_image: &ImageRGBA<u8>, right_image: &ImageRGBA<u8>) -> bool {
    let width = centre_image.width();
    let centre = centre_image.data.slice(s![.., 1..width, ..]);
    let right = right_image.data.slice(s![.., 0..width - 1, ..]);
    centre == right
}

fn check_west(centre_image: &ImageRGBA<u8>, left_image: &ImageRGBA<u8>) -> bool {
    let width = centre_image.width();
    let centre = centre_image.data.slice(s![.., 0..width - 1, ..]);
    let left = left_image.data.slice(s![.., 1..width, ..]);
    centre == left
}

fn check_north(centre_image: &ImageRGBA<u8>, top_image: &ImageRGBA<u8>) -> bool {
    let height = centre_image.height();
    let centre = centre_image.data.slice(s![0..(height - 1), .., ..]);
    let top = top_image.data.slice(s![1..height, .., ..]);
    centre == top
}

fn check_south(centre_image: &ImageRGBA<u8>, bottom_image: &ImageRGBA<u8>) -> bool {
    let height = centre_image.height();
    let centre = centre_image.data.slice(s![1..height, .., ..]);
    let bottom = bottom_image.data.slice(s![0..(height - 1), .., ..]);
    centre == bottom
}
