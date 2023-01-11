use core::fmt;
use core::fmt::Pointer;
use core::ptr::NonNull;

#[deprecated(note = "Use normal pointers/references instead which are implicitely assumed to be in the virtual memory map of the kernel.")]
#[repr(transparent)]
pub struct Virt<A: Addr>(pub A);

#[repr(transparent)]
pub struct Phys<A: Addr>(pub A);

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
impl<T: ?Sized> Addr for NonNull<T> {}

impl<T: ?Sized> Clone for Phys<*const T> {
	fn clone(&self) -> Self {
		*self
	}
}
impl<T: ?Sized> Clone for Phys<*mut T> {
	fn clone(&self) -> Self {
		*self
	}
}
impl<T: ?Sized> Clone for Phys<NonNull<T>> {
	fn clone(&self) -> Self {
		*self
	}
}

impl<T: Addr + fmt::Debug> fmt::Debug for Phys<T> {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		f.debug_tuple("Phys")
			.field(&self.0)
			.finish()
	}
}

impl<T: ?Sized> Copy for Phys<*const T> {}
impl<T: ?Sized> Copy for Phys<*mut T> {}
impl<T: ?Sized> Copy for Phys<NonNull<T>> {}

unsafe impl<T: ?Sized> Send for Phys<*const T> {}
unsafe impl<T: ?Sized> Send for Phys<*mut T> {}
unsafe impl<T: ?Sized> Send for Phys<NonNull<T>> {}

// Normal references are probably a bit too dangerous specifically
// when used in a Phys wrapper as it makes it too easy to use
// and dereference them (which will happen in the current virtual
// memory map) while they actually refer to physical memory.
//impl<T: ?Sized> Addr for &T {}
//impl<T: ?Sized> Addr for &mut T {}
