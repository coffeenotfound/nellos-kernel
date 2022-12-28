use core::ptr;
use core::ptr::NonNull;
use core::sync::atomic::AtomicUsize;

use atomic::Atomic;
use static_assertions::*;

use crate::mem::Phys;

pub mod ca;

pub static ACPI_ROOT_PTR: Atomic<Phys<*const cty::c_void>> = Atomic::new(Phys::new(ptr::null()));

assert_eq_size!(Option<Phys<NonNull<*const cty::c_void>>>, usize);
