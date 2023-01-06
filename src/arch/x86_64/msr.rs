use core::arch::asm;
use core::marker::PhantomData;

pub const EFER: Msr = Msr::from_nr(0xC000_0080);
pub const STAR: Msr = Msr::from_nr(0xC000_0081);
pub const LSTAR: Msr = Msr::from_nr(0xC000_0082);
pub const CSTAR: Msr = Msr::from_nr(0xC000_0083);
pub const SFMASK: Msr = Msr::from_nr(0xC000_0084);

pub const IA32_APIC_BASE: Msr = Msr::from_nr(0x0000_001b); // TODO: Is this the right addr?

#[derive(Copy, Clone)]
pub struct Msr<T: MsrData = u64>(u32, PhantomData<*const T>);

impl<T: MsrData> Msr<T> {
	#[inline(always)]
	pub const fn from_nr(nr: u32) -> Self {
		Self(nr, PhantomData)
	}
	
	#[inline(always)]
	pub const fn nr(&self) -> u32 {
		self.0
	}
	
	#[inline(always)]
	pub unsafe fn read(&self) -> T {
		T::from_raw(read_msr(self.0))
	}
	
	#[inline(always)]
	pub unsafe fn write(&self, val: T) {
		write_msr(self.0, val.to_raw());
	}
	
	#[inline(always)]
	pub unsafe fn mutate(&self, op: impl FnOnce(T) -> T) {
		let old_val = T::from_raw(read_msr(self.0));
		let new_val = (op)(old_val).to_raw();
		write_msr(self.0, new_val);
	}
	
	#[inline(always)]
	pub unsafe fn read_raw(&self) -> u64 {
		read_msr(self.0)
	}
	
	#[inline(always)]
	pub unsafe fn write_raw(&self, val: u64) {
		write_msr(self.0, val);
	}
	
	#[inline(always)]
	pub unsafe fn mutate_raw(&self, op: impl FnOnce(u64) -> u64) {
		let old_val = read_msr(self.0);
		let new_val = (op)(old_val);
		write_msr(self.0, new_val);
	}
}

// Needed to make Msr Send and Sync since PhantomData<*const T> isn't
unsafe impl<T: MsrData> Send for Msr<T> {}
unsafe impl<T: MsrData> Sync for Msr<T> {}

/// Some 8-byte piece of data stored in a specified msr
pub trait MsrData: Copy {
	fn to_raw(self) -> u64;
	fn from_raw(raw: u64) -> Self;
}

impl MsrData for u64 {
	#[inline]
	fn to_raw(self) -> u64 {
		self
	}
	
	#[inline]
	fn from_raw(raw: u64) -> Self {
		raw
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
