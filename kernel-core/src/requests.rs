use limine::BaseRevision;
use limine::mp::RequestFlags;
use limine::paging::Mode;
use limine::request::{
    BootloaderInfoRequest, DateAtBootRequest, FramebufferRequest, HhdmRequest, MemoryMapRequest,
    MpRequest, PagingModeRequest, RequestsEndMarker, RequestsStartMarker, RsdpRequest,
};
use limine::response::{
    BootloaderInfoResponse, DateAtBootResponse, FramebufferResponse, HhdmResponse,
    MemoryMapResponse, MpResponse, PagingModeResponse, RsdpResponse,
};

/// The start marker for Limine requests.
#[used]
#[unsafe(link_section = ".requests_start_marker")]
static _START_MARKER: RequestsStartMarker = RequestsStartMarker::new();

/// The base revision of limine.
#[used]
#[unsafe(link_section = ".requests")]
pub static BASE_REVISION: BaseRevision = BaseRevision::with_revision(3);

/// Request framebuffer info from limine.
#[used]
#[unsafe(link_section = ".requests")]
static mut FRAMEBUFFER_REQUEST: FramebufferRequest = FramebufferRequest::with_revision(0);

/// Request rsdp info from limine.
#[used]
#[unsafe(link_section = ".requests")]
static RSDP_REQUEST: RsdpRequest = RsdpRequest::with_revision(0);

/// Request boot date from limine.
#[used]
#[unsafe(link_section = ".requests")]
static BOOT_DATE_REQUEST: DateAtBootRequest = DateAtBootRequest::with_revision(0);

/// Request bootloader info from limine.
#[used]
#[unsafe(link_section = ".requests")]
static BOOTLOADER_INFO_REQUEST: BootloaderInfoRequest = BootloaderInfoRequest::with_revision(0);

/// Request paging mode from limine.
#[used]
#[unsafe(link_section = ".requests")]
static PAGING_REQUEST: PagingModeRequest =
    PagingModeRequest::with_revision(1).with_mode(Mode::FOUR_LEVEL);

/// Request memory map info from limine.
#[used]
#[unsafe(link_section = ".requests")]
static MEMORY_REQUEST: MemoryMapRequest = MemoryMapRequest::with_revision(0);

/// Request HHDM info from limine.
#[used]
#[unsafe(link_section = ".requests")]
static HHDM_REQUEST: HhdmRequest = HhdmRequest::with_revision(0);

/// Request multi-processor info from limine.
#[used]
#[unsafe(link_section = ".requests")]
static MP_REQUEST: MpRequest = MpRequest::with_revision(0).with_flags(RequestFlags::empty());

/// The end marker for Limine requests.
#[used]
#[unsafe(link_section = ".requests_end_marker")]
static _END_MARKER: RequestsEndMarker = RequestsEndMarker::new();

/// Returns the [FramebufferResponse].
///
/// # Safety
///
/// This is unsafe, because [FRAMEBUFFER_REQUEST] is a static mutable.
pub unsafe fn framebuffer<'a>() -> &'a FramebufferResponse {
    unsafe {
        FRAMEBUFFER_REQUEST
            .get_response()
            .expect("Failed to get framebuffer response.")
    }
}

/// Returns the [FramebufferResponse].
///
/// # Safety
///
/// This is unsafe, because [FRAMEBUFFER_REQUEST] is a static mutable.
pub unsafe fn framebuffer_mut<'a>() -> &'a mut FramebufferResponse {
    unsafe {
        FRAMEBUFFER_REQUEST
            .get_response_mut()
            .expect("Failed to get framebuffer response.")
    }
}

/// Returns the [RsdpResponse].
pub fn rsdp<'a>() -> &'a RsdpResponse {
    RSDP_REQUEST
        .get_response()
        .expect("Failed to get rsdp response.")
}

/// Returns the [DateAtBootResponse].
pub fn boot_date<'a>() -> &'a DateAtBootResponse {
    BOOT_DATE_REQUEST
        .get_response()
        .expect("Failed to get boot date response.")
}

/// Returns the [BootloaderInfoResponse].
pub fn bootloader_info<'a>() -> &'a BootloaderInfoResponse {
    BOOTLOADER_INFO_REQUEST
        .get_response()
        .expect("Failed to get bootloader info response.")
}

/// Returns the [PagingModeResponse].
pub fn paging<'a>() -> &'a PagingModeResponse {
    PAGING_REQUEST
        .get_response()
        .expect("Failed to get paging mode response.")
}

/// Returns the [MemoryMapResponse].
pub fn memory_map<'a>() -> &'a MemoryMapResponse {
    MEMORY_REQUEST
        .get_response()
        .expect("Failed to get memory map response.")
}

/// Returns the [HhdmResponse].
pub fn higher_half_dm<'a>() -> &'a HhdmResponse {
    HHDM_REQUEST
        .get_response()
        .expect("Failed to get hhdm response.")
}

/// Returns the [MpResponse].
pub fn multi_processors<'a>() -> &'a MpResponse {
    MP_REQUEST
        .get_response()
        .expect("Failed to get mp response.")
}
