use bit_vec::BitVec;
use serde::{Deserialize, Serialize};

/// The allowed indices for each side of a tile.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Rule {
    #[serde(with = "bitvec_as_usize_list")]
    pub north: BitVec,
    #[serde(with = "bitvec_as_usize_list")]
    pub south: BitVec,
    #[serde(with = "bitvec_as_usize_list")]
    pub west: BitVec,
    #[serde(with = "bitvec_as_usize_list")]
    pub east: BitVec,
}

impl Rule {
    pub fn new(num_tiles: usize) -> Self {
        Self {
            north: BitVec::from_elem(num_tiles, false),
            south: BitVec::from_elem(num_tiles, false),
            east: BitVec::from_elem(num_tiles, false),
            west: BitVec::from_elem(num_tiles, false),
        }
    }
}

mod bitvec_as_usize_list {
    use super::*;
    use serde::de::{SeqAccess, Visitor};
    use serde::{Deserializer, Serializer};
    use std::fmt;

    // Serialize BitVec as a Vec<usize> of indices where bits are true.
    pub fn serialize<S>(bv: &BitVec, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let indices: Vec<usize> = bv
            .iter()
            .enumerate()
            .filter_map(|(i, b)| if b { Some(i) } else { None })
            .collect();
        indices.serialize(serializer)
    }

    // Deserialize a Vec<usize> into a BitVec.
    pub fn deserialize<'de, D>(deserializer: D) -> Result<BitVec, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct BitVecVisitor;

        impl<'de> Visitor<'de> for BitVecVisitor {
            type Value = BitVec;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("a list of allowed tile indices")
            }

            fn visit_seq<A>(self, mut seq: A) -> Result<BitVec, A::Error>
            where
                A: SeqAccess<'de>,
            {
                let mut indices = Vec::new();
                while let Some(index) = seq.next_element::<usize>()? {
                    indices.push(index);
                }
                // Create a BitVec with size = max index + 1.
                let size = indices.iter().max().map_or(0, |&max| max + 1);
                let mut bv = BitVec::from_elem(size, false);
                for i in indices {
                    if i >= bv.len() {
                        bv.grow(i + 1 - bv.len(), false);
                    }
                    bv.set(i, true);
                }
                Ok(bv)
            }
        }

        deserializer.deserialize_seq(BitVecVisitor)
    }
}
