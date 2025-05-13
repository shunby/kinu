#![no_std]
#![no_main]

use lib::{elf, mmap::CMemoryMap};
use log::info;
use uefi::{
    boot::MemoryType,
    prelude::*,
    proto::media::file::{File, FileAttribute, FileInfo, FileMode},
    CStr16,
};

const KERNEL_FILE_NAME: &CStr16 = cstr16!("\\kernel.elf");

type EntryPointFn = extern "sysv64" fn(CMemoryMap) -> !;

fn to_pages(n: u64) -> usize {
    ((n + 0xfff) / 0x1000) as usize
}

fn load_kernel() -> uefi::Result<EntryPointFn> {
    let mut fs = boot::get_image_file_system(boot::image_handle())?;
    let mut kernel_file = fs
        .open_volume()?
        .open(KERNEL_FILE_NAME, FileMode::Read, FileAttribute::empty())?
        .into_regular_file()
        .expect("kernel.elf is not a regular file");

    // The size of FileInfo is 80 + filename
    let mut fileinfo_buf = [0u8; 128];
    let fileinfo: &FileInfo = kernel_file
        .get_info(&mut fileinfo_buf)
        .expect("failed to get fileinfo of kernel.elf");
    let kernel_size = fileinfo.file_size();

    // read kernel.elf into buffer
    let kernel_file_buf: &mut [u8] = unsafe {
        let n_pages = to_pages(kernel_size);
        let page_ptr = boot::allocate_pages(
            boot::AllocateType::AnyPages,
            MemoryType::LOADER_DATA,
            n_pages,
        )?;
        &mut *core::ptr::slice_from_raw_parts_mut(page_ptr.as_ptr(), n_pages * 0x1000)
    };

    kernel_file.read(kernel_file_buf)?;
    let kernel_elf = unsafe {
        // TODO: implement ELF sanity check
        elf::ElfFile::from_buffer(&kernel_file_buf)
    };
    let elf_range = kernel_elf.load_address_range();

    let pages: &mut [u8] = unsafe {
        core::slice::from_raw_parts_mut(
            boot::allocate_pages(
                boot::AllocateType::Address(elf_range.0),
                MemoryType::LOADER_DATA,
                ((elf_range.1 - elf_range.0 + 0xfff) / 0x1000) as usize,
            )?
            .as_ptr(),
            (elf_range.1 - elf_range.0) as usize,
        )
    };

    for ph in kernel_elf
        .prog_headers
        .iter()
        .filter(|h| h.p_type == elf::Elf64_PhdrType::PT_LOAD)
    {
        info!(
            "[header] {:x} {:x} {:x} {:x} {:x}",
            ph.p_offset, ph.p_vaddr, ph.p_paddr, ph.p_filesz, ph.p_memsz
        );
        let (b, e) = ph.infile_range();
        let (b, e) = (b as usize, e as usize);
        let (b2, e2) = ph.inmem_range();
        let (b2, e2) = (
            b2 as usize - elf_range.0 as usize,
            e2 as usize - elf_range.0 as usize,
        );
        let len = usize::min(e - b, e2 - b2);
        pages[b2..(b2 + len)].copy_from_slice(&kernel_file_buf[b..b + len]);
        if e2 - b2 > e - b {
            pages[b2 + len..e2].fill(0);
        }
        info!(
            "mapping 0x{:x}:0x{:x} <- 0x{:x}:0x{:x}",
            b2 + elf_range.0 as usize,
            b2 + len + elf_range.0 as usize,
            b,
            e
        );
    }

    let entry_point = unsafe { core::mem::transmute(kernel_elf.elf_header.e_entry) };

    info!("entry addr: {:x}", entry_point as usize);

    Ok(entry_point)
}

#[entry]
fn main() -> Status {
    uefi::helpers::init().unwrap();

    info!("hello world");
    let mut dummy = 0u64;
    for i in 0..300_000_000 {
        unsafe {
            core::ptr::write_volatile(&mut dummy, i);
        }
    }
    let entry = load_kernel().expect("failed to load kernel");
    unsafe {
        let mmap = boot::exit_boot_services(MemoryType::LOADER_DATA);
        (entry)(CMemoryMap::new(&mmap));
    }
}
