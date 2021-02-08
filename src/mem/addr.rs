
#[deprecated(note = "Use normal pointers/references instead which are implicitely assumed to be in the virtual memory map of the kernel.")]
#[repr(transparent)]
pub struct Virt<A: Addr>(A);

#[repr(transparent)]
pub struct Phys<A: Addr>(A);

impl<A: Addr> Phys<A> {
	pub const fn new(raw: A) -> Self {
		Self(raw)
	}
	
	pub fn ptr(self) -> A {
		self.0
	}
}

pub trait Addr {}

impl<T: ?Sized> Addr for *const T {}
impl<T: ?Sized> Addr for *mut T {}

// Normal references are probably a bit too dangerous specifically
// when used in a Phys wrapper as it makes it too easy to use
// and dereference them (which will happen in the current virtual
// memory map) while they actually refer to physical memory.
//impl<T: ?Sized> Addr for &T {}
//impl<T: ?Sized> Addr for &mut T {}
