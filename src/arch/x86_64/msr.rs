
pub const EFER: Msr = unsafe {Msr::from_nr(0xC000_0080)};
pub const STAR: Msr = unsafe {Msr::from_nr(0xC000_0081)};
pub const LSTAR: Msr = unsafe {Msr::from_nr(0xC000_0082)};
pub const CSTAR: Msr = unsafe {Msr::from_nr(0xC000_0083)};
pub const SFMASK: Msr = unsafe {Msr::from_nr(0xC000_0084)};

pub const IA32_APIC_BASE: Msr = unsafe {Msr::from_nr(0x0000_001b)}; // TODO: Is this the right addr?

pub struct Msr(u32);

impl Msr {
	#[inline(always)]
	pub const unsafe fn from_nr(nr: u32) -> Self {
		Self(nr)
	}
	
	#[inline(always)]
	pub const fn nr(&self) -> u32 {
		self.0
	}
	
	#[inline(always)]
	pub unsafe fn read(&self) -> u64 {
		read_msr(self.0)
	}
	
	#[inline(always)]
	pub unsafe fn write(&self, val: u64) {
		write_msr(self.0, val);
	}
}

#[inline]
pub(self) unsafe fn read_msr(msr: u32) -> u64 {
	let low: u32;
	let high: u32;
	
	asm!(
		"rdmsr",
		in("ecx") msr,
		out("eax") low,
		out("edx") high,
		options(nostack),
	);
	
	((high as u64) << 32) | low as u64
}

#[inline]
pub(self) unsafe fn write_msr(msr: u32, val: u64) {
	asm!(
		"wrmsr",
		in("ecx") msr,
		in("eax") val as u32,
		in("edx") (val >> 32) as u32,
		options(nostack),
	);
}
