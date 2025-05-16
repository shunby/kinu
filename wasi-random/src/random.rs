use crate::{HostRandom, RandomImpl, wasi::random::random::Host};
use alloc::vec::Vec;

pub(crate) fn u8s_to_u64(vec: Vec<u8>) -> u64 {
    vec.iter()
        .map(|&e| e as u64)
        .reduce(|acc, e| acc * 8 + e)
        .unwrap_or(0)
}

impl<T: HostRandom> Host for RandomImpl<T> {
    fn get_random_bytes(&mut self, len: u64) -> Vec<u8> {
        self.host.get_entropy(len as usize)
    }

    fn get_random_u64(&mut self) -> u64 {
        u8s_to_u64(self.host.get_entropy(8))
    }
}
