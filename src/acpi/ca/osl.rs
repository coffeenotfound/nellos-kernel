#![allow(non_snake_case)]
#![allow(unused_variables)] // TODO: Only for now

use cty::{c_char, c_void};

use acpica_sys::*;

/*
 * OSL Initialization and shutdown primitives
 */
#[no_mangle]
pub extern "C" fn AcpiOsInitialize() -> ACPI_STATUS {
//	unimplemented!()
	AE_OK
}

#[no_mangle]
pub extern "C" fn AcpiOsTerminate() -> ACPI_STATUS {
	unimplemented!()
}

/*
 * ACPI Table interfaces
 */
#[no_mangle]
pub extern "C" fn AcpiOsGetRootPointer() -> ACPI_PHYSICAL_ADDRESS {
	unimplemented!()
}

#[no_mangle]
pub extern "C" fn AcpiOsPredefinedOverride(init_val: *const ACPI_PREDEFINED_NAMES, new_val: &mut ACPI_STRING) -> ACPI_STATUS {
	unimplemented!()
}

#[no_mangle]
pub extern "C" fn AcpiOsTableOverride(existing_table: *const ACPI_TABLE_HEADER, new_table: &mut *mut ACPI_TABLE_HEADER) -> ACPI_STATUS {
	unimplemented!()
}

#[no_mangle]
pub extern "C" fn AcpiOsPhysicalTableOverride(existing_table: *const ACPI_TABLE_HEADER, new_address: &mut ACPI_PHYSICAL_ADDRESS, new_table_length: &mut UINT32) -> ACPI_STATUS {
	unimplemented!()
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
#[no_mangle]
pub extern "C" fn AcpiOsInstallInterruptHandler(interrupt_nr: UINT32, service_routine: ACPI_OSD_HANDLER, context: *mut c_void) -> ACPI_STATUS {
	unimplemented!()
}

#[no_mangle]
pub extern "C" fn AcpiOsRemoveInterruptHandler(interrupt_nr: UINT32, service_routine: ACPI_OSD_HANDLER) -> ACPI_STATUS {
	unimplemented!()
}

/*
 * Threads and Scheduling
 */
#[no_mangle]
pub extern "C" fn AcpiOsGetThreadId() -> ACPI_THREAD_ID {
	unimplemented!()
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
	// This implementation is a damn joke but both osunixxf and oswinxf do it this way.
	// I would do it a whole lot better but I don't think you even *can* read physical
	// memory without mapping it first
	// TODO: If we do decide to identity map all of the physical memory into the kernel
	//  virtual addr space (and we probably will) we actually *could* (and should)
	//  implement this properly
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
pub unsafe extern "C" fn AcpiOsPrintf(format: *const c_char, #[allow(unused_mut)] mut args: ...) -> ACPI_STATUS {
	unimplemented!()
}

#[no_mangle]
pub extern "C" fn AcpiOsVprintf(format: *const c_char, args: *const va_list) {
	unimplemented!()
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
