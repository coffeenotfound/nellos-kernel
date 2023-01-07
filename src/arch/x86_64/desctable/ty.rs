
/// Long mode system-segment descriptor types
/// 
/// amd64 page 544 
#[derive(Copy, Clone, Eq, PartialEq, Debug)]
#[non_exhaustive]
#[repr(u8)]
pub enum LongSegmentDescType {
	Ldt64 = 0b0010,
	AvailableTss64 = 0b1001,
	BusyTss64 = 0b1011,
	CallGate64 = 0b1100,
	InterruptGate64 = 0b1110,
	TrapGate64 = 0b1111,
}
