#![no_std]

use alloc::vec::Vec;
use rand::{SeedableRng, rngs::SmallRng};

extern crate alloc;
mod insecure;
mod random;
wasmtime::component::bindgen!("bindings");

pub struct RandomImpl<T: HostRandom> {
    host: T,
    rng: SmallRng,
}

impl<T: HostRandom> RandomImpl<T> {
    pub fn new(host: T) -> Self {
        let seed = host.get_entropy(32).try_into().unwrap();
        let rng = SmallRng::from_seed(seed);
        Self { host, rng }
    }
}

pub trait HostRandom {
    fn get_entropy(&self, len: usize) -> Vec<u8>;
}
