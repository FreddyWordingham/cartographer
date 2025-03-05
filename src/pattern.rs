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

    pub fn add_transformation(&mut self, transformation: Transformation) {
        self.transformations.insert(transformation);
    }

    /// Returns the transformation that can be applied to this pattern to make it equal to the other pattern,
    /// if it there is such a transformation.
    pub fn equal_under_transformation(&mut self, other: &ArrayView3<u8>) -> Option<Transformation> {
        for trans in &ALL_TRANSFORMATIONS {
            if self.image.transform(*trans).data == *other {
                return Some(*trans);
            }
        }
        None
    }

    pub fn transformed_image(&self, transformation: Transformation) -> ImageRGBA<u8> {
        assert!(self.transformations.contains(&transformation));
        self.image.transform(transformation)
    }
}
