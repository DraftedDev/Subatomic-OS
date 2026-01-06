use crate::memory::mapper::{
    map_address_if_not_present, map_address_range, translate_phys_addr_unsafe,
};
use acpi::aml::AmlError;
use acpi::sdt::madt::Madt;
use acpi::sdt::mcfg::Mcfg;
use acpi::{Handle, Handler, HpetInfo, PciAddress, PhysicalMapping};
use core::ptr::NonNull;
use kernel_core::requests;
use kernel_core::sync::init::InitData;
use kernel_core::wrapper::SendSyncWrapper;
use x86_64::PhysAddr;
use x86_64::structures::paging::PageTableFlags;

const HPET_GENERAL_CAPABILITIES_OFFSET: u64 = 0x00;
const HPET_GENERAL_CONFIGURATION_OFFSET: u64 = 0x10;
const HPET_MAIN_COUNTER_OFFSET: u64 = 0xF0;

pub static ACPI: InitData<AcpiTables> = InitData::uninit();
pub static HPET_INFO: InitData<HpetInfo> = InitData::uninit();
pub static HPET_CLOCK_TICK_UNIT: InitData<u64> = InitData::uninit();

pub unsafe fn init() {
    unsafe {
        let rsdp = requests::rsdp().address();

        let tables =
            acpi::AcpiTables::from_rsdp(AcpiHandler, rsdp).expect("failed to find acpi tables.");

        let madt = tables.find_table::<Madt>().expect("Failed to find MADT");
        let mcfg = tables.find_table::<Mcfg>().expect("Failed to find MCfg");

        let acpi = ACPI.init(AcpiTables {
            tables,
            madt: SendSyncWrapper::new(madt),
            mcfg: SendSyncWrapper::new(mcfg),
        });

        // init HPET
        {
            let hpet =
                HPET_INFO.init(HpetInfo::new(&acpi.tables).expect("failed to find hpet info."));

            let hpet_base_phys = hpet.base_address;

            let hpet_base = map_address_if_not_present(
                PhysAddr::new(hpet_base_phys as u64),
                PageTableFlags::PRESENT
                    | PageTableFlags::WRITABLE
                    | PageTableFlags::NO_CACHE
                    | PageTableFlags::NO_EXECUTE,
            )
            .as_u64();

            // read the HPET General Capabilities Register to verify the HPET is present
            let capabilities: u64 =
                ((hpet_base + HPET_GENERAL_CAPABILITIES_OFFSET) as *const u64).read_volatile();

            // lower 32 bits of the capabilities register contain the period in femto seconds
            let clock_tick_unit = (capabilities >> 32) & 0xFFFFFFFF; // 32-bit period in femto seconds

            HPET_CLOCK_TICK_UNIT.init(clock_tick_unit);
        }
    }
}

pub fn read_hpet_counter() -> u64 {
    let hpet_base =
        unsafe { translate_phys_addr_unsafe(PhysAddr::new(HPET_INFO.get().base_address as u64)) }
            .as_u64();

    unsafe { ((hpet_base + HPET_MAIN_COUNTER_OFFSET) as *const u64).read_volatile() }
}

pub unsafe fn enable_hpet() {
    let hpet_base_phys = HPET_INFO.get().base_address;
    let hpet_base =
        unsafe { translate_phys_addr_unsafe(PhysAddr::new(hpet_base_phys as u64)) }.as_u64();

    let general_config = (hpet_base + HPET_GENERAL_CONFIGURATION_OFFSET) as *mut u64;
    let mut config_value = unsafe { general_config.read_volatile() };
    config_value |= 1; // Set bit 0 to enable the counter
    unsafe {
        general_config.write_volatile(config_value);
    }
}

pub unsafe fn disable_hpet() {
    let hpet_base_phys = HPET_INFO.get().base_address;
    let hpet_base =
        unsafe { translate_phys_addr_unsafe(PhysAddr::new(hpet_base_phys as u64)) }.as_u64();

    let general_config = (hpet_base + HPET_GENERAL_CONFIGURATION_OFFSET) as *mut u64;
    unsafe { general_config.write_volatile(0) }; // disable hpet
}

pub struct AcpiTables {
    pub tables: acpi::AcpiTables<AcpiHandler>,
    pub madt: SendSyncWrapper<PhysicalMapping<AcpiHandler, Madt>>,
    pub mcfg: SendSyncWrapper<PhysicalMapping<AcpiHandler, Mcfg>>,
}

#[derive(Copy, Clone, Debug)]
pub struct AcpiHandler;

impl Handler for AcpiHandler {
    unsafe fn map_physical_region<T>(
        &self,
        physical_address: usize,
        size: usize,
    ) -> PhysicalMapping<Self, T> {
        let mapped = unsafe {
            map_address_range(
                PhysAddr::new(physical_address as u64),
                size,
                PageTableFlags::PRESENT
                    | PageTableFlags::WRITABLE
                    | PageTableFlags::NO_CACHE
                    | PageTableFlags::NO_EXECUTE,
            )
        };

        PhysicalMapping {
            physical_start: physical_address,
            virtual_start: unsafe { NonNull::new_unchecked(mapped.as_mut_ptr()) },
            region_length: size,
            mapped_length: (size + 0xFFF) & !0xFFF, // rounded to full pages
            handler: self.clone(),
        }
    }

    fn unmap_physical_region<T>(_region: &PhysicalMapping<Self, T>) {
        // TODO: unsafe { mapper::unmap_address(VirtAddr::from_ptr(region.virtual_start.as_ptr())) } ?
    }

    fn read_u8(&self, address: usize) -> u8 {
        unsafe { core::ptr::read(address as *const u8) }
    }

    fn read_u16(&self, address: usize) -> u16 {
        unsafe { core::ptr::read(address as *const u16) }
    }

    fn read_u32(&self, address: usize) -> u32 {
        unsafe { core::ptr::read(address as *const u32) }
    }

    fn read_u64(&self, address: usize) -> u64 {
        unsafe { core::ptr::read(address as *const u64) }
    }

    fn write_u8(&self, address: usize, _value: u8) {
        unsafe { core::ptr::write(address as *mut u8, _value) }
    }

    fn write_u16(&self, address: usize, _value: u16) {
        unsafe { core::ptr::write(address as *mut u16, _value) }
    }

    fn write_u32(&self, address: usize, _value: u32) {
        unsafe { core::ptr::write(address as *mut u32, _value) }
    }

    fn write_u64(&self, address: usize, _value: u64) {
        unsafe { core::ptr::write(address as *mut u64, _value) }
    }

    fn read_io_u8(&self, _port: u16) -> u8 {
        todo!()
    }

    fn read_io_u16(&self, _port: u16) -> u16 {
        todo!()
    }

    fn read_io_u32(&self, _port: u16) -> u32 {
        todo!()
    }

    fn write_io_u8(&self, _port: u16, _value: u8) {
        todo!()
    }

    fn write_io_u16(&self, _port: u16, _value: u16) {
        todo!()
    }

    fn write_io_u32(&self, _port: u16, _value: u32) {
        todo!()
    }

    fn read_pci_u8(&self, _address: PciAddress, _offset: u16) -> u8 {
        todo!()
    }

    fn read_pci_u16(&self, _address: PciAddress, _offset: u16) -> u16 {
        todo!()
    }

    fn read_pci_u32(&self, _address: PciAddress, _offset: u16) -> u32 {
        todo!()
    }

    fn write_pci_u8(&self, _address: PciAddress, _offset: u16, _value: u8) {
        todo!()
    }

    fn write_pci_u16(&self, _address: PciAddress, _offset: u16, _value: u16) {
        todo!()
    }

    fn write_pci_u32(&self, _address: PciAddress, _offset: u16, _value: u32) {
        todo!()
    }

    fn nanos_since_boot(&self) -> u64 {
        requests::boot_date().timestamp().as_nanos() as u64
    }

    fn stall(&self, _microseconds: u64) {
        todo!()
    }

    fn sleep(&self, _milliseconds: u64) {
        todo!()
    }

    fn create_mutex(&self) -> Handle {
        todo!()
    }

    fn acquire(&self, _mutex: Handle, _timeout: u16) -> Result<(), AmlError> {
        todo!()
    }

    fn release(&self, _mutex: Handle) {
        todo!()
    }
}
