use core::{
    alloc::Layout,
    ptr::{NonNull, null_mut},
};

use alloc::{
    alloc::{alloc, dealloc},
    slice,
    sync::Arc,
    vec,
};
use virtio_drivers::{
    Hal, PhysAddr,
    device::rng::VirtIORng,
    transport::{
        DeviceType, Transport,
        pci::{PciTransport, bus::PciRoot, virtio_device_type},
    },
};

use crate::{mutex::Mutex, print};

mod pci;

pub fn init() -> VirtIO<PciTransport> {
    let mut pciroot = PciRoot::new(unsafe { pci::IoCam::new() });
    let mut virtio = VirtIO::new();
    for (dev, info) in pciroot.enumerate_bus(0) {
        if let Some(_) = virtio_device_type(&info) {
            let transport = PciTransport::new::<HalImpl, _>(&mut pciroot, dev).unwrap();
            match transport.device_type() {
                DeviceType::EntropySource => init_virtio_random(&mut virtio, transport),
                _ => (),
            }
        }
    }
    virtio
}

fn init_virtio_random<T: Transport>(virtio: &mut VirtIO<T>, transport: T) {
    match VirtIORng::new(transport) {
        Ok(rng) => {
            virtio.random = Some(Arc::new(Mutex::new(rng)));
        }
        Err(e) => {
            print!("{e}");
        }
    }
}

pub struct VirtIO<T: Transport> {
    random: Option<Arc<Mutex<VirtIORng<HalImpl, T>>>>,
}

impl<T: Transport> Clone for VirtIO<T> {
    fn clone(&self) -> Self {
        Self {
            random: self.random.clone(),
        }
    }
}

impl<T: Transport> Default for VirtIO<T> {
    fn default() -> Self {
        Self { random: None }
    }
}

impl<T: Transport> VirtIO<T> {
    pub fn new() -> Self {
        Default::default()
    }
}

impl<T: Transport> wasi_random::HostRandom for VirtIO<T> {
    fn get_entropy(&self, len: usize) -> alloc::vec::Vec<u8> {
        let entropy = self
            .random
            .as_ref()
            .expect("VirtIO RNG has not been initialized");
        let mut seek = 0;
        let mut buf = vec![0u8; len];
        while seek < len {
            match entropy.lock().request_entropy(&mut buf[seek..]) {
                Ok(bytes) => {
                    seek += bytes;
                }
                Err(e) => {
                    print!("{e}");
                    return buf;
                }
            }
        }
        buf
    }
}

struct HalImpl {}

unsafe impl Hal for HalImpl {
    fn dma_alloc(
        pages: usize,
        _direction: virtio_drivers::BufferDirection,
    ) -> (virtio_drivers::PhysAddr, core::ptr::NonNull<u8>) {
        if pages == 0 {
            panic!("pages == 0");
        }
        let layout = Layout::from_size_align(pages * 0x1000, 0x1000).unwrap();
        let mem: &mut [u8] = unsafe {
            let mem = alloc(layout);
            if mem == null_mut() {
                panic!("alloc failure");
            }
            slice::from_raw_parts_mut(mem, pages * 0x1000)
        };
        mem.fill(0);
        (
            mem.as_ptr() as usize,
            NonNull::new(mem.as_mut_ptr()).unwrap(),
        )
    }

    unsafe fn dma_dealloc(_paddr: PhysAddr, vaddr: core::ptr::NonNull<u8>, pages: usize) -> i32 {
        let layout = Layout::from_size_align(pages * 0x1000, 0x1000).unwrap();
        unsafe {
            dealloc(vaddr.as_ptr(), layout);
        }
        0
    }

    unsafe fn mmio_phys_to_virt(paddr: PhysAddr, _size: usize) -> core::ptr::NonNull<u8> {
        NonNull::new(paddr as _).unwrap()
    }

    unsafe fn share(
        buffer: core::ptr::NonNull<[u8]>,
        _direction: virtio_drivers::BufferDirection,
    ) -> PhysAddr {
        buffer.cast::<*mut u8>().as_ptr() as usize
    }

    unsafe fn unshare(
        _paddr: PhysAddr,
        _buffer: core::ptr::NonNull<[u8]>,
        _direction: virtio_drivers::BufferDirection,
    ) {
    }
}
