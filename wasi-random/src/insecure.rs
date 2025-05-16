use alloc::vec;
use rand::RngCore;

use crate::{HostRandom, RandomImpl, wasi::random::insecure::Host};

impl<T: HostRandom> Host for RandomImpl<T> {
    fn get_insecure_random_bytes(&mut self, len: u64) -> wasmtime::component::__internal::Vec<u8> {
        let mut vec = vec![0u8; len as usize];
        self.rng.fill_bytes(&mut vec);
        vec
    }

    fn get_insecure_random_u64(&mut self) -> u64 {
        self.rng.next_u64()
    }
}
