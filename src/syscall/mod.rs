//! amd64 syscall implementation (using syscall/sysret)
//! 
//! [`syscall_entry_long0`] is the syscall entry stored into `LSTAR`.
//! 
//! Note that we are saving all 16 gprs since they are either
//! sysv64 callee-saved (and thus we want to preserve them anyways),
//! are syscall arguments (including rax) or are rcx and r11 which
//! we also need to save. In fact every single reg except r10
//! we strictly need to preserve so we might as well preserve all of them,
//! with the exception of rcx and r11 which we technically cannot preserve
//! and are therefore cleared upon sysret.
//! 
//! Steps of a syscall
//! * User sets up arguments in the proper registers
//! * User execs syscall
//! * ...
//! * GS is swapped to grant access to per-cpu kernel data
//! * Stack is switched to kernel stack
//! * User regs are saved to the kernel stack (having made sure not to
//!   clobber any regs in the process!)
//!   TODO: Maybe either use direct segment addressing `[cs:...]` or maybe use one reg like r10 as a scratch reg that is also not preserved?
//! * Syscall is serviced
//! * TODO: Syscall return values get stored into return regs
//! * GS segment is swapped back
//! * User regs are restored (including rbp and rsp, restoring the user stack)
//! * sysret is executed

// TODO: Figure out the best way of having arch specific functionality
//  in top level modules (like syscall, mem, proc) instead of the arch
//  submodules (or maybe that *is* the best option, I dunno, figure it out!)

// USCI unpriveleged syscall interface
// PSCI priveleged -"-

use core::arch::asm;

use memoffset::offset_of;

use crate::tty;

// Note:
// - regs: sysv_x86-64-psABI-1.0.pdf, page 24

/// Long mode syscall entry, pointed to by LSTAR
/// 
/// Naked function as it is directly called by the syscall instruction.
/// extern "sysv64" so that it internally assumes the sysv64 ABI.
// TODO: extern "C" should also be fine as it's based on our platform json file?
//  Not sure, but using sysv64 explicitely should definitely be correct
#[naked]
pub unsafe extern "sysv64" fn syscall_entry_long0() -> ! {
	asm!(
		// [Switch to kernel stack]
		// (Sadly this must be done in asm as `syscall_handler_long` assumes
		//  it's a normal sysv64 function which can use the stack and clobber regs)
		
		// Swap gs segment which contains per-cpu kernel info for the current cpu
		"swapgs",
		
		// Temporarily save um rsp before overwriting with kernel stack rsp
		"mov gs:[{per_cpu_temp_um_rsp_offset}], rsp",
		
		// Load kernel stack from gs which points to this cpu's PerCpuData
		"mov rsp, gs:[{per_cpu_kern_stack_offset}]",
		
		// Re-enable irqs after syscall has disabled them
		// This is massively important to prevent a race condition
		// when another interrupt is serviced between syscall elevating the CPL to 0
		// and switching to the kernel stack
		// (At least I think that might be bad, depends on how I implement other isrs)
		"sti",
		
		// [Save user regs to kernel stack]
		// This also has to be done in asm so that the abi doesn't clobber any regs we need to save
		// Also we need to push them backwards since the stack grows downwards, so that they
		// have the same layout as `SyscallSavedRegs`
		"push r15",
		"push r14",
		"push r13",
		"push r12",
		"push r11",
		"push r10",
		"push  r9",
		"push  r8",
		"push rdi",
		"push rsi",
		"push rbp",
		"push gs:[{per_cpu_temp_um_rsp_offset}]",
		"push rdx",
		"push rcx",
		"push rbx",
		"push rax",
		
		// After saving all the regs we can complete the stack switch by
		// setting the kernel mode rbp
		
		// Load pointer to SyscallSavedRegs into rdi (first fn arg `saved_regs`)
		"mov rdi, rsp",
		
		// Jump to the actual syscall handler fn
		// Note: Make sure to have proper stack alignment of 16 bytes!
		// (In this case we have since we push 128 byte (16 regs à 8 byte) which is aligned to 16)
		"jmp {handler}",
		
		handler = sym syscall_handler_long,
		per_cpu_kern_stack_offset = const {offset_of!(PerCpuData, syscall_stack_base)},
		per_cpu_temp_um_rsp_offset = const {offset_of!(PerCpuData, syscall_temp_um_rsp)},
	
		options(noreturn)
	)
}

/// The real syscall handler logic, called manually from the
/// asm block in [`syscall_entry_long0`]
pub extern "sysv64" fn syscall_handler_long(saved_regs: &SyscallSavedRegs) -> ! {
	// Service the syscall
	service_syscall(saved_regs);
	
	// Unwind syscall
	unsafe {
		asm!(
			// Restore user regs (and sysret needed rcx and r11)
			"mov rax, [{saved_regs} +  0]",
			"mov rbx, [{saved_regs} +  8]",
			// Target rip for sysret
			"mov rcx, [{saved_regs} + 16]",
			"mov rdx, [{saved_regs} + 24]",
			"mov rsp, [{saved_regs} + 32]",
			"mov rbp, [{saved_regs} + 40]",
			"mov rsi, [{saved_regs} + 48]",
			"mov rdi, [{saved_regs} + 56]",
			"mov  r8, [{saved_regs} + 64]",
			"mov  r9, [{saved_regs} + 72]",
			"mov r10, [{saved_regs} + 80]",
			// r11 must be loaded with the rflags value to be restored
			// by sysret. For now we just reload the previous rflags.
			"mov r11, [{saved_regs} + 88]",
			"mov r12, [{saved_regs} + 96]",
			"mov r13, [{saved_regs} + 104]",
			"mov r14, [{saved_regs} + 112]",
			"mov r15, [{saved_regs} + 120]",
			
			// Disable irqs before swapgs
			// (irqs are reenabled by sysret due to reloading rflags)
			"cli",
			
			// Swap gs back
			"swapgs",
			
			// sysret to um (q suffix to make sure it doesn't enter compat mode)
			// Reloads rflags from r11, the target rip from rcx and CS and SS from IA32_STAR
			"sysretq",
			
			// TODO: This might break and overwrite another reg instead of just using
			//  rdi which is the outer functions 1st argument...
			saved_regs = in(reg) saved_regs,
			
			options(noreturn)
		)
	}
}

/// Saved registers across syscalls, i.e.
/// stored by the syscall handler on entry and restored
/// before sysret'ing.
/// 
/// Refer to the docs on `syscall\mod.rs` for further info.
/// Marked as a copy type to make sure there cannot be a Drop impl
/// since we won't be calling that.
#[repr(C)]
#[derive(Copy, Clone)]
pub struct SyscallSavedRegs {
	/// +0
	pub rax_syscall_idx: u64,
	/// +8
	pub rbx: u64,
	/// +16
	pub rcx_um_rip: u64,
	/// +24
	pub rdx_arg2: u64,
	/// +32
	pub rsp: u64,
	/// +40
	pub rbp: u64,
	/// +48
	pub rsi_arg1: u64,
	/// +56
	pub rdi_arg0: u64,
	/// +64
	pub r8_arg3: u64,
	/// +72
	pub r9: u64,
	/// +80
	pub r10: u64,
	/// +88
	pub r11_rflags: u64,
	/// +96
	pub r12: u64,
	/// +104
	pub r13: u64,
	/// +112
	pub r14: u64,
	/// +120
	pub r15: u64,
}

#[inline]
fn service_syscall(regs: &SyscallSavedRegs) {
	// DEBUG:
	unsafe {
		tty::write_tty(b"I'm servicing a syscall");
	}
	
	// TODO: Actually service syscalls
}

#[repr(C)]
pub struct PerCpuData {
	pub syscall_stack_base: *const u8,
	/// A place to temporarily save the usermode rsp
	/// on syscall entry, so that the kernel can overwrite the rsp
	/// with it's own kernel stack rsp and start saving regs
	pub syscall_temp_um_rsp: u64,
}

/*
// Godbolt stuff:
/// Usermode syscall entry with 3 of the 5 max args
pub fn syscall3_um(idx: u64, arg0: u64, arg1: u64, arg2: u64) {
    unsafe {
        asm!(
            "syscallq",
		
            // Map inputs
            in("rax") idx,
            in("rdi") arg0,
            in("rsi") arg1,
            in("rdx") arg2,

            // Clobber sysv64 caller-saved regs
            lateout("rax") _,
            lateout("rcx") _,
            lateout("rdx") _,
            lateout("rsi") _,
            lateout("rdi") _,
            lateout("r8")  _,
            lateout("r9")  _,
            lateout("r10") _,
            lateout("r11") _,
        );
    }
}

pub fn do_syscall_um(a: u32, b: u64) {
    let idx = b + (a << 4) as u64;

    syscall3_um(idx, 0, 0, 0);
}
*/
