use core::arch::asm;
use core::fmt::Write;
use core::mem::MaybeUninit;

use crate::{LongIdtDesc, PseudoDesc, SegmentSel, SegmentSelTI, tty, tty_writer};
use crate::arch::x86_64::desctable::LongSegmentDescType;

pub static mut IDT_BUF: [LongIdtDesc; 256] = [LongIdtDesc::null(); 256];
pub static mut TSS_BUF: [u32; 68] = [0; 68];

pub unsafe fn setup_idt() {
	// Construct the IDT
	let gdt_cs_sel = SegmentSel::new(0, SegmentSelTI::GDT, 1);
	for i in 0..256 {
		let isr_entry_ptr: u64 = match i {
			0x03 => isr_bp as u64,
			0x08 => isr_df as u64,
			0x0c => isr_ss as u64,
			0x0d => isr_gp as u64,
			0x42 => isr_serial_com13 as u64,
			_ => isr_other as u64,
		};
		let desc = LongIdtDesc::new(isr_entry_ptr, gdt_cs_sel, 0, LongSegmentDescType::InterruptGate64, 0x0, true);
		IDT_BUF[i] = desc;
	}
	
	// Set up the IDT
	let idt_desc = PseudoDesc {
		base: IDT_BUF.as_ptr() as u64,
		limit: core::alloc::Layout::for_value(&IDT_BUF).size().saturating_sub(1) as u16,
	};
	asm!("lidt [{}]", in(reg) &idt_desc, options(nostack));
	
//	// DEBUG:
//	let _ = writeln!(tty_writer(), "gdt desc: {:x?}", gdt_desc);
//	let _ = writeln!(tty_writer(), "idt desc: {:x?}", idt_desc);
//	let _ = writeln!(tty_writer(), "gdt dump: {:x?}", &IDT_BUF);
//	let _ = writeln!(tty_writer(), "idt dump: {:x?}", MaybeUninit::slice_assume_init_ref(&IDT_BUF[0..4]));
	
	// DEBUG: Dump idt entry
	let _ = writeln!(tty_writer(), "dump OUR #bp idte: +0: {:x}, +8: {:x}", *((idt_desc.base as usize + 3*16) as *const u64), *((idt_desc.base as usize + 3*16+8) as *const u64));
}

pub unsafe fn setup_interrupt_tss() {
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

#[inline(always)]
pub unsafe fn cli() {
	asm!("cli", options(nostack));
}

#[inline(always)]
pub unsafe fn sti() {
	asm!("sti", options(nostack));
}

macro_rules! isr_entry {
	($entry:ident => $handler_call:expr; $ec:tt) => {
		#[naked]
		pub unsafe extern "sysv64" fn $entry() {
			asm!(
				// Save sysv64 caller-saved registers
				// Other regs will be saved implicitely by the inner func itself
				// thanks to it's abi and us calling it instead of jmping to it
				// (Using push instead of sub rsp + mov is better for performance
				//  on modern (last 10 years) processors.
				"push rax",
				"push rcx",
				"push rdx",
				"push rsi",
				"push rdi",
				"push  r8",
				"push  r9",
				"push r10",
				"push r11",
				
				// Call handler
				"call {inner}",
				
				// Restore saved registers
				"pop rax",
				"pop rcx",
				"pop rdx",
				"pop rsi",
				"pop rdi",
				"pop  r8",
				"pop  r9",
				"pop r10",
				"pop r11",
				
				isr_entry!(@poperrcode; $ec),
				
				"iretq",
				
				inner = sym _inner,
				options(noreturn),
			);
			
			unsafe extern "sysv64" fn _inner() {
				use crate::arch::x86_64::msr::Msr;
				
				// Signal EOI to lapic
				let lapic_base = Msr::<u64>::from_nr(0x0000_001b).read() & 0xf_ffff_ffff_f000;
				((lapic_base+0xb0) as *mut u32).write_volatile(0x00);
				
//				let _ = writeln!(tty_writer(), "> IN ISR: {}", $id);
				if $ec {
					let _errcode: usize;
					let rip: usize;
					asm!(
						"pop {temp:r}", // Pop frame return rip
						"pop {errcode:r}", // Pop error code
						"push {temp:r}", // Re-push frame retur rip
						
						errcode = out(reg) _errcode,
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

isr_entry!(isr_other => echo_handler("#<other>"); false);
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
	
	while let Some(c) = unsafe {tty::read_tty_char()} {
//		let _ = writeln!(tty_writer(), "tty >> {:02X}h", c);
//		let _ = write!(tty_writer(), "{}", char::from_u32(c).unwrap());
		let _ = tty_writer().write_char(char::from_u32(c as u32).unwrap());
	}
	
	let iir = unsafe {crate::arch::x86_64::port::inb(0x3F8 + 2)};
	
//	let _ = writeln!(tty_writer(), "tty status {:08b}", iir);
//	for _ in 0..(0x1<<20) {}
}
