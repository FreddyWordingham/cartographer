use ndarray::ArrayView3;
use photo::{ALL_TRANSFORMATIONS, ImageRGBA, Transformation};
use std::collections::HashSet;

pub struct Pattern {
    pub image: ImageRGBA<u8>,
    pub transformations: HashSet<Transformation>,
    pub frequency: usize,
}

impl Pattern {
    pub fn new(image: ImageRGBA<u8>, frequency: usize) -> Self {
        debug_assert!(image.width() > 0);
        debug_assert!(image.height() > 0);
        debug_assert!(frequency > 0);

        let mut transformations = HashSet::new();
        transformations.insert(Transformation::Identity);

        Pattern {
            image,
            transformations,
            frequency,
        }
    }

    pub fn num_transformations(&self) -> usize {
        self.transformations.len()
    }

    pub fn add_transformation(&mut self, transformation: Transformation) {
        self.transformations.insert(transformation);
    }

    #[allow(dead_code)]
    pub fn add_all_transformations(&mut self) {
        for transformation in ALL_TRANSFORMATIONS {
            self.transformations.insert(transformation);
        }
    }

    /// Returns the transformation that makes the image equal to the other image, if it exists.
    pub fn equal_under_transformation(&mut self, other: &ArrayView3<u8>) -> Option<Transformation> {
        for trans in &ALL_TRANSFORMATIONS {
            if self.image.transform(*trans).data == *other {
                return Some(*trans);
            }
        }
        None
    }
}
