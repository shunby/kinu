use core::{
    arch::asm, sync::atomic::{AtomicBool, AtomicU64, Ordering}
};

use lock_api::{GuardNoSend, GuardSend, RawMutex, RawRwLock};


pub struct SpinMutex {
    locked: AtomicBool,
}

unsafe impl RawMutex for SpinMutex {
    #[allow(clippy::declare_interior_mutable_const)]
    const INIT: Self = Self {
        locked: AtomicBool::new(false),
    };
    type GuardMarker = GuardSend;
    fn lock(&self) {
        while self.locked.swap(true, Ordering::AcqRel) {
            unsafe {asm!("hlt");}
        }
    }

    fn try_lock(&self) -> bool {
        !self.locked.swap(true, Ordering::Acquire)
    }

    unsafe fn unlock(&self) {
        self.locked.store(false, Ordering::Release);
    }
}

pub type Mutex<T> = lock_api::Mutex<SpinMutex, T>;

pub struct SpinRwLock {
    /// 0 when unlocked
    /// 2n when n readers exist
    /// 1 when a writer exists
    count: AtomicU64
}

unsafe impl RawRwLock for SpinRwLock {
    #[allow(clippy::declare_interior_mutable_const)]
    const INIT: Self = Self {
        count: AtomicU64::new(0)
    };
    type GuardMarker = GuardNoSend;
    fn lock_exclusive(&self) {
        loop {
            while self.count.load(Ordering::Relaxed) != 0 {unsafe {asm!("hlt");}}
            if self.count.compare_exchange_weak(0, 1, Ordering::Acquire, Ordering::Relaxed).is_ok() {
                break;
            }
        }
    }

    fn lock_shared(&self) {
        loop {
            let old = self.count.load(Ordering::Relaxed);
            if old != 1 && self.count.compare_exchange_weak(old, old+2, Ordering::Acquire, Ordering::Relaxed).is_ok() {
                break;
            }
        }
    }

    fn try_lock_exclusive(&self) -> bool {
        self.count.compare_exchange_weak(0, 1, Ordering::Acquire, Ordering::Relaxed).is_ok()
    }

    fn try_lock_shared(&self) -> bool {
        let old = self.count.load(Ordering::Relaxed);
        old != 1 && self.count.compare_exchange_weak(old, old+2, Ordering::Acquire, Ordering::Relaxed).is_ok()
    }

    unsafe fn unlock_exclusive(&self) {
        self.count.store(0, Ordering::Release)
    }

    unsafe fn unlock_shared(&self) {
        self.count.fetch_sub(2, Ordering::Relaxed);
    }
}

pub type RwLock<T> = lock_api::RwLock<SpinRwLock, T>;
