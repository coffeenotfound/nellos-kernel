pub use gdt::*;
pub use idt::*;

mod gdt;
mod idt;

/// Vol. 3A, §3.4.2 (p.2877)
#[derive(Copy, Clone, Debug)]
#[repr(transparent)]
pub struct SegmentSelector(u16);

impl SegmentSelector {
	#[inline(always)]
	pub const fn new(rpl: u8, ldt: bool, index: u16) -> Self {
		// Old buggy impl:
//		let ti = if ldt {1} else {0};
//		Self((rpl & 0x4) as u16 | ti | (index << 3))
		
		let ti = if ldt {1} else {0};
		Self(((index as u16 & 0x1fff) << 3) | ((ti as u16 & 0b1) << 2) | (rpl as u16 & 0b11))
	}
	
	/// Constructs a new null segment selector
	#[inline(always)]
	pub const fn new_null(rpl: u8) -> Self {
		Self(rpl as u16 & 0b11)
	}
	
	#[inline(always)]
	pub const unsafe fn from_raw(raw: u16) -> Self {
		Self(raw)
	}
	
	#[inline(always)]
	pub const fn into_raw(self) -> u16 {
		self.0
	}
}

#[derive(Copy, Clone, Debug)]
#[repr(packed(2))]
pub struct PseudoDesc {
	// Note: The ordering of limit THEN base should be correct
	// according to the processor reference manuals.
	// Note also that this only applies to m16&64 operands,
	// m16:64 operands are (AFAIK) ordered exactly the opposite way
	// (first 64-bit base then 16-bit limit)!
	pub limit: u16,
	pub base: u64,
}
