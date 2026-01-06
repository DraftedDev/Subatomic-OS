/// Represents the PCI Base Class field.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Class {
    /// Unclassified device
    Unclassified(u8),
    /// Mass Storage Controller
    MassStorageController(MassStorageSubClass),
    /// Network Controller
    NetworkController(NetworkSubClass),
    /// Display Controller
    DisplayController(DisplaySubClass),
    /// Multimedia Controller
    MultimediaController(MultimediaSubClass),
    /// Memory Controller
    MemoryController(MemorySubClass),
    /// Bridge Device
    BridgeDevice(BridgeDeviceSubClass),
    /// Simple Communication Controller
    SimpleCommunicationController(SimpleCommunicationSubClass),
    /// Base System Peripheral
    BaseSystemPeripheral(BaseSystemPeripheralSubClass),
    /// Input Device Controller
    InputDeviceController(InputDeviceSubClass),
    /// Docking Station
    DockingStation(DockingStationSubClass),
    /// Processor
    Processor(ProcessorSubClass),
    /// Serial Bus Controller
    SerialBusController(SerialBusSubClass),
    /// Wireless Controller
    WirelessController(WirelessSubClass),
    /// Intelligent I/O Controller
    IntelligentIOController(IntelligentIOSubClass),
    /// Satellite Communication Controller
    SatelliteCommunicationController(SatelliteCommunicationSubClass),
    /// Encryption/Decryption Controller
    EncryptionController(EncryptionSubClass),
    /// Signal Processing Controller
    SignalProcessingController(SignalProcessingSubClass),
    /// Processing Accelerator
    ProcessingAccelerator(ProcessingAcceleratorSubClass),
    /// Non-Essential Instrumentation
    NonEssentialInstrumentation(NonEssentialInstrumentationSubClass),
    /// Unknown or vendor-specific class
    Unknown { class: u8, subclass: u8 },
}

impl Class {
    /// Convert from a raw u8 to a [BaseClass].
    pub fn from_u8(val: u8, sub: u8) -> Self {
        match val {
            0x00 => Self::Unclassified(sub),
            0x01 => Self::MassStorageController(MassStorageSubClass::from_u8(sub)),
            0x02 => Self::NetworkController(NetworkSubClass::from_u8(sub)),
            0x03 => Self::DisplayController(DisplaySubClass::from_u8(sub)),
            0x04 => Self::MultimediaController(MultimediaSubClass::from_u8(sub)),
            0x05 => Self::MemoryController(MemorySubClass::from_u8(sub)),
            0x06 => Self::BridgeDevice(BridgeDeviceSubClass::from_u8(sub)),
            0x07 => Self::SimpleCommunicationController(SimpleCommunicationSubClass::from_u8(sub)),
            0x08 => Self::BaseSystemPeripheral(BaseSystemPeripheralSubClass::from_u8(sub)),
            0x09 => Self::InputDeviceController(InputDeviceSubClass::from_u8(sub)),
            0x0A => Self::DockingStation(DockingStationSubClass::from_u8(sub)),
            0x0B => Self::Processor(ProcessorSubClass::from_u8(sub)),
            0x0C => Self::SerialBusController(SerialBusSubClass::from_u8(sub)),
            0x0D => Self::WirelessController(WirelessSubClass::from_u8(sub)),
            0x0E => Self::IntelligentIOController(IntelligentIOSubClass::from_u8(sub)),
            0x0F => {
                Self::SatelliteCommunicationController(SatelliteCommunicationSubClass::from_u8(sub))
            }
            0x10 => Self::EncryptionController(EncryptionSubClass::from_u8(sub)),
            0x11 => Self::SignalProcessingController(SignalProcessingSubClass::from_u8(sub)),
            0x12 => Self::ProcessingAccelerator(ProcessingAcceleratorSubClass::from_u8(sub)),
            0x13 => {
                Self::NonEssentialInstrumentation(NonEssentialInstrumentationSubClass::from_u8(sub))
            }
            other => Self::Unknown {
                class: other,
                subclass: sub,
            },
        }
    }

    /// Convert a [BaseClass] to its raw u8 value.
    pub fn to_u8(&self) -> u8 {
        match *self {
            Self::Unclassified(_) => 0x00,
            Self::MassStorageController(_) => 0x01,
            Self::NetworkController(_) => 0x02,
            Self::DisplayController(_) => 0x03,
            Self::MultimediaController(_) => 0x04,
            Self::MemoryController(_) => 0x05,
            Self::BridgeDevice(_) => 0x06,
            Self::SimpleCommunicationController(_) => 0x07,
            Self::BaseSystemPeripheral(_) => 0x08,
            Self::InputDeviceController(_) => 0x09,
            Self::DockingStation(_) => 0x0A,
            Self::Processor(_) => 0x0B,
            Self::SerialBusController(_) => 0x0C,
            Self::WirelessController(_) => 0x0D,
            Self::IntelligentIOController(_) => 0x0E,
            Self::SatelliteCommunicationController(_) => 0x0F,
            Self::EncryptionController(_) => 0x10,
            Self::SignalProcessingController(_) => 0x11,
            Self::ProcessingAccelerator(_) => 0x12,
            Self::NonEssentialInstrumentation(_) => 0x13,
            Self::Unknown { class: val, .. } => val,
        }
    }
}

/// PCI Subclasses for Mass Storage Controller (BaseClass 0x01)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MassStorageSubClass {
    SCSI,
    IDE,
    Floppy,
    IPI,
    RAID,
    ATA,
    SATA,
    SAS,
    NonVolatileMemory,
    Other,
    Unknown(u8),
}

impl MassStorageSubClass {
    pub fn from_u8(val: u8) -> Self {
        match val {
            0x00 => Self::SCSI,
            0x01 => Self::IDE,
            0x02 => Self::Floppy,
            0x03 => Self::IPI,
            0x04 => Self::RAID,
            0x05 => Self::ATA,
            0x06 => Self::SATA,
            0x07 => Self::SAS,
            0x08 => Self::NonVolatileMemory,
            0x80 => Self::Other,
            other => Self::Unknown(other),
        }
    }

    pub fn to_u8(&self) -> u8 {
        match *self {
            Self::SCSI => 0x00,
            Self::IDE => 0x01,
            Self::Floppy => 0x02,
            Self::IPI => 0x03,
            Self::RAID => 0x04,
            Self::ATA => 0x05,
            Self::SATA => 0x06,
            Self::SAS => 0x07,
            Self::NonVolatileMemory => 0x08,
            Self::Other => 0x80,
            Self::Unknown(val) => val,
        }
    }
}

/// PCI Subclasses for Network Controller (BaseClass 0x02)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NetworkSubClass {
    Ethernet,
    TokenRing,
    FDDI,
    ATM,
    ISDN,
    WorldFip,
    PICMG,
    Infiniband,
    Other,
    Unknown(u8),
}

impl NetworkSubClass {
    pub fn from_u8(val: u8) -> Self {
        match val {
            0x00 => Self::Ethernet,
            0x01 => Self::TokenRing,
            0x02 => Self::FDDI,
            0x03 => Self::ATM,
            0x04 => Self::ISDN,
            0x05 => Self::WorldFip,
            0x06 => Self::PICMG,
            0x07 => Self::Infiniband,
            0x80 => Self::Other,
            other => Self::Unknown(other),
        }
    }

    pub fn to_u8(&self) -> u8 {
        match *self {
            Self::Ethernet => 0x00,
            Self::TokenRing => 0x01,
            Self::FDDI => 0x02,
            Self::ATM => 0x03,
            Self::ISDN => 0x04,
            Self::WorldFip => 0x05,
            Self::PICMG => 0x06,
            Self::Infiniband => 0x07,
            Self::Other => 0x80,
            Self::Unknown(val) => val,
        }
    }
}

/// PCI Subclasses for Display Controller (BaseClass 0x03)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DisplaySubClass {
    VGACompatible,
    XGA,
    _3DController,
    Other,
    Unknown(u8),
}

impl DisplaySubClass {
    pub fn from_u8(val: u8) -> Self {
        match val {
            0x00 => Self::VGACompatible,
            0x01 => Self::XGA,
            0x02 => Self::_3DController,
            0x80 => Self::Other,
            other => Self::Unknown(other),
        }
    }

    pub fn to_u8(&self) -> u8 {
        match *self {
            Self::VGACompatible => 0x00,
            Self::XGA => 0x01,
            Self::_3DController => 0x02,
            Self::Other => 0x80,
            Self::Unknown(val) => val,
        }
    }
}

/// PCI Subclasses for Multimedia Controller (BaseClass 0x04)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MultimediaSubClass {
    Video,
    Audio,
    ComputerPeripherals,
    Other,
    Unknown(u8),
}

impl MultimediaSubClass {
    pub fn from_u8(val: u8) -> Self {
        match val {
            0x00 => Self::Video,
            0x01 => Self::Audio,
            0x02 => Self::ComputerPeripherals,
            0x80 => Self::Other,
            other => Self::Unknown(other),
        }
    }

    pub fn to_u8(&self) -> u8 {
        match *self {
            Self::Video => 0x00,
            Self::Audio => 0x01,
            Self::ComputerPeripherals => 0x02,
            Self::Other => 0x80,
            Self::Unknown(val) => val,
        }
    }
}

/// PCI Subclasses for Memory Controller (BaseClass 0x05)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MemorySubClass {
    RAMController,
    FlashController,
    Other,
    Unknown(u8),
}

impl MemorySubClass {
    pub fn from_u8(val: u8) -> Self {
        match val {
            0x00 => Self::RAMController,
            0x01 => Self::FlashController,
            0x80 => Self::Other,
            other => Self::Unknown(other),
        }
    }

    pub fn to_u8(&self) -> u8 {
        match *self {
            Self::RAMController => 0x00,
            Self::FlashController => 0x01,
            Self::Other => 0x80,
            Self::Unknown(val) => val,
        }
    }
}

/// PCI Subclasses for Bridge Device (BaseClass 0x06)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BridgeDeviceSubClass {
    Host,
    ISA,
    EISA,
    MicroChannel,
    PCItoPCI,
    PCMCIA,
    NuBus,
    CardBus,
    RACEway,
    Other,
    Unknown(u8),
}

impl BridgeDeviceSubClass {
    pub fn from_u8(val: u8) -> Self {
        match val {
            0x00 => Self::Host,
            0x01 => Self::ISA,
            0x02 => Self::EISA,
            0x03 => Self::MicroChannel,
            0x04 => Self::PCItoPCI,
            0x05 => Self::PCMCIA,
            0x06 => Self::NuBus,
            0x07 => Self::CardBus,
            0x08 => Self::RACEway,
            0x80 => Self::Other,
            other => Self::Unknown(other),
        }
    }

    pub fn to_u8(&self) -> u8 {
        match *self {
            Self::Host => 0x00,
            Self::ISA => 0x01,
            Self::EISA => 0x02,
            Self::MicroChannel => 0x03,
            Self::PCItoPCI => 0x04,
            Self::PCMCIA => 0x05,
            Self::NuBus => 0x06,
            Self::CardBus => 0x07,
            Self::RACEway => 0x08,
            Self::Other => 0x80,
            Self::Unknown(val) => val,
        }
    }
}

/// PCI Subclasses for Simple Communication Controller (BaseClass 0x07)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SimpleCommunicationSubClass {
    Serial,
    Parallel,
    MultiPortSerial,
    Modem,
    GPIB,
    SmartCard,
    Other,
    Unknown(u8),
}

impl SimpleCommunicationSubClass {
    pub fn from_u8(val: u8) -> Self {
        match val {
            0x00 => Self::Serial,
            0x01 => Self::Parallel,
            0x02 => Self::MultiPortSerial,
            0x03 => Self::Modem,
            0x04 => Self::GPIB,
            0x05 => Self::SmartCard,
            0x80 => Self::Other,
            other => Self::Unknown(other),
        }
    }

    pub fn to_u8(&self) -> u8 {
        match *self {
            Self::Serial => 0x00,
            Self::Parallel => 0x01,
            Self::MultiPortSerial => 0x02,
            Self::Modem => 0x03,
            Self::GPIB => 0x04,
            Self::SmartCard => 0x05,
            Self::Other => 0x80,
            Self::Unknown(val) => val,
        }
    }
}

/// PCI Subclasses for Base System Peripheral (BaseClass 0x0B)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BaseSystemPeripheralSubClass {
    PIC,
    DMAController,
    Timer,
    RTC,
    PCIHotPlug,
    SDHostController,
    Other,
    Unknown(u8),
}

impl BaseSystemPeripheralSubClass {
    pub fn from_u8(val: u8) -> Self {
        match val {
            0x00 => Self::PIC,
            0x01 => Self::DMAController,
            0x02 => Self::Timer,
            0x03 => Self::RTC,
            0x04 => Self::PCIHotPlug,
            0x05 => Self::SDHostController,
            0x80 => Self::Other,
            other => Self::Unknown(other),
        }
    }

    pub fn to_u8(&self) -> u8 {
        match *self {
            Self::PIC => 0x00,
            Self::DMAController => 0x01,
            Self::Timer => 0x02,
            Self::RTC => 0x03,
            Self::PCIHotPlug => 0x04,
            Self::SDHostController => 0x05,
            Self::Other => 0x80,
            Self::Unknown(val) => val,
        }
    }
}

/// PCI Subclasses for Input Device Controller (BaseClass 0x0D)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InputDeviceSubClass {
    KeyboardController,
    Digitizer,
    MouseController,
    Other,
    Unknown(u8),
}

impl InputDeviceSubClass {
    pub fn from_u8(val: u8) -> Self {
        match val {
            0x00 => Self::KeyboardController,
            0x01 => Self::Digitizer,
            0x02 => Self::MouseController,
            0x80 => Self::Other,
            other => Self::Unknown(other),
        }
    }

    pub fn to_u8(&self) -> u8 {
        match *self {
            Self::KeyboardController => 0x00,
            Self::Digitizer => 0x01,
            Self::MouseController => 0x02,
            Self::Other => 0x80,
            Self::Unknown(val) => val,
        }
    }
}

/// PCI Subclasses for Docking Station (BaseClass 0x0E)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DockingStationSubClass {
    Generic,
    Other,
    Unknown(u8),
}

impl DockingStationSubClass {
    pub fn from_u8(val: u8) -> Self {
        match val {
            0x00 => Self::Generic,
            0x80 => Self::Other,
            other => Self::Unknown(other),
        }
    }

    pub fn to_u8(&self) -> u8 {
        match *self {
            Self::Generic => 0x00,
            Self::Other => 0x80,
            Self::Unknown(val) => val,
        }
    }
}

/// PCI Subclasses for Processor (BaseClass 0x0B)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProcessorSubClass {
    I386,
    I486,
    Pentium,
    Alpha,
    PowerPC,
    MIPS,
    CoProcessor,
    Other,
    Unknown(u8),
}

impl ProcessorSubClass {
    pub fn from_u8(val: u8) -> Self {
        match val {
            0x00 => Self::I386,
            0x01 => Self::I486,
            0x02 => Self::Pentium,
            0x10 => Self::Alpha,
            0x20 => Self::PowerPC,
            0x30 => Self::MIPS,
            0x40 => Self::CoProcessor,
            0x80 => Self::Other,
            other => Self::Unknown(other),
        }
    }

    pub fn to_u8(&self) -> u8 {
        match *self {
            Self::I386 => 0x00,
            Self::I486 => 0x01,
            Self::Pentium => 0x02,
            Self::Alpha => 0x10,
            Self::PowerPC => 0x20,
            Self::MIPS => 0x30,
            Self::CoProcessor => 0x40,
            Self::Other => 0x80,
            Self::Unknown(val) => val,
        }
    }
}

/// PCI Subclasses for Serial Bus Controller (BaseClass 0x0C)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SerialBusSubClass {
    FireWire,
    AccessBus,
    SSA,
    USB,
    FibreChannel,
    SMBus,
    InfiniBand,
    IPMI,
    Other,
    Unknown(u8),
}

impl SerialBusSubClass {
    pub fn from_u8(val: u8) -> Self {
        match val {
            0x00 => Self::FireWire,
            0x01 => Self::AccessBus,
            0x02 => Self::SSA,
            0x03 => Self::USB,
            0x04 => Self::FibreChannel,
            0x05 => Self::SMBus,
            0x06 => Self::InfiniBand,
            0x07 => Self::IPMI,
            0x80 => Self::Other,
            other => Self::Unknown(other),
        }
    }

    pub fn to_u8(&self) -> u8 {
        match *self {
            Self::FireWire => 0x00,
            Self::AccessBus => 0x01,
            Self::SSA => 0x02,
            Self::USB => 0x03,
            Self::FibreChannel => 0x04,
            Self::SMBus => 0x05,
            Self::InfiniBand => 0x06,
            Self::IPMI => 0x07,
            Self::Other => 0x80,
            Self::Unknown(val) => val,
        }
    }
}

/// PCI Subclasses for Wireless Controller (BaseClass 0x0D)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WirelessSubClass {
    IrDA,
    ConsumerIR,
    RF,
    Bluetooth,
    Other,
    Unknown(u8),
}

impl WirelessSubClass {
    pub fn from_u8(val: u8) -> Self {
        match val {
            0x00 => Self::IrDA,
            0x01 => Self::ConsumerIR,
            0x10 => Self::RF,
            0x11 => Self::Bluetooth,
            0x80 => Self::Other,
            other => Self::Unknown(other),
        }
    }

    pub fn to_u8(&self) -> u8 {
        match *self {
            Self::IrDA => 0x00,
            Self::ConsumerIR => 0x01,
            Self::RF => 0x10,
            Self::Bluetooth => 0x11,
            Self::Other => 0x80,
            Self::Unknown(val) => val,
        }
    }
}

/// PCI Subclasses for Intelligent I/O Controller (BaseClass 0x0E)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IntelligentIOSubClass {
    I2O,
    Other,
    Unknown(u8),
}

impl IntelligentIOSubClass {
    pub fn from_u8(val: u8) -> Self {
        match val {
            0x00 => Self::I2O,
            0x80 => Self::Other,
            other => Self::Unknown(other),
        }
    }

    pub fn to_u8(&self) -> u8 {
        match *self {
            Self::I2O => 0x00,
            Self::Other => 0x80,
            Self::Unknown(val) => val,
        }
    }
}

/// PCI Subclasses for Satellite Communication Controller (BaseClass X)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SatelliteCommunicationSubClass {
    Generic,
    Unknown(u8),
}

impl SatelliteCommunicationSubClass {
    pub fn from_u8(val: u8) -> Self {
        match val {
            0x00 => Self::Generic,
            other => Self::Unknown(other),
        }
    }

    pub fn to_u8(&self) -> u8 {
        match *self {
            Self::Generic => 0x00,
            Self::Unknown(val) => val,
        }
    }
}

/// PCI Subclasses for Encryption/Decryption Controller (BaseClass X)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EncryptionSubClass {
    Generic,
    Unknown(u8),
}

impl EncryptionSubClass {
    pub fn from_u8(val: u8) -> Self {
        match val {
            0x00 => Self::Generic,
            other => Self::Unknown(other),
        }
    }

    pub fn to_u8(&self) -> u8 {
        match *self {
            Self::Generic => 0x00,
            Self::Unknown(val) => val,
        }
    }
}

/// PCI Subclasses for Signal Processing Controller (BaseClass X)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SignalProcessingSubClass {
    Generic,
    Unknown(u8),
}

impl SignalProcessingSubClass {
    pub fn from_u8(val: u8) -> Self {
        match val {
            0x00 => Self::Generic,
            other => Self::Unknown(other),
        }
    }

    pub fn to_u8(&self) -> u8 {
        match *self {
            Self::Generic => 0x00,
            Self::Unknown(val) => val,
        }
    }
}

/// PCI Subclasses for Processing Accelerator (BaseClass X)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProcessingAcceleratorSubClass {
    Generic,
    Unknown(u8),
}

impl ProcessingAcceleratorSubClass {
    pub fn from_u8(val: u8) -> Self {
        match val {
            0x00 => Self::Generic,
            other => Self::Unknown(other),
        }
    }

    pub fn to_u8(&self) -> u8 {
        match *self {
            Self::Generic => 0x00,
            Self::Unknown(val) => val,
        }
    }
}

/// PCI Subclasses for Non-Essential Instrumentation (BaseClass X)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NonEssentialInstrumentationSubClass {
    Generic,
    Unknown(u8),
}

impl NonEssentialInstrumentationSubClass {
    pub fn from_u8(val: u8) -> Self {
        match val {
            0x00 => Self::Generic,
            other => Self::Unknown(other),
        }
    }

    pub fn to_u8(&self) -> u8 {
        match *self {
            Self::Generic => 0x00,
            Self::Unknown(val) => val,
        }
    }
}
