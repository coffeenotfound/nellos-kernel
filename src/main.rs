#![no_std]
#![no_main]

#![feature(lang_items)]
#![feature(alloc_error_handler)]
#![feature(asm)]
#![feature(const_fn)]
#![feature(abi_efiapi)]
#![feature(naked_functions)]
#![feature(c_variadic)]
#![feature(array_chunks)]
#![feature(const_option)]

#![feature(nonnull_slice_from_raw_parts)]
#![feature(maybe_uninit_uninit_array)]
#![feature(maybe_uninit_extra)]
#![feature(maybe_uninit_slice)]

// DEBUG:
#![feature(fmt_as_str)]
#![feature(panic_info_message)]
#![feature(fmt_internals)]
#![feature(core_panic)]

extern crate alloc;
extern crate core;

use core::{fmt, iter, ptr};
use core::arch::x86_64::__cpuid;
use core::fmt::{LowerHex, Write};
use core::mem::{ManuallyDrop, MaybeUninit, size_of, transmute};
use core::ptr::NonNull;
use core::sync::atomic::AtomicUsize;
use core::sync::atomic::Ordering::*;

use fallo::FallVec;
use uefi_rs::ResultExt;

use acpica_sys::{ACPI_MADT_INTERRUPT_OVERRIDE, ACPI_MADT_INTERRUPT_SOURCE, ACPI_MADT_IO_APIC, ACPI_MADT_LOCAL_APIC, ACPI_MADT_PCAT_COMPAT, ACPI_SUBTABLE_HEADER, ACPI_TABLE_DESC, ACPI_TABLE_HEADER, ACPI_TABLE_MADT, AcpiIsFailure, AcpiMadtType_ACPI_MADT_TYPE_INTERRUPT_OVERRIDE, AcpiMadtType_ACPI_MADT_TYPE_IO_APIC, AcpiMadtType_ACPI_MADT_TYPE_LOCAL_APIC};

use crate::arch::x86_64::desctable::{LongCodeDataSegmentDesc, LongIdtDesc, LongNullSegmentDesc, LongSystemSegmentDesc, PseudoDesc, SegmentSelector};
use crate::arch::x86_64::ioapic::{DeliveryMode, DestinationMode, IoApicDesc, IoApicRedTblVal, IrqPolarity, TriggerMode};
use crate::arch::x86_64::isr::{cli, sti};
use crate::arch::x86_64::msr::Msr;
use crate::global_alloc::KernelGlobalAlloc;
use crate::mem::Phys;
use crate::tty::{read_tty_char, tty_writer};
use crate::uefi::boot_alloc::{self, UefiBootAlloc};

pub mod acpi;
pub mod global_alloc;
pub mod proc;
pub mod uefi;
pub mod mem;
pub mod syscall;
pub mod arch;
pub mod vga;
pub mod tty;

#[global_allocator]
static KERNEL_GLOBAL_ALLOC: KernelGlobalAlloc = KernelGlobalAlloc::new();

pub static GLOBAL_UEFI_STDOUT_PTR: AtomicUsize = AtomicUsize::new(0x0);

#[no_mangle]
#[used]
pub static LOW_HEX_FMT_FN: fn(&usize, &mut core::fmt::Formatter<'_>) -> core::fmt::Result = <usize as core::fmt::LowerHex>::fmt;

//#[deprecated(note = "Don't use! Only here to force the linker to keep other exported functions in the binary.")]
#[cfg(target_arch = "x86_64")]
#[no_mangle]
pub extern "sysv64" fn _start(bootloader_handle_uefi: uefi_rs::Handle, sys_table_uefi: uefi_rs::prelude::SystemTable<uefi_rs::table::Boot>) -> ! {
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
//		let _ = stdout.write_str("lowerhex fmt is null\n");
//	} else {
//		let _ = stdout.write_str("lowerhex fmt is NOT null\n");
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
	let mut mmap_buf = FallVec::<u8, UefiBootAlloc>::with_len_zeroed(mmap_size + 128).unwrap();
	
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
	
	// Drop uefi resources before transitioning
	let stdout = ();
	
	// Finally exit boot services
	let (_status, (rt_table_uefi, mmap_iter)) = sys_table_uefi.exit_boot_services(bootloader_handle_uefi, mmap_buf.as_mut_slice())
		.unwrap().split();
	
//	for mem_desc in mmap_iter {
//		let mem_desc: &uefi_rs::table::boot::MemoryDescriptor = mem_desc;
//		
////		stdout.write_str("[[ after boot service exit ]]").unwrap();
//	}
	
//	// DEBUG:
//	unsafe {
//		rt_table_uefi.runtime_services().reset(uefi_rs::table::runtime::ResetType::Shutdown, uefi_rs::Status::SUCCESS, None);
//  }
	
//	for _ in 0..(0x1<<22) {}
	
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
	unsafe {
		tty::enable_serial_tty();
		
		/*
		for i in 0..8 {
			tty::write_tty(b"\x1b[35m\x1b[40m");
			tty::write_tty(b"\x1b[3");
			tty::write_tty_char(b'0' + (i % 8usize) as u8);
			tty::write_tty_char(b'm');
			
			tty::write_tty(b"ABC");
		}
		
		tty::write_tty(b"\x1b[20Cforward!");
		tty::write_tty(b"\x1b[2Aup!");
		tty::write_tty(b"\x1b[16Dback!");
		tty::write_tty(b"\x1b[8S");
		tty::write_tty(b"\x1b[4Bdown!\r\n");
		
		tty::write_tty(b"\x1b[0m");
		for y in 0..10 {
			for i in 0..10 {
				tty::write_tty(b"\x1b[");
				if y > 0 {
					tty::write_tty_char(b'0' + y as u8);
				}
				tty::write_tty_char(b'0' + i as u8);
				tty::write_tty_char(b'm');
				
				tty::write_tty(b"Test");
				if y > 0 {
					tty::write_tty_char(b'0' + y as u8);
				}
				tty::write_tty_char(b'0' + i as u8);
				
				tty::write_tty(b"\x1b[0m");
				tty::write_tty_char(b' ');
			}
			tty::write_tty(b"\r\n");
			
			tty::write_tty(b"\x1b[T");
		}	
//		tty::write_tty(b"\x1b[?1049h");
		tty::write_tty(b"\x1b[1S");
		tty::write_tty(b"\x1b[1E");
		
		tty::write_tty_nl_only();
		*/
	}
	
	// Log
	writeln!(tty_writer(), "After ExitBootServices");
	
	unsafe {
		for entry in rt_table_uefi.config_table() {
//			if entry.guid == RawUefiGuid::new(0x8868e871, 0xe4f1, 0x11d3, [0xbc,0x22,0x00,0x80,0xc7,0x3c,0x88,0x81]).into_uefi_rs() {
			if entry.guid == uefi_rs::table::cfg::ACPI2_GUID {
				let acpi_root_ptr = Phys::new(entry.address as *const cty::c_void);
				
				acpi::ACPI_ROOT_PTR.store(acpi_root_ptr, SeqCst);
			}
		}
	}
	
	// Do early acpica table manager initialization
	// This is needed to do our early kernel init and get
	// virtual memory et al. running.
	// We later do the full acpica initialization
	unsafe {
		let mut tables = [MaybeUninit::<ACPI_TABLE_DESC>::zeroed(); 16];
		
		let s = acpica_sys::AcpiInitializeTables(tables.as_mut_ptr() as _, tables.len() as _, acpica_sys::TRUE);
		if AcpiIsFailure(s) {
			panic!("Failed to do early acpica table initialization: {}", s);
		}
		
		// DEBUG: Log found table signatures
		for tab in &tables {
			let tab = &*tab.as_ptr();
			
			if tab.Signature.Integer != 0 {
				let sig_bytes = tab.Signature.Integer.to_le_bytes();
				let sig_str = core::str::from_utf8(&sig_bytes).unwrap_or("[XX]");
				
				writeln!(tty_writer(), "acpi table: {}", sig_str);
			}
		}
	}
	
	// Query MADT info
	let has_8259_pics: bool;
	let mut first_io_apic = MaybeUninit::<IoApicDesc>::zeroed();
	let mut io_apic_order = 0;
	unsafe {
		// Get MADT table ptr
		let mut madt_sig = *b"APIC";
		let mut table_hdr: *mut ACPI_TABLE_HEADER = ptr::null_mut();
		assert_eq!(acpica_sys::AcpiGetTable(madt_sig.as_mut_ptr() as _, 1, &mut table_hdr as _), 0);
		
		let madt = &*(table_hdr as *const ACPI_TABLE_MADT);
		
		assert!(&madt.Header.Signature == transmute::<_, &[i8; 4]>(b"APIC"));
		
		has_8259_pics = (madt.Flags & ACPI_MADT_PCAT_COMPAT) != 0;
		
		writeln!(tty_writer(), "madt lapic base addr: {:08x}", madt.Address);
		writeln!(tty_writer(), "madt has 8259PICs: {}", has_8259_pics);
		
		let mut sub_ptr = (madt as *const _ as usize + 44) as *const ACPI_SUBTABLE_HEADER;
		
		while (sub_ptr as usize + size_of::<ACPI_SUBTABLE_HEADER>()) <= (madt as *const _ as usize + madt.Header.Length as usize) {
			let sub = &*sub_ptr;
			
			writeln!(tty_writer(), "madt entry: {}", sub.Type);
			
			if sub.Type as u32 == AcpiMadtType_ACPI_MADT_TYPE_LOCAL_APIC {
				let lapic_tab = &*(sub_ptr as *const ACPI_MADT_LOCAL_APIC);
				
				writeln!(tty_writer(), "  processor _UID {} -> local apic id {}", lapic_tab.ProcessorId, lapic_tab.Id);
			}
			
			if sub.Type as u32 == AcpiMadtType_ACPI_MADT_TYPE_INTERRUPT_OVERRIDE {
				let irq_override_tab = &*(sub_ptr as *const ACPI_MADT_INTERRUPT_OVERRIDE);
				writeln!(tty_writer(), "  source override: ISA #{} -> IOAPIC (GSI) #{}", irq_override_tab.SourceIrq, irq_override_tab.GlobalIrq);
			}
			
			if sub.Type as u32 == AcpiMadtType_ACPI_MADT_TYPE_IO_APIC {
				let io_apic_tab = &*(sub_ptr as *const ACPI_MADT_IO_APIC);
				
				if io_apic_order == 0 {
					first_io_apic.write(IoApicDesc {
						order: io_apic_order,
						id: io_apic_tab.Id,
						regs: Phys(io_apic_tab.Address as usize as *mut u128),
						base_gsi: io_apic_tab.GlobalIrqBase,
					});
				}
				
				io_apic_order += 1;
			}
			
			sub_ptr = sub_ptr.byte_offset(sub.Length as _);
		}
//		let first_io_apic = first_io_apic.assume_init();
		
//		// DEBUG:
//		wait_here();
	}
	
//	// Do full acpica initialization
//	unsafe {
//		// Init acpica subsystem
//		let s = acpica_sys::AcpiInitializeSubsystem();
//		if acpica_sys::AcpiIsFailure(s) {
//			panic!("Failed to init acpica subsystem: {}", s);
//		}
//		
//		// Init acpica tables
//		acpica_sys::AcpiLoadTables();
//		
//		// Load acpi tables
//		acpica_sys
//	}
	
//	// DEBUG:
//	stdout.write_str("[[ after tty write ]]\n").unwrap();
	
	/*
	// DEBUG:
	unsafe {
		let width = 80;
		let height = 24;
		
//		tty::write_tty(b"\x1b[?3h");
//		tty::write_tty(b"\x1b[12l");
		tty::write_tty_char(b'+');
		for x in 0..width-2 {
			tty::write_tty_char(b'-');
		}
		tty::write_tty_char(b'+');
		
		for y in 0..height-2 {
			tty::write_tty_char(b'|');
			
			for x in 0..width-2 {
				tty::write_tty_char(b' ');	
			}
			
//			tty::write_tty(b"\x1b[");
//			let mut num_buf = [0u8; 4];
//			tty::write_tty(format_unsigned(width-1, 10, &mut num_buf));
//			tty::write_tty(b"C");
			
			tty::write_tty_char(b'|');
		}
		
		tty::write_tty_char(b'+');
		for x in 0..width-2 {
			tty::write_tty_char(b'-');
		}
		tty::write_tty_char(b'+');
		
		tty::write_tty(b"\x1b[");
	}
	*/
	
	// Disable interrupts
	unsafe {cli();}
	
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
		STAR.write(0x0); // Clear bits, protected mode explicitely unsupported
		LSTAR.write(syscall::syscall_handler_long as u64);
		CSTAR.write(syscall::syscall_handler_compat as u64);
		SFMASK.write(0x0); // TODO: ?? No clue what flags we should mask
		
//		#[repr(C)]
//		struct GdtrPseudoDesc {
//			_pad: [u16; 3],
//			limit: u16,
//			base: u64,
//		}
		
//		static GDTR_VAL: GdtrPseudoDesc = GdtrPseudoDesc {_pad: [0; 3], limit: 1*core::mem::size_of::<u64>() as u16, base: 0x0};
		
		/*
		{// DEBUG: Dump ovmf gdt and idt
			// Dump all gdt entries
			let mut uefi_gdt_desc = PseudoDesc {limit: 0, base: 0};
			asm!("sgdt [{}]", in(reg) &mut uefi_gdt_desc, options(nostack));
			
			for off in (0..=uefi_gdt_desc.limit as usize).step_by(8) {
				let seg_desc = &*((uefi_gdt_desc.base as usize + off) as *const u64);
				writeln!(tty_writer(), "dump desc: {:x}", seg_desc);
			}
			
			// Dump idt entry
			let mut uefi_idt_desc = PseudoDesc {limit: 0, base: 0};
			asm!("sidt [{}]", in(reg) &mut uefi_idt_desc, options(nostack));
			
			writeln!(tty_writer(), "dump #bp idte: +0: {:x}, +8: {:x}", *((uefi_idt_desc.base as usize + 3*16) as *const u64), *((uefi_idt_desc.base as usize + 3*16+8) as *const u64));
		}
		*/
		
		static mut GDT_BUF: [MaybeUninit<u64>; 7] = MaybeUninit::uninit_array();
		static mut IDT_BUF: [MaybeUninit<LongIdtDesc>; 256] = MaybeUninit::uninit_array();
		static mut TSS_BUF: [u32; 68] = [0; 68];
		
		{// Set up the TSS
			// Reserved, IGN
			TSS_BUF[0] = 0x0;
			TSS_BUF[7] = 0x0;
			TSS_BUF[22] = 0x0;
			TSS_BUF[23] = 0x0;
			
			// Zero out rsp1/rsp2
			TSS_BUF[3] = 0x0;
			TSS_BUF[4] = 0x0;
			TSS_BUF[5] = 0x0;
			TSS_BUF[6] = 0x0;
			
			// Zero out IST entries
			// TODO: Maybe make use of ISTs in the future
			for ist in TSS_BUF[8..22].array_chunks_mut::<2>() {
				*ist = [0x0, 0x0];
			}
			
			// Disable IO Permission Bitmap by setting the IOPB address offset
			// to u16::MAX, causing it to *definitely* fall outside the TSS segment limit
			// (https://stackoverflow.com/questions/54876039/creating-a-proper-task-state-segment-tss-structure-with-and-without-an-io-bitm/54876040#54876040)
			TSS_BUF[24] = (0xffff_ffff << 16) | 0x0000_0000;
		}
		
		// Construct GDT
		let gdt_ptr = GDT_BUF.as_mut_ptr();
		(gdt_ptr as *mut LongNullSegmentDesc).write(LongNullSegmentDesc::new());
		(gdt_ptr.offset(1) as *mut LongCodeDataSegmentDesc).write(LongCodeDataSegmentDesc::new_code(0, 1, 1, 0x0, 0)); // Kernel Code Segment
		(gdt_ptr.offset(2) as *mut LongCodeDataSegmentDesc).write(LongCodeDataSegmentDesc::new_data(1, 0x0)); // Kernel Data Segment
		(gdt_ptr.offset(3) as *mut LongCodeDataSegmentDesc).write(LongCodeDataSegmentDesc::new_code(0, 1, 1, 0x3, 1)); // Usermode Code Segment
		(gdt_ptr.offset(4) as *mut LongCodeDataSegmentDesc).write(LongCodeDataSegmentDesc::new_data(1, 0x3)); // Usermode Data Segment
		// TODO: Figure out what RPL the TSS descriptor should have
		(gdt_ptr.offset(5) as *mut LongSystemSegmentDesc).write(LongSystemSegmentDesc::new(TSS_BUF.as_ptr() as u64, core::alloc::Layout::for_value(&TSS_BUF).size().saturating_sub(1) as u32, 0b0, 0b0, 0b1, 0x0, 0xb)); // Long mode TSS
		
		// Construct IDT
		let gdt_code_selector = SegmentSelector::new(0x0, false, 1);
		for i in 0..256 {
			let isr: u64 = match i {
				0x03 => isr_bp as u64,
				0x08 => isr_df as u64,
				0x0c => isr_ss as u64,
				0x0d => isr_gp as u64,
				0x42 => isr_serial_com13 as u64,
				_ => isr_any as u64,
			};
			IDT_BUF[i].write(LongIdtDesc::new(isr, gdt_code_selector, 0, 0b1110, 0x0, 1));
		}
		
		// TODO: Ensure no interrupts occur before or while configuring the GDT, IDT, APIC, etc. else it will probably crash
		
		// Set up GDT
		let gdt_desc = PseudoDesc {
			base: &GDT_BUF as *const _ as u64,
//			limit: core::mem::size_of_val(&GDT_BUF).saturating_sub(1) as u16,
			limit: core::alloc::Layout::for_value(&GDT_BUF).size().saturating_sub(1) as u16,
		};
		asm!("lgdt [{}]", in(reg) &gdt_desc, options(nostack));
		
		// Set up IDT
		let idt_desc = PseudoDesc {
//			base: IDT_BUF.as_ptr() as u64,
			base: &IDT_BUF as *const _ as u64,
			limit: core::alloc::Layout::for_value(&IDT_BUF).size().saturating_sub(1) as u16,
		};
		asm!("lidt [{}]", in(reg) &idt_desc, options(nostack));
		
		// DEBUG:
		writeln!(tty_writer(), "gdt desc: {:x?}", gdt_desc);
		writeln!(tty_writer(), "idt desc: {:x?}", idt_desc);
		writeln!(tty_writer(), "gdt dump: {:x?}", MaybeUninit::slice_assume_init_ref(&GDT_BUF));
		writeln!(tty_writer(), "idt dump: {:x?}", MaybeUninit::slice_assume_init_ref(&IDT_BUF[0..4]));
		
		// DEBUG: Dump gdt entries
		for (i, off) in (0..=gdt_desc.limit as usize).step_by(8).enumerate() {
			let seg_desc = &*((gdt_desc.base as usize + off) as *const u64);
			writeln!(tty_writer(), "dump OUR gdt desc #{:02}: {:x}", i, seg_desc);
		}
		
		// DEBUG: Dump idt entry
		writeln!(tty_writer(), "dump OUR #bp idte: +0: {:x}, +8: {:x}", *((idt_desc.base as usize + 3*16) as *const u64), *((idt_desc.base as usize + 3*16+8) as *const u64));
		
		// Set up segment registers
		writeln!(tty_writer(), "changing segments");
		
		asm!(
			"mov ds, ax",
			"mov es, ax",
			"mov fs, ax",
			"mov gs, ax",
			in("ax") 0x0,
		);
		
		// Load null selector into ss reg
		// I thought the ss needs to be a valid selector but turns
		// out long mode doesn't care, in fact inter-privelege-level
		// calls load null ss selectors anyways.
		asm!(
			"mov ss, ax",
//			in("ax") SegmentSelector::new(0x0, false, 2).into_raw(),
			in("ax") 0x0,
		);
		
		let mut prev_cs: u16;
		asm!("mov {}, cs", out(reg) prev_cs);
		writeln!(tty_writer(), "prev CS {:04x}h", prev_cs);
		
		writeln!(tty_writer(), "changing cs segment");
		
		// Set up cs segment register
		let mut test_sel: usize = 0;
		let mut test_label: usize = 0;
		let mut temp_rip: usize = 0;
		asm!(
			"sub rsp, 16",
			"mov qword ptr [rsp+8], 0",
			"mov word ptr [rsp+8], {sel:x}",
			"mov qword ptr [rsp], offset here",
			
			// DEBUG:
			"lea {temp_rip}, [rip]",
			
			// NOTE: JMP m16:64 may not be fully portable (AFAIK some early AMDs didn't have it)
			//  The prefered way would be a retf but I couldn't for the life of me figure that out
			
			// This `ljmp tbyte ptr [rsp]` has been verified with a
			// disassembler to really result in the correct `rex.w jmp m16:64`.
			"ljmp tbyte ptr [rsp]",
			"here:",
			"add rsp, 16",
			
			sel = in(reg) SegmentSelector::new(0x0, false, 1).into_raw(),
			
			// DEBUG:
//			test_sel = out(reg) test_sel,
//			test_label = out(reg) test_label,
			temp_rip = out(reg) temp_rip,
		);
//		writeln!(tty_writer(), "{:016x}, {:04x} ({:016x})", test_label, test_sel as u16, test_sel);
		writeln!(tty_writer(), "rip {:x}, code dump {:x?}", temp_rip, core::slice::from_raw_parts(temp_rip as *const u8, 64));
		
		let mut new_cs: u16;
		asm!("mov {}, cs", out(reg) new_cs);
		writeln!(tty_writer(), "new CS {:04x}h", new_cs);
	}
	
	// Disable pic
	unsafe {
		// Actually this is probably already done by the uefi firmware
	}
	
	// TODO: lapics need to be configured on each cpu itself
	//  But do we send IPIs thought without having configured interrupts??
	// Configure processor local lapic
	unsafe {
		use arch::x86_64::msr::*;
		
		// Note that only x2APIC is configured through MSRs,
		// APIC is done through a 4 KiB mmio region starting
		// at the address specified by the IA32_APIC_BASE MSR
		// Right now we just assume x2APIC because I'm lazy
		
		// Note that x2APIC mode is disabled by default
		// and right now we don't enable it, so we only use
		// normal APIC mode
		
		let apic_base_msr = Msr::from_nr(0x0000_001b);
		let apic_base_msr_val = apic_base_msr.read();
		
		let lapic_base = apic_base_msr_val & 0xf_ffff_ffff_f000;
		
		// DEBUG: Check x2APIC support
		let feature_cpuid = __cpuid(1);
		
		writeln!(tty_writer(), "Supports x2APIC: {}", feature_cpuid.ecx & (0x1 << 21) != 0);
		
		// Log
		writeln!(tty_writer(), "apic_base_msr = {:08x} (ABA {:x}, AE {:b}, BSC {:b})",
			apic_base_msr_val,
			lapic_base,
			(apic_base_msr_val >> 11) & 0b1,
			(apic_base_msr_val >> 8) & 0b1,
		);
		
		writeln!(tty_writer(), "lapic id = 0x{:x}, lapic ver = 0x{:x}", ((lapic_base+0x20) as *const u32).read_volatile(), ((lapic_base+0x30) as *const u32).read_volatile());
		
		// Enable lapic
		// DEBUG:
		writeln!(tty_writer(), "spurious reg = 0x{:0x}", ((lapic_base+0xf0) as *const u32).read_volatile());
		
		let spurious_isr_nr: u8 = 0xff; // Map spurious apic isr to #255
		((lapic_base+0xf0) as *mut u32).write_volatile(0x100 | (spurious_isr_nr & 0xff) as u32);
	}
	
	// Configure ioapic(s)
	unsafe {
		let io_apic = first_io_apic.assume_init();
		
		// TODO: use set_full_dest()
		// https://wiki.osdev.org/IOAPIC#IOREDTBL
		let mut com_entry = IoApicRedTblVal(0);
		com_entry.set_dest_field(0); // physical dest: apic 0
		com_entry.set_interrupt_mask(false);
		com_entry.set_trigger_mode(TriggerMode::EdgeSensitive);
		com_entry.set_polarity(IrqPolarity::ActiveHigh);
		com_entry.set_dest_mode(DestinationMode::Physical);
		com_entry.set_delv_mode(DeliveryMode::Fixed);
		com_entry.set_irq_vector(0x42);
		
		io_apic.write_redir(4, com_entry);
//		io_apic.write_redir(3, com_entry);
	}
	
	// Reenable interrupts
	unsafe {sti();}
	
	// DEBUG: Test sending local isr
	unsafe {
		writeln!(tty_writer(), "requesting interrupt");
		asm!("int3");
		writeln!(tty_writer(), "after interrupt");
	}
	
	unsafe {
		// DEBUG: Jump to usermode
	}
	
	loop {}
}

//#[naked]
//pub unsafe extern "C" fn dummy_isr() {
////	asm!("iret", options(noreturn));
//	
////	asm!(
////		"LOOP:",
////		"pause",
////		"jmp LOOP",
////		options(noreturn),
////	);
//	asm!(
//		"call {target}",
//		"iret",
//		target = sym dummy_isr_cooked,
//		options(noreturn),
//	);
//}
//
//pub unsafe extern "sysv64" fn dummy_isr_cooked(_it: u32) {
//	let apic_base_msr = Msr::from_nr(0x0000_001b);
//	let apic_base_msr_val = apic_base_msr.read();
//	
//	let lapic_base = apic_base_msr_val & 0xf_ffff_ffff_f000;
//	
//	// Send EOI to lapic
//	((lapic_base+0xb0) as *mut u32).write_volatile(0x00);
//	
//	tty::write_tty_ln(b"IN ISR");
////	asm!(
////		"div {zero}",
////		zero = in(reg) 0,
////	);
////	for _ in 0..usize::MAX {}
//}

macro_rules! isr_entry {
	($entry:ident => $handler_call:expr; $ec:tt) => {
		#[naked]
		pub unsafe extern "C" fn $entry() {
			asm!(
				// Save sysv64 caller-saved registers
				"sub rsp, 72",
				"mov [rsp], rax",
				"mov [rsp+8], rcx",
				"mov [rsp+16], rdx",
				"mov [rsp+24], rsi",
				"mov [rsp+32], rdi",
				"mov [rsp+40], r8",
				"mov [rsp+48], r9",
				"mov [rsp+56], r10",
				"mov [rsp+64], r11",
				
				// Call handler
				"call {inner}",
				
				// Restore saved registers
				"mov rax, [rsp]",
				"mov rcx, [rsp+8]",
				"mov rdx, [rsp+16]",
				"mov rsi, [rsp+24]",
				"mov rdi, [rsp+32]",
				"mov r8, [rsp+40]",
				"mov r9, [rsp+48]",
				"mov r10, [rsp+56]",
				"mov r11, [rsp+64]",
				"add rsp, 72",
				isr_entry!(@poperrcode; $ec),
				
				"iretq",
				
				inner = sym _inner,
				options(noreturn),
			);
			
			unsafe extern "sysv64" fn _inner() {
				// Signal EOI to lapic
				let lapic_base = Msr::from_nr(0x0000_001b).read() & 0xf_ffff_ffff_f000;
				((lapic_base+0xb0) as *mut u32).write_volatile(0x00);
				
//				let _ = writeln!(tty_writer(), "> IN ISR: {}", $id);
				if $ec {
					let errcode: usize;
					let rip: usize;
					asm!(
						"pop {temp:r}", // Pop frame return rip
						"pop {errcode:r}", // Pop error code
						"push {temp:r}", // Re-push frame retur rip
						
						errcode = out(reg) errcode,
						temp = out(reg) _,
					);
					asm!("mov {:r}, qword ptr [rsp]", out(reg) rip);
					
//					let _ = writeln!(tty_writer(), "errcode {:x}, rip {:08x}", errcode, rip);
				}
				
				// Call handler
				$handler_call;
				
//				for _ in 0..(0x1<<20) {}
			}
		}
	};
	(@poperrcode; true) => ("add rsp, 8");
	(@poperrcode; false) => ("");
}

isr_entry!(isr_any => echo_handler("#any"); false);
isr_entry!(isr_bp => echo_handler("#bp"); false);
isr_entry!(isr_df => echo_handler("#df"); false);
isr_entry!(isr_ss => echo_handler("#ss"); false);
isr_entry!(isr_gp => echo_handler("#gp"); true);

isr_entry!(isr_serial_com13 => serial_com13_handler(); false);

fn echo_handler(name: &'_ str) {
	let _ = writeln!(tty_writer(), "> IN ISR: {}", name);
}

fn serial_com13_handler() {
//	let mut buf = [0u8; 4];
//	
//	buf.iter_mut()
//		.zip(iter::from_fn(|| unsafe {read_tty_char()}))
//		.for_each(|(slot, char)| *slot = char);
//	
//	let _ = writeln!(tty_writer(), "tty >> {}", core::str::from_utf8(&buf).unwrap());
//	let _ = writeln!(tty_writer(), "tty >> {:?}", &buf);
	
	while let Some(c) = unsafe {read_tty_char()} {
//		let _ = writeln!(tty_writer(), "tty >> {:02X}h", c);
//		let _ = write!(tty_writer(), "{}", char::from_u32(c).unwrap());
		let _ = tty_writer().write_char(char::from_u32(c as u32).unwrap());
	}
	
	let iir = unsafe {crate::arch::x86_64::port::inb(0x3F8 + 2)};
	
//	let _ = writeln!(tty_writer(), "tty status {:08b}", iir);
//	for _ in 0..(0x1<<20) {}
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
#[no_mangle]
fn kernel_panic_handler(info: &core::panic::PanicInfo) -> ! {
//	/// Flag for checking reentrant panics, i.e. when
//	/// code called from this panic handler again panics.
//	// TODO: This is a bug! panic_handler may be called from multiple threads (maybe running on different processors) so this should really use a KernelThreadLocal
//	static PANIC_REENTRANT: AtomicBool = AtomicBool::new(false);
	
	unsafe {
		tty::write_tty(b"\x1b[31;1m");
		
		tty::write_tty_nl_only();
		tty::write_tty_ln(b"Kernel panic!");
		
		if let Some(loc) = info.location() {
			tty::write_tty(b"At ");
			tty::write_tty(loc.file().as_bytes());
			
			tty::write_tty(b"(");
			
			let mut line_buf = [0u8; 8];
			tty::write_tty(format_unsigned(loc.line() as usize, 10, &mut line_buf));
			tty::write_tty_char(b':');
			let mut col_buf = [0u8; 8];
			tty::write_tty(format_unsigned(loc.column() as usize, 10, &mut col_buf));
			
			tty::write_tty_ln(b")");
		}
		
		if let Some(msg) = info.message() {
			writeln!(tty_writer(), "{}", msg);
//			write!(&mut msg_buf, "{}", msg);
//			tty::write_tty_ln(&msg_buf.buf[0..msg_buf.len]);
		}
		
//		if let Some(msg) = info.message() {
//			if let Some(trivial_msg) = msg.as_str() {
//				tty::write_tty_ln(trivial_msg.as_bytes());				
//			}
//		}
	}
	
//	if let Some(mut ptr) = NonNull::new(GLOBAL_UEFI_STDOUT_PTR.load(SeqCst) as *mut uefi_rs::proto::console::text::Output) {
//		let stdout = unsafe {ptr.as_mut()};
////		let _ = stdout.write_str("Kernel panic\n");
//		
////		write!(stdout, "{}", 2);
//	}
	
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
	
	unsafe {
		loop {
			asm!("hlt", options(nomem, nostack))
		}
	}
}

fn format_unsigned<'a>(num: usize, radix: usize, buf: &'a mut [u8]) -> &'a mut [u8] {
	if radix <= 1 || radix > 16 {
		panic!("Illegal radix {}", radix);
	}
	
	let mut cursor = buf.len() - 1;
	let mut rest = num;
	loop {
		let digit = rest % radix;
		let digit_char = match digit {
			i @ 0..=9 => b'0' + i as u8,
			i @ 10..=15 => b'A' + (i - 10) as u8,
			_ => unreachable!(),
		};
		
		if let Some(slot) = buf.get_mut(cursor) {
			*slot = digit_char;
			cursor -= 1;
		} else {
			// Write overflow char
			if let Some(first) = buf.get_mut(0) {
				*first = b'^';
			}
			break;
		}
		
		rest /= radix;
		
		// Do this here instead of using a while loop
		// so that if num is 0 we still emit one digit (zero)
		// instead of doing nothing
		if rest <= 0 {
			break;
		}
	}
	
	&mut buf[cursor+1..]
}

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub enum Radix {
	Binary,
	Octal,
	Decimal,
	Hex,
}

//impl Radix {
//	pub fn base(&self) -> usize {
//		use Self::*;
//		match self {
//			Binary => 2,
//			Octal => 8,
//			Decimal => 10,
//			Hex => 16,
//		}
//	}
//	
//	pub fn from_base(base: usize) -> Option<Radix> {
//		
//	}
//}

pub fn wait_here() {
	loop {
		unsafe {
			asm!("hlt");
		}
	}
}

pub trait PtrOpsExt {
	unsafe fn byte_offset(self, offset: isize) -> Self;
}
impl<T> PtrOpsExt for *const T {
	unsafe fn byte_offset(self, offset: isize) -> Self {
		self.cast::<u8>()
			.offset(offset)
			.cast()
	}
}
impl<T> PtrOpsExt for *mut T {
	unsafe fn byte_offset(self, offset: isize) -> Self {
		self.cast::<u8>()
			.offset(offset)
			.cast()
	}
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

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
#[repr(C)]
pub struct RawUefiGuid {
	a: u32,
	b: u16,
	c: u16,
	d: [u8; 8],
}

impl RawUefiGuid {
	pub const fn new(a: u32, b: u16, c: u16, d: [u8; 8]) -> Self {
		Self {a, b, c, d}
	}
	
	pub fn into_uefi_rs(self) -> uefi_rs::Guid {
		unsafe {core::mem::transmute(self)}
	}
}
