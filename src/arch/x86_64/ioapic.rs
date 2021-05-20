use core::marker::PhantomData;

use bitfield::bitfield;

use crate::mem::Phys;
use crate::PtrOpsExt;
use crate::vga::write_regs;
use core::mem::MaybeUninit;

// TODO: This whole architecture is still kinda bad
//  since R/W is not specified per IO APIC register
//  but per register field (some are actually just read only)
//  eventhough the reg is r/w.
//  So mabye figure that out (i dont know if were maybe
//  even just allowed to write read only fields it just
//  doesnt have an effect. i dunno figure something out)

pub struct IoApicDesc {
	/// The order of this IO APIC (0 = first, 1 = second, (n+2)th, ..)
	/// derived from the order the IO APICs
	/// descriptions appear in the MADT.
	/// 
	/// This could be useful for determining the first IO APIC
	/// (which is the one with the legacy ISA irqs mapped to it),
	/// since "first" is defined as the first one that appears
	/// in the MADT.
	pub order: u32,
	
	/// This IO APICs apic id
	pub id: u8,
	
	/// This IO APICs physical address.
	/// 
	/// Must be 16 byte aligned and be somwhere in
	/// the range of `0xfec0_0000` to `0xfec1_0000` (excl.)
	pub regs: Phys<*mut u128>,
	//pub regs: IoApicRegs,
	
	/// The nr of the base Global System Interrupt that
	/// is mapped to this IO APICs first interrupt
	/// input
	pub base_gsi: u32,
}

impl IoApicDesc {
	#[inline]
	pub unsafe fn write_reg<V: IoRegVal>(&self, reg: IoReg<V, impl IoWritable>, val: V) {
		let base = self.regs.ptr() as *mut u32;
		
		// Write IOREGSEL
		base.write_volatile(reg.offset() as u32);
		
		// Write IOWIN
		base.byte_offset(0x10).write_volatile(val.into_raw());
	}
	
	#[inline]
	pub unsafe fn read_reg<V: IoRegVal>(&self, reg: IoReg<V, impl IoReadable>) -> V {
		let base = self.regs.ptr() as *mut u32;
		
		// Write IOREGSEL
		base.write_volatile(reg.offset() as u32);
		
		// Write IOWIN
		V::from_raw(base.byte_offset(0x10).read_volatile())
	}
	
	pub unsafe fn write_redir(&self, idx: u8, entry: IoApicRedTblVal) {
		let (lo, hi) = entry.into_raw_regs();
		self.write_reg(IOREDTBL_LO(idx), lo);
		self.write_reg(IOREDTBL_HI(idx), hi);
	}
	
	pub unsafe fn read_redir(&self, idx: u8) -> MaybeUninit<IoApicRedTblVal> {
		let lo = self.read_reg(IOREDTBL_LO(idx));
		let hi = self.read_reg(IOREDTBL_HI(idx));
		MaybeUninit::new(IoApicRedTblVal::from_raw_regs(lo, hi))
	}
}

macro_rules! impl_reg_val {
	($name:ident) => {
		impl IoRegVal for $name {
			fn from_raw(raw: u32) -> Self {
				Self(raw)
			}
			
			fn into_raw(self) -> u32 {
				self.0
			}
		}
	}
}

pub const IOAPICID: IoReg<IoApicIdVal, R> = IoReg::from_raw(0x00);
pub const IOAPICVER: IoReg<IoApicVerVal, R> = IoReg::from_raw(0x01);
pub const IOAPICARB: IoReg<IoApicArbVal, R> = IoReg::from_raw(0x02);

#[allow(non_snake_case)]
pub const fn IOREDTBL_HI(idx: u8) -> IoReg<IoApicRedTblHiVal, RW> {
	IoReg::from_raw(0x11u8.checked_add(idx.checked_mul(2).unwrap()).unwrap())
}
#[allow(non_snake_case)]
pub const fn IOREDTBL_LO(idx: u8) -> IoReg<IoApicRedTblLoVal, RW> {
	IoReg::from_raw(0x10u8.checked_add(idx.checked_mul(2).unwrap()).unwrap())
}

bitfield! {
	#[derive(Copy, Clone)]
	pub struct IoApicIdVal(u32);
	impl Debug;
	
	#[inline]
	pub from into u32/*u8*/, id, set_id : 27, 24;
}
impl_reg_val!(IoApicIdVal);

bitfield! {
	#[derive(Copy, Clone)]
	pub struct IoApicVerVal(u32);
	impl Debug;
	
	#[inline]
	pub from into u32/*u8*/, max_redir_entries, set_max_redir_entries : 23, 16;
	#[inline]
	pub from into u32/*u8*/, version, set_version : 7, 0;
}
impl_reg_val!(IoApicVerVal);

bitfield! {
	#[derive(Copy, Clone)]
	pub struct IoApicArbVal(u32);
	impl Debug;
	
	#[inline]
	pub from into u32/*u8*/, arb_id, set_arb_id : 27, 24;
}
impl_reg_val!(IoApicArbVal);

pub type IoApicRedTblHiVal = u32;
pub type IoApicRedTblLoVal = u32;

bitfield! {
	#[derive(Copy, Clone)]
	pub struct IoApicRedTblVal(u64);
	impl Debug;
	
	#[inline]
	pub from into u64/*u8*/, dest_field, set_dest_field : 63, 56;
	#[inline]
	pub from into bool, interrupt_mask, set_interrupt_mask : 16;
	#[inline]
	pub from into TriggerMode, trigger_mode, set_trigger_mode : 15, 15;
	#[inline]
	pub from into bool, remote_irr, set_remote_irr : 14;
	#[inline]
	pub from into IrqPolarity, polarity, set_polarity : 13, 13;
	#[inline]
	pub from into DeliveryStatus, delv_status, set_delv_status : 12, 12;
	#[inline]
	pub from into DestinationMode, dest_mode, set_dest_mode : 11, 11;
	#[inline]
	pub from into DeliveryMode, delv_mode, set_delv_mode : 10, 8;
	#[inline]
	pub from into u64/*u8*/, irq_vector, set_irq_vector : 7, 0;
}
impl IoApicRedTblVal {
	#[inline]
	pub fn from_raw_regs(lo: IoApicRedTblLoVal, hi: IoApicRedTblHiVal) -> Self {
		Self((lo as u64) | ((hi as u64) << 32))
	}
	
	#[inline]
	pub fn into_raw_regs(self) -> (IoApicRedTblLoVal, IoApicRedTblHiVal) {
		(self.0 as u32, (self.0 >> 32) as u32)
	}
	
	// TODO: implement
	pub fn set_full_dest(&mut self) {}
}

#[derive(Copy, Clone, Debug)]
pub enum TriggerMode {
	LevelSensitive,
	EdgeSensitive,
}
impl Into<u64> for TriggerMode {
	fn into(self) -> u64 {
		match self {
			Self::EdgeSensitive => 0,
			Self::LevelSensitive => 1,
		}
	}
}
impl Into<TriggerMode> for u64 {
	fn into(self) -> TriggerMode {
		match self {
			0 => TriggerMode::EdgeSensitive,
			_ => TriggerMode::LevelSensitive,
		}
	}
}

#[derive(Copy, Clone, Debug)]
pub enum IrqPolarity {
	ActiveHigh,
	ActiveLow,
}
impl Into<u64> for IrqPolarity {
	fn into(self) -> u64 {
		match self {
			Self::ActiveHigh => 0,
			Self::ActiveLow => 1,
		}
	}
}
impl Into<IrqPolarity> for u64 {
	fn into(self) -> IrqPolarity {
		match self {
			0 => IrqPolarity::ActiveHigh,
			_ => IrqPolarity::ActiveLow,
		}
	}
}

#[derive(Copy, Clone, Debug)]
pub enum DeliveryStatus {
	Idle,
	SendPending,
}
impl Into<u64> for DeliveryStatus {
	fn into(self) -> u64 {
		match self {
			Self::Idle => 0,
			Self::SendPending => 1,
		}
	}
}
impl Into<DeliveryStatus> for u64 {
	fn into(self) -> DeliveryStatus {
		match self {
			0 => DeliveryStatus::Idle,
			_ => DeliveryStatus::SendPending,
		}
	}
}

#[derive(Copy, Clone, Debug)]
pub enum DestinationMode {
	Physical,
	Logical,
}
impl Into<u64> for DestinationMode {
	fn into(self) -> u64 {
		match self {
			Self::Physical => 0,
			Self::Logical => 1,
		}
	}
}
impl Into<DestinationMode> for u64 {
	fn into(self) -> DestinationMode {
		match self {
			0 => DestinationMode::Physical,
			_ => DestinationMode::Logical,
		}
	}
}

#[derive(Copy, Clone, Debug)]
pub enum DeliveryMode {
	Fixed,
	LowestPriority,
	Smi,
	Reserved1,
	Nmi,
	Init,
	Reserved2,
	ExtInt,
}
impl Into<u64> for DeliveryMode {
	fn into(self) -> u64 {
		match self {
			Self::Fixed => 0b000,
			Self::LowestPriority => 0b001,
			Self::Smi => 0b010,
			Self::Reserved1 => 0b011,
			Self::Nmi => 0b100,
			Self::Init => 0b101,
			Self::Reserved2 => 0b110,
			Self::ExtInt => 0b111,
		}
	}
}
impl Into<DeliveryMode> for u64 {
	fn into(self) -> DeliveryMode {
		match self {
			0b000 => DeliveryMode::Fixed,
			0b001 => DeliveryMode::LowestPriority,
			0b010 => DeliveryMode::Smi,
			0b011 => DeliveryMode::Reserved1,
			0b100 => DeliveryMode::Nmi,
			0b101 => DeliveryMode::Init,
			0b110 => DeliveryMode::Reserved2,
			0b111 => DeliveryMode::ExtInt,
			_ => panic!("Invalid bits for io apic delivery mode"),
		}
	}
}

#[derive(Copy, Clone)]
pub struct IoReg<V: IoRegVal, A: IoAccess> {
	offset: u8,
	_ph: PhantomData<(V, A)>,
}

impl<V: IoRegVal, A: IoAccess> IoReg<V, A> {
	pub const fn from_raw(offset: u8) -> Self {
		Self {
			offset,
			_ph: PhantomData,
		}
	}
	
	#[inline(always)]
	pub const fn offset(self) -> u8 {
		self.offset
	}
}

pub trait IoRegVal {
	fn from_raw(raw: u32) -> Self;
	fn into_raw(self) -> u32;
}

impl IoRegVal for u32 {
	fn from_raw(raw: u32) -> Self {
		raw
	}
	
	fn into_raw(self) -> u32 {
		self
	}
}

pub trait IoAccess {}
pub trait IoReadable: IoAccess {}
pub trait IoWritable: IoAccess {}

pub struct R {}
impl IoAccess for R {}
impl IoReadable for R {}

pub struct W {}
impl IoAccess for W {}
impl IoWritable for W {}

pub struct RW {}
impl IoAccess for RW {}
impl IoReadable for RW {}
impl IoWritable for RW {}
