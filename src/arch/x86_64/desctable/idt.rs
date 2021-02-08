use crate::arch::x86_64::desctable::SegmentSelector;

/// Vol. 3A, §6.14.1 (p.2992)
/// 
/// Long mode interrupt descriptor
#[derive(Clone, Debug)]
#[repr(transparent)]
pub struct LongIdtDesc([u32; 4]);

impl LongIdtDesc {
	#[inline(always)]
	pub const fn new(offset: u64, selector: SegmentSelector, ist: u8, desc_type: u8, dpl: u8, p: u8) -> Self {
		Self([
			((selector.into_raw() as u32) << 16) | (offset as u32 & 0xffff),
			(offset as u32 & 0xffff_0000) | ((p as u32 & 0b1) << 15) | ((dpl as u32 & 0b11) << 13) | ((desc_type as u32 & 0b1111) << 8) | (ist as u32 & 0b11),
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
	
	#[inline(always)]
	pub const fn as_raw(&self) -> &[u32; 4] {
		&self.0
	}
}
