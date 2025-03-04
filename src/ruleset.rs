use std::{collections::HashMap, fmt};

/// Represents a unique tile orientation state.
#[derive(Copy, Clone, Eq, PartialEq, Hash)]
struct TileState {
    tile_id: u32,    // Base tile identifier
    orientation: u8, // Orientation index (e.g., 0=0°, 1=90°, 2=180°, 3=270°, etc.)
}

/// Cardinal directions for adjacency (could be extended for 3D or hex grids).
#[derive(Debug, Copy, Clone)]
enum Direction {
    North,
    East,
    South,
    West,
}
const NUM_DIRS: usize = 4;

pub struct RuleSet {
    /// Total number of distinct tile states (each tile orientation counted separately).
    state_count: usize,
    /// Mapping from a (tile_id, orientation) to a state index in our arrays.
    state_index: HashMap<TileState, usize>,
    /// Inverse mapping from state index back to the TileState (for decoding results or debugging).
    index_to_state: Vec<TileState>,
    /// Adjacency rules: `neighbors[dir][s]` is a bitset (or list) of neighbor state indices allowed adjacent to state `s` on side `dir`.
    neighbors: [Vec<u64>; NUM_DIRS], // using a u64 bitset for example; use multiple u64s or a BitVec if state_count > 64
}

impl RuleSet {
    /// Creates a new RuleSet given a list of tiles and their allowed transformations.
    fn new(
        tile_variants: &[(u32, &[u8])],
        adjacency_list: &[((u32, u8), Direction, (u32, u8))],
    ) -> Self {
        // `tile_variants` might list each tile ID with its allowed orientations.
        // `adjacency_list` might list allowed neighbor pairs: ((tile, ori), dir, (neighbor_tile, neighbor_ori)).
        let mut state_index = HashMap::new();
        let mut index_to_state = Vec::new();
        let mut next_index = 0;
        // Assign an index to each tile orientation state
        for &(tile_id, orientations) in tile_variants {
            for &ori in orientations {
                let state = TileState {
                    tile_id,
                    orientation: ori,
                };
                state_index.insert(state, next_index);
                index_to_state.push(state);
                next_index += 1;
            }
        }
        let state_count = next_index;
        // Initialize neighbor bitsets
        let mut neighbors = [(); NUM_DIRS].map(|_| vec![0u64; state_count]);
        // Populate adjacency rules
        for &((tile, ori), dir, (nbr_tile, nbr_ori)) in adjacency_list {
            if let Some(&s_idx) = state_index.get(&TileState {
                tile_id: tile,
                orientation: ori,
            }) {
                if let Some(&nbr_idx) = state_index.get(&TileState {
                    tile_id: nbr_tile,
                    orientation: nbr_ori,
                }) {
                    // Set the bit corresponding to the neighbor index in the source state's neighbor bitset
                    let dir_idx = dir as usize;
                    if nbr_idx < 64 {
                        neighbors[dir_idx][s_idx] |= 1 << nbr_idx;
                    } else {
                        // If more than 64 states, you'd manage multiple u64s or use a dynamic bitset structure
                        // (For simplicity, assume state_count <= 64 here)
                    }
                }
            }
        }
        RuleSet {
            state_count,
            state_index,
            index_to_state,
            neighbors,
        }
    }

    /// Get allowed neighbor states (by index) for a given state index and direction.
    #[inline]
    fn allowed_neighbors(&self, state_idx: usize, dir: Direction) -> u64 {
        self.neighbors[dir as usize][state_idx]
    }

    /// Restrict a set of possible states (bitset) for a neighbor cell given the current possibilities of this cell.
    fn constrain_neighbor(&self, current_states: u64, dir: Direction, neighbor_possible: &mut u64) {
        let dir_idx = dir as usize;
        // Compute the union of allowed neighbors for all states in `current_states`.
        let mut allowed_union = 0u64;
        let mut bits = current_states;
        while bits != 0 {
            let s = bits.trailing_zeros() as usize; // get index of one state bit
            bits &= bits - 1; // clear the lowest set bit
            allowed_union |= self.neighbors[dir_idx][s];
        }
        // Intersect with neighbor's current possibilities
        *neighbor_possible &= allowed_union;
    }
}

// Display for debugging
impl fmt::Debug for RuleSet {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        writeln!(f, "RuleSet with {} states:", self.state_count)?;
        for (idx, state) in self.index_to_state.iter().enumerate() {
            writeln!(
                f,
                "  State {} = Tile {} (ori {})",
                idx, state.tile_id, state.orientation
            )?;
            for &dir in &[
                Direction::North,
                Direction::East,
                Direction::South,
                Direction::West,
            ] {
                let allowed = self.allowed_neighbors(idx, dir);
                write!(f, "    {:?}-> [", dir)?;
                // list allowed neighbor states by their base tile id for clarity
                let mut first = true;
                let mut bits = allowed;
                while bits != 0 {
                    let nbr_idx = bits.trailing_zeros() as usize;
                    bits &= bits - 1;
                    let nbr_state = &self.index_to_state[nbr_idx];
                    if !first {
                        write!(f, ", ")?;
                    }
                    write!(f, "{}(ori {})", nbr_state.tile_id, nbr_state.orientation)?;
                    first = false;
                }
                writeln!(f, "]")?;
            }
        }
        Ok(())
    }
}
