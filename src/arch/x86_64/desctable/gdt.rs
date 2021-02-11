
/// Intel 64 Vol. 3A, §3.4.5 (p.2880)
/// AMD64 Vol. 2, §4.8 (p.542)
/// 
/// Long mode code segment descriptor
/// TODO: Refactor this into a general "LongSegmentDesc" with an fn desc_type(&self) -> LongModeDescType
///  that then indicates which fields are valid and their meaning
#[derive(Clone, Debug)]
#[repr(transparent)]
pub struct LongCodeDataSegmentDesc([u32; 2]);

impl LongCodeDataSegmentDesc {
	pub const fn new_code(d: u8, l: u8, p: u8, dpl: u8, c: u8) -> Self {
		Self([
			0x0000_0000,
			(0b1 << 23) | (d as u32 & 0b1) << 22 | (l as u32 & 0b1) << 21 | (0b0000 << 16) | (p as u32 & 0b1) << 15 | (dpl as u32 & 0b11) << 13 | (0b11 << 11) | (c as u32 & 0b1) << 10 | (0b10 << 8),
		])
	}
	
	/// Note that dpl is ignored in long mode and thus kinda
	/// unncessary but we just set it out of principle.
	/// Plus compat mode does actually require it IIRC.
	pub const fn new_data(p: u8, dpl: u8) -> Self {
		Self([
			0x0000_0000,
			(0b1100 << 20) | (0b0000 << 16) | (p as u32 & 0b1) << 15 | (dpl as u32 & 0b11) << 13 | (0b10u32 << 11) | (0b010u32 << 8)
		])
	}
}

#[derive(Clone, Debug)]
#[repr(transparent)]
pub struct LongNullSegmentDesc([u32; 2]);

impl LongNullSegmentDesc {
	pub const fn new() -> Self {
		Self([0x0000_0000, 0x0000_0000])
	}
}

#[derive(Clone, Debug)]
#[repr(transparent)]
pub struct LongSystemSegmentDesc([u32; 4]);

impl LongSystemSegmentDesc {
	/// Note: `base_addr` must be in canonical form
	pub const fn new(base_addr: u64, limit: u32, g: u8, avl: u8, p: u8, dpl: u8, desc_type: u8) -> Self {
		Self([
			((base_addr & 0xffff) as u32) << 16 | (limit as u32 & 0xffff),
			(base_addr & 0xff00_0000) as u32 | (g as u32 & 0b1) << 23 | (avl as u32 & 0b1) << 20 | (limit as u32 & 0x000f_0000) | (p as u32 & 0b1) << 15 | (dpl as u32 & 0b11) << 13 | (0b0 << 12) | (desc_type as u32 & 0b1111) << 8 | ((base_addr >> 8) as u32 & 0xffff),
			(base_addr >> 32) as u32,
			0x0000_0000,
		])
	}
}
