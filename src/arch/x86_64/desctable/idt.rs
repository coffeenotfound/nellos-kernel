use crate::arch::x86_64::desctable::SegmentSel;
use crate::arch::x86_64::desctable::ty::{LongSegmentDescType};

/// Vol. 3A, §6.14.1 (p.2992)
/// amd64 page 547
/// 
/// Long mode interrupt/trag gate descriptor
#[derive(Copy, Clone, Debug)]
#[repr(transparent)]
pub struct LongIdtDesc([u32; 4]);

impl LongIdtDesc {
	/// Creates a new long mode interrupt/trap gate descriptor
	#[inline(always)]
	pub const fn new(offset: u64, selector: SegmentSel, ist: u8, desc_type: LongSegmentDescType, dpl: u8, present: bool) -> Self {
		Self([
			((selector.to_raw() as u32) << 16) | (offset as u32 & 0xffff),
			(offset as u32 & 0xffff_0000) | ((present as u32 & 0b1) << 15) | ((dpl as u32 & 0b11) << 13) | ((desc_type as u32 & 0b1111) << 8) | (ist as u32 & 0b111),
			(offset >> 32) as u32,
			0,
		])
	}
	
//	#[inline(always)]
//	pub const fn new_empty() -> Self {
//	    // TODO: This doesn't work as per §3.4.5 (p.2881)
//		//  Even with the P bit cleared, some fields must still hold valid values
//		// Present bit is zero, so all zeroes is perfectly fine
//		Self([0; 4])
//	}
	
	pub const fn null() -> Self {
		Self([0; 4])
	}
	
	#[inline(always)]
	pub const fn as_raw(&self) -> &[u32; 4] {
		&self.0
	}
}
