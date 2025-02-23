use ndarray::Array2;

pub fn map_tiles<T: PartialEq>(
    image_tiles: &Array2<T>,
    unique_tiles: &[(T, usize)],
) -> Array2<usize> {
    let mut tile_mapping = Array2::<usize>::zeros(image_tiles.dim());
    for (map_index, tile) in tile_mapping.iter_mut().zip(image_tiles.iter()) {
        for (unique_tile_index, (unique_tile, _frequency)) in unique_tiles.iter().enumerate() {
            if tile == unique_tile {
                *map_index = unique_tile_index;
                break;
            }
        }
    }
    tile_mapping
}
