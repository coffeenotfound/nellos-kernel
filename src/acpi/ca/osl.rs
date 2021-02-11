#![allow(non_snake_case)]
#![allow(unused_variables)] // TODO: Only for now

use core::ptr;

use cty::{c_char, c_void};

use acpica_sys::*;
pub use ty::*;
use core::sync::atomic::Ordering::SeqCst;

//acpica_sys::gen_osl!(crate::acpi::ca::osl::ty);

pub mod ty {
	#![allow(non_camel_case_types)]
	
	pub type ACPI_CPU_FLAGS = u32;
	pub type ACPI_THREAD_ID = u64;
	
	pub type ACPI_SPINLOCK = *mut cty::c_void;
	pub type ACPI_SEMAPHORE = *mut cty::c_void;
	pub type ACPI_MUTEX = *mut cty::c_void;
	
	pub type ACPI_CACHE_T = acpica_sys::ACPI_MEMORY_LIST;
}
/*
 * OSL Initialization and shutdown primitives
 */
#[no_mangle]
pub extern "C" fn AcpiOsInitialize() -> ACPI_STATUS {
	// Do nothing for now
	AE_OK
}

#[no_mangle]
pub extern "C" fn AcpiOsTerminate() -> ACPI_STATUS {
	// Do nothing for now
	AE_OK
}

/*
 * ACPI Table interfaces
 */
#[no_mangle]
pub extern "C" fn AcpiOsGetRootPointer() -> ACPI_PHYSICAL_ADDRESS {
	let root_ptr = crate::acpi::ACPI_ROOT_PTR.load(SeqCst).ptr();
	
	if root_ptr.is_null() {
		panic!("ACPI root pointer is null");
	} else {
		root_ptr as ACPI_PHYSICAL_ADDRESS
	}
}

#[no_mangle]
pub extern "C" fn AcpiOsPredefinedOverride(init_val: *const ACPI_PREDEFINED_NAMES, new_val: &mut ACPI_STRING) -> ACPI_STATUS {
	// Don't override any predefined object
	*new_val = ptr::null_mut();
	
	AE_OK
}

#[no_mangle]
pub extern "C" fn AcpiOsTableOverride(existing_table: *const ACPI_TABLE_HEADER, new_table: &mut *mut ACPI_TABLE_HEADER) -> ACPI_STATUS {
	// Don't override any table
	*new_table = ptr::null_mut();
	
	AE_OK
}

#[no_mangle]
pub extern "C" fn AcpiOsPhysicalTableOverride(existing_table: *const ACPI_TABLE_HEADER, new_address: &mut ACPI_PHYSICAL_ADDRESS, new_table_length: &mut UINT32) -> ACPI_STATUS {
	// Don't override any table
	*new_address = ptr::null_mut::<cty::c_void>() as ACPI_PHYSICAL_ADDRESS;
	*new_table_length = 0;
	
	AE_OK
}

/*
 * Spinlock primitives
 */
#[no_mangle]
pub extern "C" fn AcpiOsCreateLock(out_handle: &mut ACPI_SPINLOCK) -> ACPI_STATUS {
	unimplemented!()
}

#[no_mangle]
pub extern "C" fn AcpiOsDeleteLock(handle: ACPI_SPINLOCK) {
	unimplemented!()
}

#[no_mangle]
pub extern "C" fn AcpiOsAcquireLock(handle: ACPI_SPINLOCK) -> ACPI_CPU_FLAGS {
	unimplemented!()
}

#[no_mangle]
pub extern "C" fn AcpiOsReleaseLock(handle: ACPI_SPINLOCK, flags: ACPI_CPU_FLAGS) {
	unimplemented!()
}

/*
 * Semaphore primitives
 */
#[no_mangle]
pub extern "C" fn AcpiOsCreateSemaphore(max_unitx: UINT32, initial_units: UINT32, out_handle: &mut ACPI_SEMAPHORE) -> ACPI_STATUS {
	unimplemented!()
}

#[no_mangle]
pub extern "C" fn AcpiOsDeleteSemaphore(handle: ACPI_SEMAPHORE) -> ACPI_STATUS {
	unimplemented!()
}

#[no_mangle]
pub extern "C" fn AcpiOsWaitSemaphore(handle: ACPI_SEMAPHORE, units: UINT32, timeout: UINT16) -> ACPI_STATUS {
	unimplemented!()
}

#[no_mangle]
pub extern "C" fn AcpiOsSignalSemaphore(handle: ACPI_SEMAPHORE, units: UINT32) -> ACPI_STATUS {
	unimplemented!()
}

/*
 * Mutex primitives. May be configured to use semaphores instead via
 * ACPI_MUTEX_TYPE (see platform/acenv.h)
 */
#[no_mangle]
pub extern "C" fn AcpiOsCreateMutex(out_handle: &mut ACPI_MUTEX) -> ACPI_STATUS {
	unimplemented!()
}

#[no_mangle]
pub extern "C" fn AcpiOsDeleteMutex(handle: ACPI_MUTEX) {
	unimplemented!()
}

#[no_mangle]
pub extern "C" fn AcpiOsAcquireMutex(handle: ACPI_MUTEX, timeout: UINT16) -> ACPI_STATUS {
	unimplemented!()
}

#[no_mangle]
pub extern "C" fn AcpiOsReleaseMutex(handle: ACPI_MUTEX) {
	unimplemented!()
}

/*
 * Memory allocation and mapping
 */
#[no_mangle]
pub extern "C" fn AcpiOsAllocate(size: ACPI_SIZE) -> *mut c_void {
	unimplemented!()
}

#[no_mangle]
pub extern "C" fn AcpiOsAllocateZeroed(size: ACPI_SIZE) -> *mut c_void {
	unimplemented!()
}

#[no_mangle]
pub extern "C" fn AcpiOsFree(memory: *mut c_void) {
	unimplemented!()
}

#[no_mangle]
pub extern "C" fn AcpiOsMapMemory(phys_addr: ACPI_PHYSICAL_ADDRESS, length: ACPI_SIZE) -> *mut c_void {
	unimplemented!()
}

#[no_mangle]
pub extern "C" fn AcpiOsUnmapMemory(logical_addr: *mut c_void, size: ACPI_SIZE) {
	unimplemented!()
}

#[no_mangle]
pub extern "C" fn AcpiOsGetPhysicalAddress(logical_addr: *mut c_void, physical_addr: ACPI_PHYSICAL_ADDRESS) -> ACPI_STATUS {
	unimplemented!()
}

/*
 * Memory/Object Cache
 */
//#[no_mangle]
//pub extern "C" fn AcpiOsCreateCache(cache_name: *const c_char, object_size: UINT16, max_depth: UINT16, out_cache: &mut *mut ACPI_CACHE_T) -> ACPI_STATUS {
//	unimplemented!()
//}
//
//#[no_mangle]
//pub extern "C" fn AcpiOsDeleteCache(cache: *mut ACPI_CACHE_T) -> ACPI_STATUS {
//	unimplemented!()
//}
//
//#[no_mangle]
//pub extern "C" fn AcpiOsPurgeCache(cache: *mut ACPI_CACHE_T) -> ACPI_STATUS {
//	unimplemented!()
//}
//
//#[no_mangle]
//pub extern "C" fn AcpiOsAcquireObject(cache: *mut ACPI_CACHE_T) -> *mut c_void {
//	unimplemented!()
//}
//
//#[no_mangle]
//pub extern "C" fn AcpiOsReleaseObject(cache: *mut ACPI_CACHE_T, object: *mut c_void) -> ACPI_STATUS {
//	unimplemented!()
//}

/*
 * Interrupt handlers
 */

/// This function installs an interrupt handler for a hardware interrupt level. The ACPI driver must
/// install an interrupt handler to service the SCI (System Control Interrupt) which it owns. The
/// interrupt level for the SCI interrupt is obtained from the ACPI tables.
/// 
/// **interrupt_level**: Interrupt level that the handler will service.<br>
/// **service_routine**: Address of the handling service routine.<br>
/// **context**: A context value that is passed to the handler when the interrupt is dispatched.<br>
#[no_mangle]
pub extern "C" fn AcpiOsInstallInterruptHandler(interrupt_level: UINT32, service_routine: ACPI_OSD_HANDLER, context: *mut c_void) -> ACPI_STATUS {
	unimplemented!()
}

/// Remove a previously installed hardware interrupt handler.
/// 
/// **interrupt_level**: Interrupt number that the handler is currently servicing.<br>
/// **service_routine**: Address of the handler that was previously installed.<br> 
#[no_mangle]
pub extern "C" fn AcpiOsRemoveInterruptHandler(interrupt_level: UINT32, service_routine: ACPI_OSD_HANDLER) -> ACPI_STATUS {
	unimplemented!()
}

/*
 * Threads and Scheduling
 */
#[no_mangle]
pub extern "C" fn AcpiOsGetThreadId() -> ACPI_THREAD_ID {
	0
//	unimplemented!()
}

#[no_mangle]
pub extern "C" fn AcpiOsExecute(exec_type: ACPI_EXECUTE_TYPE, funtion: ACPI_OSD_EXEC_CALLBACK, context: *mut c_void) -> ACPI_STATUS {
	unimplemented!()
}

#[no_mangle]
pub extern "C" fn AcpiOsWaitEventsComplete() {
	unimplemented!()
}

#[no_mangle]
pub extern "C" fn AcpiOsSleep(millis: UINT64) {
	unimplemented!()
}

#[no_mangle]
pub extern "C" fn AcpiOsStall(micros: UINT32) {
	unimplemented!()
}

/*
 * Platform and hardware-independent I/O interfaces
 */
#[no_mangle]
pub extern "C" fn AcpiOsReadPort(addr: ACPI_IO_ADDRESS, out_val: &mut UINT32, width: UINT32) -> ACPI_STATUS {
	unimplemented!()
}

#[no_mangle]
pub extern "C" fn AcpiOsWritePort(addr: ACPI_IO_ADDRESS, val: UINT32, width: UINT32) -> ACPI_STATUS {
	unimplemented!()
}

#[no_mangle]
pub extern "C" fn AcpiOsReadMemory(addr: ACPI_PHYSICAL_ADDRESS, out_val: &mut UINT64, width: UINT32) -> ACPI_STATUS {
	// This implementation is a damn joke but both osunixxf and oswinxf haveit
	// Thing is, Linux actually implements it correctly and this functions is REALLY important (see acpica's hwregs.c)
	// (linux impl under drivers/acpi/osl.c)
	// TODO: Implement this properly as it is absolute necessary for a functioning ACPICA implementation
	match width {
		8 | 16 | 32 | 64 => {
			*out_val = 0x0;
			AE_OK
		},
		_ => AE_BAD_PARAMETER,
	}
}

#[no_mangle]
pub extern "C" fn AcpiOsWriteMemory(addr: ACPI_PHYSICAL_ADDRESS, val: UINT64, width: UINT32) -> ACPI_STATUS {
	// See note in AcpiOsReadMemory regarding this implementation
	// TODO: Again, see note
	match width {
		8 | 16 | 32 | 64 => {
			AE_OK
		},
		_ => AE_BAD_PARAMETER,
	}
}

/*
 * Platform and hardware-independent PCI configuration space access
 */
#[no_mangle]
pub extern "C" fn AcpiOsReadPciConfiguration(pci_id: *mut ACPI_PCI_ID, reg: UINT32, out_val: &mut UINT64, width: UINT32) -> ACPI_STATUS {
	unimplemented!()
}

#[no_mangle]
pub extern "C" fn AcpiOsWritePciConfiguration(pci_id: *mut ACPI_PCI_ID, reg: UINT32, val: UINT64, width: UINT32) -> ACPI_STATUS {
	unimplemented!()
}

/*
 * Miscellaneous
 */
#[no_mangle]
pub extern "C" fn AcpiOsReadable(ptr: *mut c_void) -> BOOLEAN {
	unimplemented!()
}

#[no_mangle]
pub extern "C" fn AcpiOsWritable(ptr: *mut c_void, length: ACPI_SIZE) -> BOOLEAN {
	unimplemented!()
}

#[no_mangle]
pub extern "C" fn AcpiOsGetTimer() -> UINT64 {
	unimplemented!()
}

#[no_mangle]
pub extern "C" fn AcpiOsSignal(function: UINT32, info: *mut c_void) -> ACPI_STATUS {
	unimplemented!()
}

#[no_mangle]
pub extern "C" fn AcpiOsEnterSleep(sleep_state: UINT8, reg_a: UINT32, reg_b: UINT32) -> ACPI_STATUS {
	unimplemented!()
}

/*
 * Debug print routines
 */
#[no_mangle]
pub unsafe extern "C" fn AcpiOsPrintf(format: *const c_char, #[allow(unused_mut)] mut args: ...) {
	let mut len = 0;
	while *format.offset(len as isize) != 0x0 {
		len += 1;
	}
	
	let msg_slice = core::slice::from_raw_parts(format as *const u8, len);
	
	crate::tty::write_tty(msg_slice);
//	unimplemented!()
}

#[no_mangle]
pub extern "C" fn AcpiOsVprintf(format: *const c_char, args: *const va_list) {
	unsafe {
		AcpiOsPrintf(format);
	}
}

#[no_mangle]
pub extern "C" fn AcpiOsRedirectOutput(dest: *const c_void) {
	unimplemented!()
}

/*
 * Debug IO
 */
#[no_mangle]
pub extern "C" fn AcpiOsGetLine(buf: *mut char, buf_len: UINT32, bytes_read: &mut UINT32) -> ACPI_STATUS {
	unimplemented!()
}

#[no_mangle]
pub extern "C" fn AcpiOsInitializeDebugger() -> ACPI_STATUS {
	unimplemented!()
}

#[no_mangle]
pub extern "C" fn AcpiOsTerminateDebugger() {
	unimplemented!()
}

#[no_mangle]
pub extern "C" fn AcpiOsWaitCommandReady() -> ACPI_STATUS {
	unimplemented!()
}

#[no_mangle]
pub extern "C" fn AcpiOsNotifyCommandComplete() -> ACPI_STATUS {
	unimplemented!()
}

#[no_mangle]
pub extern "C" fn AcpiOsTracePoint(event_type: ACPI_TRACE_EVENT_TYPE, begin: BOOLEAN, aml: *const UINT8, pathname: *const c_char) {
	unimplemented!()
}

/*
 * Obtain ACPI table(s)
 */
#[no_mangle]
pub extern "C" fn AcpiOsGetTableByName(signature: *const c_char, instance: UINT32, out_table: &mut *mut ACPI_TABLE_HEADER, out_addr: &mut *const ACPI_PHYSICAL_ADDRESS) -> ACPI_STATUS {
	unimplemented!()
}

#[no_mangle]
pub extern "C" fn AcpiOsGetTableByIndex(index: UINT32, out_table: &mut *mut ACPI_TABLE_HEADER, instance: UINT32, address: *mut ACPI_PHYSICAL_ADDRESS) -> ACPI_STATUS {
	unimplemented!()
}

#[no_mangle]
pub extern "C" fn AcpiOsGetTableByAddress(address: ACPI_PHYSICAL_ADDRESS, out_table: &mut *mut ACPI_TABLE_HEADER) -> ACPI_STATUS {
	unimplemented!()
}

/*
 * Directory manipulation
 */
#[no_mangle]
pub extern "C" fn AcpiOsOpenDirectory(pathname: *const c_char, wildcard_spec: *const c_char, requested_file_type: c_char) -> *mut c_void {
	unimplemented!()
}

#[no_mangle]
pub extern "C" fn AcpiOsGetNextFilename(dir_handle: *mut c_void) -> *const c_char {
	unimplemented!()
}

#[no_mangle]
pub extern "C" fn AcpiOsCloseDirectory(dir_handle: *mut c_void) {
	unimplemented!()
}
