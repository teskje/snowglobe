use std::alloc::{GlobalAlloc, Layout};
use std::cell::UnsafeCell;
use std::io::{self, Write};
use std::mem::MaybeUninit;
use std::ptr::{self, NonNull};
use std::sync::atomic::{AtomicBool, Ordering};
use std::{process, slice};

const FLLEN: usize = usize::BITS as usize;
const SLLEN: usize = usize::BITS as usize;

type Tlsf = rlsf::Tlsf<'static, usize, usize, FLLEN, SLLEN>;

#[global_allocator]
pub(crate) static ALLOCATOR: LockedAllocator = LockedAllocator::new();

/// Allocation-free version of `panic!`.
macro_rules! panic {
    ($($arg:tt)*) => {
        let _ = writeln!(Stderr, "panic: {}", format_args!($($arg)*));
        process::abort();
    };
}

/// Allocation-free version of `assert!`.
macro_rules! assert {
    ($expr:expr, $($arg:tt)*) => {
        if !$expr {
            panic!("assert failed: {}", format_args!($($arg)*));
        }
    };
}

/// Allocation-free version of `assert_eq!`.
#[allow(unused)]
macro_rules! assert_eq {
    ($left:expr, $right:expr) => {
        match ($left, $right) {
            (left, right) => {
                if left != right {
                    panic!("assert failed: {left:?} != {right:?}");
                }
            }
        }
    };
}

pub(crate) struct LockedAllocator(Lock<Allocator>);

impl LockedAllocator {
    const fn new() -> Self {
        Self(Lock::new(Allocator::new()))
    }

    pub fn allocate(&self, layout: Layout) -> Option<NonNull<u8>> {
        self.0.locked(|a| a.allocate(layout))
    }

    pub unsafe fn deallocate(&self, ptr: NonNull<u8>, align: usize) {
        self.0.locked(|a| unsafe { a.deallocate(ptr, align) });
    }

    pub unsafe fn reallocate(&self, ptr: NonNull<u8>, new_layout: Layout) -> Option<NonNull<u8>> {
        self.0.locked(|a| unsafe { a.reallocate(ptr, new_layout) })
    }
}

unsafe impl GlobalAlloc for LockedAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        self.allocate(layout)
            .map(NonNull::as_ptr)
            .unwrap_or(ptr::null_mut())
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        let ptr = NonNull::new(ptr).unwrap();
        let align = layout.align();
        unsafe { self.deallocate(ptr, align) };
    }

    unsafe fn realloc(&self, ptr: *mut u8, layout: Layout, new_size: usize) -> *mut u8 {
        let ptr = NonNull::new(ptr).unwrap();
        let new_layout = Layout::from_size_align(new_size, layout.align()).unwrap();
        unsafe { self.reallocate(ptr, new_layout) }
            .map(NonNull::as_ptr)
            .unwrap_or(ptr::null_mut())
    }
}

struct Allocator {
    tlsf: Option<Tlsf>,
}

impl Allocator {
    const fn new() -> Self {
        Self { tlsf: None }
    }

    fn ensure_tlsf(&mut self) -> &mut Tlsf {
        self.tlsf.get_or_insert_with(|| {
            let mut tlsf = Tlsf::new();
            let block = alloc_heap();
            tlsf.insert_free_block(block);
            tlsf
        })
    }

    fn allocate(&mut self, layout: Layout) -> Option<NonNull<u8>> {
        self.ensure_tlsf().allocate(layout)
    }

    unsafe fn deallocate(&mut self, ptr: NonNull<u8>, align: usize) {
        let tlsf = self.ensure_tlsf();
        unsafe { tlsf.deallocate(ptr, align) };
    }

    unsafe fn reallocate(&mut self, ptr: NonNull<u8>, new_layout: Layout) -> Option<NonNull<u8>> {
        let tlsf = self.ensure_tlsf();
        unsafe { tlsf.reallocate(ptr, new_layout) }
    }
}

struct Lock<T> {
    inner: UnsafeCell<T>,
    locked: AtomicBool,
}

unsafe impl<T: Send> Sync for Lock<T> {}

impl<T> Lock<T> {
    const fn new(x: T) -> Self {
        Self {
            inner: UnsafeCell::new(x),
            locked: AtomicBool::new(false),
        }
    }

    fn locked<F, R>(&self, f: F) -> R
    where
        F: FnOnce(&mut T) -> R,
    {
        let was_locked = self.locked.swap(true, Ordering::Acquire);
        assert!(!was_locked, "unsupported concurrent access");

        let result = {
            let inner = unsafe { &mut *self.inner.get() };
            f(inner)
        };

        self.locked.store(false, Ordering::Release);
        result
    }
}

const HEAP_BASE: usize = 0x1000_0000_0000;
const HEAP_SIZE: usize = 1 << 30;
const HEAP_END: usize = HEAP_BASE + HEAP_SIZE;

#[cfg(target_os = "linux")]
fn alloc_heap() -> &'static mut [MaybeUninit<u8>] {
    use libc::{MAP_ANON, MAP_FIXED_NOREPLACE, MAP_PRIVATE, PROT_READ, PROT_WRITE};

    let base = unsafe {
        libc::mmap(
            HEAP_BASE as *mut _,
            HEAP_SIZE,
            PROT_READ | PROT_WRITE,
            MAP_ANON | MAP_FIXED_NOREPLACE | MAP_PRIVATE,
            -1,
            0,
        )
    };

    if base != HEAP_BASE as *mut _ {
        panic!(
            "deterministic heap address range {:#x}..{:#x} already occupied",
            HEAP_BASE, HEAP_END,
        );
    }

    unsafe { slice::from_raw_parts_mut(base.cast(), HEAP_SIZE) }
}

#[cfg(target_os = "macos")]
fn alloc_heap() -> &'static mut [MaybeUninit<u8>] {
    use libc::{MAP_ANON, MAP_FIXED, MAP_PRIVATE, PROT_READ, PROT_WRITE};
    use mach2::kern_return::KERN_SUCCESS;
    use mach2::traps::mach_task_self;
    use mach2::vm::mach_vm_region;
    use mach2::vm_region::{VM_REGION_BASIC_INFO_64, vm_region_basic_info_64};
    use std::mem;

    // MacOS doesn't have `MAP_FIXED_NOREPLACE`, so we need to check first if the heap region is
    // already mapped.
    let mut address = HEAP_BASE as u64;
    let mut size = 0;
    let mut info = vm_region_basic_info_64::default();
    let mut count = (mem::size_of::<vm_region_basic_info_64>() / mem::size_of::<i32>()) as u32;
    let mut object_name = 0;

    let ret = unsafe {
        mach_vm_region(
            mach_task_self(),
            &mut address,
            &mut size,
            VM_REGION_BASIC_INFO_64,
            &mut info as *mut _ as *mut i32,
            &mut count,
            &mut object_name,
        )
    };

    if ret == KERN_SUCCESS && address < HEAP_END as u64 {
        panic!(
            "deterministic heap address range {:#x}..{:#x} already occupied",
            HEAP_BASE, HEAP_END,
        );
    }

    let base = unsafe {
        libc::mmap(
            HEAP_BASE as *mut _,
            HEAP_SIZE,
            PROT_READ | PROT_WRITE,
            MAP_ANON | MAP_FIXED | MAP_PRIVATE,
            -1,
            0,
        )
    };

    assert_eq!(base, HEAP_BASE as *mut _);
    unsafe { slice::from_raw_parts_mut(base.cast(), HEAP_SIZE) }
}

struct Stderr;

impl Write for Stderr {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        const FD: i32 = 2;

        let ptr = buf.as_ptr().cast();
        let n = unsafe { libc::write(FD, ptr, buf.len()) };

        if n < 0 {
            Err(io::ErrorKind::BrokenPipe.into())
        } else {
            Ok(n as usize)
        }
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}
