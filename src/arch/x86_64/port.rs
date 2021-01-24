
#[inline(always)]
pub unsafe fn outb(port: u16, val: u8) {
	asm!(
		"out dx, al",
		in("dx") port,
		in("al") val,
	);
}

#[inline(always)]
pub unsafe fn inb(port: u16) -> u8 {
	let val;
	asm!(
		"in al, dx",
		in("dx") port,
		out("al") val,
	);
	val
}
