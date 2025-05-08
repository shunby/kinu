#![no_std]

pub mod elf;
pub mod mmap;
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
    }
}
