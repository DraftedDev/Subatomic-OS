use limine::paging::PagingMode;
use limine::request::{
    BootloaderInfoRequest, BootloaderInfoResponse, DateAtBootRequest, DateAtBootResponse,
    FramebufferRequest, FramebufferResponse, HhdmRepsonse, HhdmRequest, MemmapRequest,
    MemmapResponse, ModulesRequest, ModulesResponse, MpRequest, MpResponse, PagingModeRequest,
    PagingModeResponse, RsdpRequest, RsdpResponse,
};
use limine::{BaseRevision, RequestsEndMarker, RequestsStartMarker};

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
static mut FRAMEBUFFER_REQUEST: FramebufferRequest = FramebufferRequest::new();

/// Request rsdp info from limine.
#[used]
#[unsafe(link_section = ".requests")]
static RSDP_REQUEST: RsdpRequest = RsdpRequest::new();

/// Request boot date from limine.
#[used]
#[unsafe(link_section = ".requests")]
static BOOT_DATE_REQUEST: DateAtBootRequest = DateAtBootRequest::new();

/// Request bootloader info from limine.
#[used]
#[unsafe(link_section = ".requests")]
static BOOTLOADER_INFO_REQUEST: BootloaderInfoRequest = BootloaderInfoRequest::new();

/// Request paging mode from limine.
#[used]
#[unsafe(link_section = ".requests")]
static PAGING_REQUEST: PagingModeRequest = PagingModeRequest::new_exact(PagingMode::X86_64_4LVL);

/// Request memory map info from limine.
#[used]
#[unsafe(link_section = ".requests")]
static MEMORY_REQUEST: MemmapRequest = MemmapRequest::new();

/// Request HHDM info from limine.
#[used]
#[unsafe(link_section = ".requests")]
static HHDM_REQUEST: HhdmRequest = HhdmRequest::new();

/// Request multi-processor info from limine.
#[used]
#[unsafe(link_section = ".requests")]
static MP_REQUEST: MpRequest = MpRequest::new(0);

/// Request modules from limine.
#[used]
#[unsafe(link_section = ".requests")]
static MODULES_REQUEST: ModulesRequest = ModulesRequest::new();

/// The end marker for Limine requests.
#[used]
#[unsafe(link_section = ".requests_end_marker")]
static _END_MARKER: RequestsEndMarker = RequestsEndMarker::new();

/// Returns the [FramebufferResponse].
pub fn framebuffer<'a>() -> &'a FramebufferResponse {
    unsafe {
        FRAMEBUFFER_REQUEST
            .response()
            .expect("Failed to get framebuffer response")
    }
}

/// Returns the [RsdpResponse].
pub fn rsdp<'a>() -> &'a RsdpResponse {
    RSDP_REQUEST
        .response()
        .expect("Failed to get rsdp response")
}

/// Returns the [DateAtBootResponse].
pub fn boot_date<'a>() -> &'a DateAtBootResponse {
    BOOT_DATE_REQUEST
        .response()
        .expect("Failed to get boot date response")
}

/// Returns the [BootloaderInfoResponse].
pub fn bootloader_info<'a>() -> &'a BootloaderInfoResponse {
    BOOTLOADER_INFO_REQUEST
        .response()
        .expect("Failed to get bootloader info response")
}

/// Returns the [PagingModeResponse].
pub fn paging<'a>() -> &'a PagingModeResponse {
    PAGING_REQUEST
        .response()
        .expect("Failed to get paging mode response")
}

/// Returns the [MemoryMapResponse].
pub fn memory_map<'a>() -> &'a MemmapResponse {
    MEMORY_REQUEST
        .response()
        .expect("Failed to get memory map response")
}

/// Returns the [HhdmResponse].
pub fn higher_half_dm<'a>() -> &'a HhdmRepsonse {
    HHDM_REQUEST
        .response()
        .expect("Failed to get hhdm response")
}

/// Returns the [MpResponse].
pub fn multi_processors<'a>() -> &'a MpResponse {
    MP_REQUEST.response().expect("Failed to get mp response")
}

/// Returns the [ModuleRequest] or [None] if no modules were found.
pub fn modules<'a>() -> Option<&'a ModulesResponse> {
    MODULES_REQUEST.response()
}
