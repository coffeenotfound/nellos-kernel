use core::{mem, ptr};
use core::ptr::NonNull;
use core::sync::atomic::{AtomicUsize, Ordering::*};

use fallo::alloc::{AllocError, BareAlloc, FallibleAlloc};
use fallo::stdalloc::Layout;
use uefi_rs::table::boot::MemoryType;

type AllocPoolFn = extern "efiapi" fn(pool_type: MemoryType, size: usize, buffer: *mut *mut u8) -> uefi_rs::Status;
type FreePoolFn = extern "efiapi" fn(buffer: *mut u8) -> uefi_rs::Status;

static BOOT_ALLOC_POOL_FN: AtomicUsize = AtomicUsize::new(0x0);
static BOOT_FREE_POOL_FN: AtomicUsize = AtomicUsize::new(0x0);

pub fn init_boot_alloc(boot_services: &uefi_rs::table::boot::BootServices) {
	unsafe {
		BOOT_ALLOC_POOL_FN.store(mem::transmute::<_, &table::RawBootServicesTable>(boot_services).allocate_pool as usize, SeqCst);
		BOOT_FREE_POOL_FN.store(mem::transmute::<_, &table::RawBootServicesTable>(boot_services).allocate_pool as usize, SeqCst);
	}
}

pub unsafe fn deinit_boot_alloc() {
	BOOT_ALLOC_POOL_FN.store(0x0, SeqCst);
	BOOT_FREE_POOL_FN.store(0x0, SeqCst);
}

pub struct UefiBootAlloc;

impl FallibleAlloc for UefiBootAlloc {
	type Error = AllocError;
	
	fn alloc(&mut self, layout: Layout) -> Result<NonNull<[u8]>, Self::Error> {
		let p = BOOT_ALLOC_POOL_FN.load(SeqCst) as *const ();
		if !p.is_null() {
			let mut buf_ptr = ptr::null_mut::<u8>();
			unsafe {
				if (mem::transmute::<_, AllocPoolFn>(p))(MemoryType::LOADER_DATA, layout.size(), &mut buf_ptr).is_success() {
					return Ok(NonNull::slice_from_raw_parts(NonNull::new_unchecked(buf_ptr), layout.size()));
				}
			}
		}
		Err(AllocError)
	}
	
	unsafe fn dealloc(&mut self, ptr: NonNull<u8>, _layout: Layout) {
		let p = BOOT_FREE_POOL_FN.load(SeqCst) as *const ();
		if !p.is_null() {
			#[allow(unused_unsafe)]
			unsafe {
				let _ = mem::transmute::<_, FreePoolFn>(p)(ptr.as_ptr());
			}
		}
	}
}

impl BareAlloc for UefiBootAlloc {
	const INIT: Self = UefiBootAlloc;
}

// TODO: This may not be correct, check fallo for notes on what the semantic of Clone on a FallibleAlloc is
impl Clone for UefiBootAlloc {
	fn clone(&self) -> Self {
		UefiBootAlloc
	}
}

mod table {
	use core::ffi::c_void;
	
	use uefi_rs::{Event, Guid, Handle, Status};
	use uefi_rs::proto::loaded_image::DevicePath;
	use uefi_rs::table::boot::{EventType, MemoryDescriptor, MemoryMapKey, MemoryType, Tpl};
	use uefi_rs::table::Header;
	
	#[repr(C)]
	pub struct RawBootServicesTable {
		pub header: Header,
		
		// Task Priority services
		pub raise_tpl: unsafe extern "efiapi" fn(new_tpl: Tpl) -> Tpl,
		pub restore_tpl: unsafe extern "efiapi" fn(old_tpl: Tpl),
		
		// Memory allocation functions
		pub allocate_pages: extern "efiapi" fn(
			alloc_ty: u32,
			mem_ty: MemoryType,
			count: usize,
			addr: &mut u64,
		) -> Status,
		pub free_pages: extern "efiapi" fn(addr: u64, pages: usize) -> Status,
		pub get_memory_map: unsafe extern "efiapi" fn(
			size: &mut usize,
			map: *mut MemoryDescriptor,
			key: &mut MemoryMapKey,
			desc_size: &mut usize,
			desc_version: &mut u32,
		) -> Status,
		pub allocate_pool: extern "efiapi" fn(pool_type: MemoryType, size: usize, buffer: &mut *mut u8) -> Status,
		pub free_pool: extern "efiapi" fn(buffer: *mut u8) -> Status,
		
		// Event & timer functions
		pub create_event: unsafe extern "efiapi" fn(
			ty: EventType,
			notify_tpl: Tpl,
			notify_func: Option<EventNotifyFn>,
			notify_ctx: *mut c_void,
			event: *mut Event,
		) -> Status,
		pub set_timer: unsafe extern "efiapi" fn(event: Event, ty: u32, trigger_time: u64) -> Status,
		pub wait_for_event: unsafe extern "efiapi" fn(
			number_of_events: usize,
			events: *mut Event,
			out_index: *mut usize,
		) -> Status,
		pub signal_event: usize,
		pub close_event: usize,
		pub check_event: usize,
		
		// Protocol handlers
		pub install_protocol_interface: usize,
		pub reinstall_protocol_interface: usize,
		pub uninstall_protocol_interface: usize,
		pub handle_protocol: extern "efiapi" fn(handle: Handle, proto: &Guid, out_proto: &mut *mut c_void) -> Status,
		pub _reserved: usize,
		pub register_protocol_notify: usize,
		pub locate_handle: unsafe extern "efiapi" fn(
			search_ty: i32,
			proto: *const Guid,
			key: *mut c_void,
			buf_sz: &mut usize,
			buf: *mut Handle,
		) -> Status,
		pub locate_device_path: unsafe extern "efiapi" fn(
			proto: &Guid,
			device_path: &mut *mut DevicePath,
			out_handle: *mut Handle,
		) -> Status,
		pub install_configuration_table: usize,
		
		// Image services
		pub load_image: usize,
		pub start_image: usize,
		pub exit: usize,
		pub unload_image: usize,
		pub exit_boot_services: unsafe extern "efiapi" fn(image_handle: Handle, map_key: MemoryMapKey) -> Status,
		
		// Misc services
		pub get_next_monotonic_count: usize,
		pub stall: extern "efiapi" fn(microseconds: usize) -> Status,
		pub set_watchdog_timer: unsafe extern "efiapi" fn(
			timeout: usize,
			watchdog_code: u64,
			data_size: usize,
			watchdog_data: *const u16,
		) -> Status,
		
		// Driver support services
		pub connect_controller: usize,
		pub disconnect_controller: usize,
		
		// Protocol open / close services
		pub open_protocol: usize,
		pub close_protocol: usize,
		pub open_protocol_information: usize,
		
		// Library services
		pub protocols_per_handle: usize,
		pub locate_handle_buffer: usize,
		pub locate_protocol: extern "efiapi" fn(
			proto: &Guid,
			registration: *mut c_void,
			out_proto: &mut *mut c_void,
		) -> Status,
		pub install_multiple_protocol_interfaces: usize,
		pub uninstall_multiple_protocol_interfaces: usize,
		
		// CRC services
		pub calculate_crc32: usize,
		
		// Misc services
		pub copy_mem: unsafe extern "efiapi" fn(dest: *mut u8, src: *const u8, len: usize),
		pub set_mem: unsafe extern "efiapi" fn(buffer: *mut u8, len: usize, value: u8),
		
		// New event functions (UEFI 2.0 or newer)
		pub create_event_ex: usize,
	}
	
	pub type EventNotifyFn = unsafe extern "efiapi" fn(event: Event, context: *mut c_void);
}
