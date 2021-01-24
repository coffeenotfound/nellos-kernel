#![no_std]
#![no_main]

#![feature(lang_items)]
#![feature(alloc_error_handler)]
#![feature(asm)]
#![feature(const_fn)]
#![feature(abi_efiapi)]
#![feature(naked_functions)]
#![feature(c_variadic)]

#![feature(nonnull_slice_from_raw_parts)]

// DEBUG:
#![feature(fmt_as_str)]
#![feature(panic_info_message)]
#![feature(fmt_internals)]

extern crate alloc;
extern crate core;

use core::fmt::{LowerHex, Write};
use core::ptr::NonNull;
use core::sync::atomic::AtomicUsize;
use core::sync::atomic::Ordering::*;

use fallo::FallVec;
use uefi_rs::ResultExt;

use crate::global_alloc::KernelGlobalAlloc;
use crate::uefi::boot_alloc::{self, UefiBootAlloc};

pub mod acpi;
pub mod global_alloc;
pub mod proc;
pub mod uefi;
pub mod mem;
pub mod syscall;
pub mod arch;
pub mod vga;

#[global_allocator]
static KERNEL_GLOBAL_ALLOC: KernelGlobalAlloc = KernelGlobalAlloc::new();

pub static GLOBAL_UEFI_STDOUT_PTR: AtomicUsize = AtomicUsize::new(0x0);

#[no_mangle]
#[used]
pub static LOW_HEX_FMT_FN: fn(&usize, &mut core::fmt::Formatter<'_>) -> core::fmt::Result = <usize as core::fmt::LowerHex>::fmt;

//#[deprecated(note = "Don't use! Only here to force the linker to keep other exported functions in the binary.")]
#[cfg(target_arch = "x86_64")]
#[no_mangle]
pub extern "sysv64" fn _start(_bootloader_handle_uefi: uefi_rs::Handle, sys_table_uefi: uefi_rs::prelude::SystemTable<uefi_rs::table::Boot>) -> ! {
	// DEBUG: Print
	let stdout = sys_table_uefi.stdout();
	let _ = stdout.set_color(uefi_rs::proto::console::text::Color::LightGreen, uefi_rs::proto::console::text::Color::Black).unwrap();
	let _ = stdout.write_str("[[ in kernel start ]]\n").unwrap();
	
	// Init uefi boot allocator
	boot_alloc::init_boot_alloc(sys_table_uefi.boot_services());
	
	// DEBUG:
//	(|stdout: &mut uefi_rs::proto::console::text::Output| {
//		unsafe {
//			GLOBAL_UEFI_STDOUT_PTR.store(mem::transmute(stdout), SeqCst)
//		}
//	})(stdout);
	GLOBAL_UEFI_STDOUT_PTR.store(stdout as *mut uefi_rs::proto::console::text::Output as usize, SeqCst);
	
//	let a = [65u8; 128];
//	unsafe {
//		stdout.write_str(core::str::from_utf8_unchecked(&a)).unwrap();
//		stdout.write_str("\r\n").unwrap();
//	}
//	stdout.write_str("[[ inited boot alloc ]]\n").unwrap();
//	stdout.write_str("text addr ").unwrap();
	
//	panic!("Oh noes! A panic!");
	
//	let a = LOW_HEX_FMT_FN as usize;
//	if (a as *const ()).is_null() {
//		let _ = stdout.write_str("lowerhex fmt is null");
//	} else {
//		let _ = stdout.write_str("lowerhex fmt is NOT null");
//	}
//	
//	let _ = stdout.write_char(LOW_HEX_FMT_FN as usize as u8 as char);
//	
//	stdout.write_fmt(::core::fmt::Arguments::new_v1_formatted(
//        &["", "\n"],
//        &match (&2,) {
//            (arg0,) => [::core::fmt::ArgumentV1::new(
//                arg0,
//                ::core::fmt::LowerHex::fmt,
//            )],
//        },
//        &[::core::fmt::rt::v1::Argument {
//            position: 0usize,
//            format: ::core::fmt::rt::v1::FormatSpec {
//                fill: ' ',
//                align: ::core::fmt::rt::v1::Alignment::Unknown,
//                flags: 8u32,
//                precision: ::core::fmt::rt::v1::Count::Implied,
//                width: ::core::fmt::rt::v1::Count::Is(8usize),
//            },
//        }],
//	));
	
//	writeln!(stdout, "{:08x}", 2);
//	stdout.write_str("next is fmt >>").unwrap();
//	writeln!(stdout, "{}", 3);
	
	// DEBUG:
	stdout.write_str("[[ inited boot alloc ]]\n").unwrap();
	
	// Alloc buffer for uefi memory map
	let mmap_size = sys_table_uefi.boot_services().memory_map_size();
	let mut mmap_buf = FallVec::<u8, UefiBootAlloc>::with_len_zeroed(mmap_size).unwrap();
	
	// DEBUG:
	stdout.write_str("[[ alloc'ed mmap buffer ]]\n").unwrap();
	
	// Retrieve uefi memory map
	let _ = sys_table_uefi.boot_services()
		.memory_map(mmap_buf.as_mut_slice());
	
	// DEBUG:
	stdout.write_str("[[ retrieved mmap ]]\n").unwrap();
	
	// Deinit the uefi boot allocator
	unsafe {
		boot_alloc::deinit_boot_alloc();
	}
	
	// DEBUG:
	stdout.write_str("[[ in kernel before exit boot serives ]]\n").unwrap();
	
//	// Finally exit boot services
//	let (_status, (rt_table_uefi, mmap_iter)) = sys_table_uefi.exit_boot_services(bootloader_handle_uefi, mmap_buf.as_mut_slice())
//		.unwrap().split();
//	
//	for mem_desc in mmap_iter {
//		let mem_desc: &uefi_rs::table::boot::MemoryDescriptor = mem_desc;
//		
////		stdout.write_str("[[ after boot service exit ]]").unwrap();
//	}
	
//	// DEBUG:
//	unsafe {
//		rt_table_uefi.runtime_services().reset(uefi_rs::table::runtime::ResetType::Shutdown, uefi_rs::Status::SUCCESS, None);
//  }
	
//	unsafe {
////		acpica_sys::AcpiInitializeSubsystem();
//		acpica_sys::AcpiOsCreateCache(0x0 as _, 0, 0, 0x0 as _);
//	}
	
//	// DEBUG:
//	stdout.write_str("[[ after acpica init ]]\n").unwrap();
	
//	// DEBUG:
//	unsafe {
//		vga::set_text_mode(false);
//		
//		let vga_test_buf = core::slice::from_raw_parts_mut(0xb8000 as *mut u8, 80*25);
//		vga_test_buf[0] = b'A';
//		vga_test_buf[2] = b'B';
//		vga_test_buf[3] = b'C';
//	}
	
	// DEBUG:
	stdout.write_str("[[ after tty write ]]\n").unwrap();
	
	// TODO: MSRs and CRs have to be set up for each processor
	
	unsafe {
		use arch::x86_64::msr::*;
		
		// Configure EFER MSR
		let mut efer_val = EFER.read();
		
		assert_ne!(efer_val & (0x1 << 10), 0, "Long Mode Active was 0 unexpectedly");
		assert_ne!(efer_val & (0x1 << 8), 0, "Long Mode Enable was 0 unexpectedly");
		
		efer_val |= 0x1 << 0; // Enable SCE (System Call Extensions)
		efer_val |= 0x1 << 11; // Enable NXE
		EFER.write(efer_val);
		
		// Set up x86 SYSCALL MSRs
		STAR.write(0x0); // Clear bits, legacy mode explicitely unsupported
		LSTAR.write(syscall::syscall_handler_long as u64);
		CSTAR.write(syscall::syscall_handler_compat as u64);
		SFMASK.write(0x0); // TODO: ?? No clue what flags we should mask
	}
	
	loop {}
}

#[cfg(target_arch = "aarch64")]
#[no_mangle]
pub extern "C" fn _start() {
	unimplemented!();
}

//#[cfg(target_arch = "x86_64")]
//fn start_x86_64_uefi() -> ! {
//	loop {}
//}

//#[cfg(target_arch = "aarch64")]
//fn start_aarch64_uefi() -> ! {
//	unimplemented!()
//}

#[panic_handler]
fn kernel_panic_handler(_info: &core::panic::PanicInfo) -> ! {
//	/// Flag for checking reentrant panics, i.e. when
//	/// code called from this panic handler again panics.
//	// TODO: This is a bug! panic_handler may be called from multiple threads (maybe running on different processors) so this should really use a KernelThreadLocal
//	static PANIC_REENTRANT: AtomicBool = AtomicBool::new(false);
	
	if let Some(mut ptr) = NonNull::new(GLOBAL_UEFI_STDOUT_PTR.load(SeqCst) as *mut uefi_rs::proto::console::text::Output) {
		let stdout = unsafe {ptr.as_mut()};
		let _ = stdout.write_str("Kernel panic\n");
		
//		write!(stdout, "{}", 2);
	}
	loop {}
	
//	if let Some(ptr) = NonNull::new(GLOBAL_UEFI_STDOUT_PTR.load(SeqCst) as *mut ()) {
//		let stdout = unsafe { mem::transmute::<_, &mut uefi_rs::proto::console::text::Output>(ptr) };
//		
//		let _ = stdout.set_color(uefi_rs::proto::console::text::Color::LightRed, uefi_rs::proto::console::text::Color::Black);
//		let _ = stdout.write_str("Kernel panic\n");
//		
//		if let Some(msg) = info.message() {
//			if let Some(raw_str) = msg.as_str() {
//				let _ = stdout.write_str(raw_str);
//				let _ = stdout.write_char('\n');
//			}
//		}
//		
//		if let Some(loc) = info.location() {
//			stdout.write_str(loc.file());
//		}
////		if let Some(msg) = info.message() {
////			stdout.write_fmt(*msg);
////		}
//		let _ = write!(stdout, "{}", info);
//	}
//	
//	loop {}
}

#[alloc_error_handler]
fn kernel_alloc_error_handler(_layout: core::alloc::Layout) -> ! {
	panic!("Unfallible global allocation failed (this is a bug, global allocation is forbidden)")
}

pub struct A;

impl core::fmt::Display for A {
	fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
		<usize as core::fmt::LowerHex>::fmt(&42, f)
	}
}

mod type_check {
	use prebootlib::KernelEntryFn;
	
	use crate::_start;
	
	static _TYPE_CHECK_ENTRY_FN: KernelEntryFn = _start;
}
