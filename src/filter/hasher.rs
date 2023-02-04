use std::hash::{Hasher, BuildHasher};

#[derive(Clone)]
pub struct Murmur3Hasher {
    state: u64,
}

impl Hasher for Murmur3Hasher {
    fn finish(&self) -> u64 {
        self.state
    }

    fn write(&mut self, bytes: &[u8]) {
        self.state = fastmurmur3::hash(bytes) as u64;
    }
}

#[derive(Clone)]
pub struct BuildMurmur3Hasher;

impl BuildHasher for BuildMurmur3Hasher {
    type Hasher = Murmur3Hasher;

    fn build_hasher(&self) -> Self::Hasher {
        Murmur3Hasher  { state: 0 }
    }
}