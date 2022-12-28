
//pub static IDT_BUFFER: [u64; 256] = [0; 256];

use core::arch::asm;

#[inline(always)]
pub unsafe fn cli() {
	asm!("cli", options(nostack));
}

#[inline(always)]
pub unsafe fn sti() {
	asm!("sti", options(nostack));
}
