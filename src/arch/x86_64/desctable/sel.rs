
/// Vol. 3A, §3.4.2 (p.2877)
#[derive(Copy, Clone, Debug)]
#[repr(transparent)]
pub struct SegmentSel(u16);

impl SegmentSel {
	#[inline(always)]
	#[allow(unused_parens)]
	pub const fn new(rpl: u8, ti: SegmentSelTI, idx: u16) -> Self {
		// Old buggy impl:
//		let ti = if ldt {1} else {0};
//		Self((rpl & 0x4) as u16 | ti | (index << 3))
		
//		let ti = if ldt {1} else {0};
// 		Self(((index as u16 & 0x1fff) << 3) | ((ti as u16 & 0b1) << 2) | (rpl as u16 & 0b11))
		
		let cooked_idx = (idx & 0x1fff) << 3;
		let cooked_ti = (ti as u16) << 2;
		let cooked_rpl = (rpl as u16 & 0x3);
		
		Self(cooked_idx | cooked_ti | cooked_rpl)
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
	pub const fn to_raw(self) -> u16 {
		self.0
	}
}

#[repr(u8)]
pub enum SegmentSelTI {
	GDT = 0x0,
	IDT = 0x1,
}

/// Used in LIDT and LGDT
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
