
// TODO: Figure out the best way of having arch specific functionality
//  in top level modules (like syscall, mem, proc) instead of the arch
//  submodules (or maybe that *is* the best option, I dunno, figure it out!)

// USCI unpriveleged syscall interface
// PSCI priveleged -"-

use core::arch::asm;

#[naked]
pub unsafe extern "C" fn syscall_handler_long() {
	asm!(
		"ud2",
		options(noreturn)
	);
}

#[naked]
pub unsafe extern "C" fn syscall_handler_compat() {
	asm!(
		"ud2",
		options(noreturn)
	);
}
