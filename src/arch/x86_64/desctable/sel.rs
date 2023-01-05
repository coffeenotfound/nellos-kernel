
#[derive(Copy, Clone, Debug)]
#[repr(transparent)]
pub struct SegmentSel(u16);

impl SegmentSel {
	/// idx is masked with 0x1fff
	#[inline]
	pub const fn new(rpl: u8, ti: SegmentSelTI, idx: u16) -> Self {
		let cooked_idx = (idx & 0x1fff) << 3;
		let cooked_ti = (ti as u16) << 2;
		let cooked_rpl = (rpl as u16 & 0x3);
		
		Self(cooked_idx | cooked_ti | cooked_rpl)
	}
	
	pub const fn to_raw(self) -> u16 {
		self.0
	}
}

#[repr(u8)]
pub enum SegmentSelTI {
	GDT = 0x0,
	IDT = 0x1,
}
