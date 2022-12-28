use core::arch::asm;

pub type PortAddr = u16;

#[inline(always)]
pub unsafe fn outb(port: PortAddr, val: u8) {
	asm!(
		"out dx, al",
		in("dx") port,
		in("al") val,
		options(nostack),
	);
}

#[inline(always)]
pub unsafe fn inb(port: PortAddr) -> u8 {
	let val;
	asm!(
		"in al, dx",
		in("dx") port,
		out("al") val,
		options(nostack),
	);
	val
}
