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
    Unknown {
        /// The device base class.
        class: u8,
        /// The device subclass.
        subclass: u8,
    },
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

/// PCI Subclasses for **Mass Storage Controller** (Base Class `0x01`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MassStorageSubClass {
    /// SCSI storage controller
    SCSI,
    /// IDE storage controller
    IDE,
    /// Floppy disk controller
    Floppy,
    /// IPI storage controller
    IPI,
    /// RAID controller
    RAID,
    /// ATA controller
    ATA,
    /// SATA controller
    SATA,
    /// SAS controller
    SAS,
    /// Non-Volatile Memory controller (e.g., NVMe)
    NonVolatileMemory,
    /// Other subclass defined by vendor or reserved
    Other,
    /// Unknown or unrecognized subclass
    Unknown(u8),
}

impl MassStorageSubClass {
    /// Converts a raw `u8` value from PCI config space into a [MassStorageSubClass] variant.
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

    /// Converts a [MassStorageSubClass] variant back into its raw `u8` representation.
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

/// PCI Subclasses for Network Controller (BaseClass `0x02`)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NetworkSubClass {
    /// Ethernet controller.
    Ethernet,
    /// Token ring controller.
    TokenRing,
    /// FDDI controller.
    FDDI,
    /// ATM controller.
    ATM,
    /// ISDN controller.
    ISDN,
    /// World Fip controller
    WorldFip,
    /// PICMG controller.
    PICMG,
    /// Infiniband controller.
    Infiniband,
    /// Other controller.
    Other,
    /// Unknown controller.
    Unknown(u8),
}

impl NetworkSubClass {
    /// Converts a [NetworkSubClass] variant back into its raw `u8` representation.
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

    /// Converts a [NetworkSubClass] variant back into its raw `u8` representation.
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
    /// VGA-Compatible controller.
    VGACompatible,
    /// XGA controller.
    XGA,
    /// 3D controller.
    _3DController,
    /// Other controller.
    Other,
    /// Unknown controller.
    Unknown(u8),
}

impl DisplaySubClass {
    /// Converts a [DisplaySubClass] variant back into its raw `u8` representation.
    pub fn from_u8(val: u8) -> Self {
        match val {
            0x00 => Self::VGACompatible,
            0x01 => Self::XGA,
            0x02 => Self::_3DController,
            0x80 => Self::Other,
            other => Self::Unknown(other),
        }
    }

    /// Converts a [DisplaySubClass] variant back into its raw `u8` representation.
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
    /// Video controller.
    Video,
    /// Audio controller.
    Audio,
    /// Computer peripherals controller.
    ComputerPeripherals,
    /// Other controller.
    Other,
    /// Unknown controller.
    Unknown(u8),
}

impl MultimediaSubClass {
    /// Converts a [MultimediaSubClass] variant back into its raw `u8` representation.
    pub fn from_u8(val: u8) -> Self {
        match val {
            0x00 => Self::Video,
            0x01 => Self::Audio,
            0x02 => Self::ComputerPeripherals,
            0x80 => Self::Other,
            other => Self::Unknown(other),
        }
    }

    /// Converts a [MultimediaSubClass] variant back into its raw `u8` representation.
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
    /// RAM controller.
    RAMController,
    /// Flash controller.
    FlashController,
    /// Other controller.
    Other,
    /// Unknown controller.
    Unknown(u8),
}

impl MemorySubClass {
    /// Converts a [MemorySubClass] variant back into its raw `u8` representation.
    pub fn from_u8(val: u8) -> Self {
        match val {
            0x00 => Self::RAMController,
            0x01 => Self::FlashController,
            0x80 => Self::Other,
            other => Self::Unknown(other),
        }
    }

    /// Converts a [MemorySubClass] variant back into its raw `u8` representation.
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
    /// Host bridge.
    Host,
    /// ISA bridge.
    ISA,
    /// EISA bridge.
    EISA,
    /// Micro Channel bridge.
    MicroChannel,
    /// PCI to PCI bridge.
    PCItoPCI,
    /// PCMCIA bridge.
    PCMCIA,
    /// NuBus bridge.
    NuBus,
    /// CardBus bridge.
    CardBus,
    /// RAID controller bridge.
    RACEway,
    /// Other bridge.
    Other,
    /// Unknown bridge.
    Unknown(u8),
}

impl BridgeDeviceSubClass {
    /// Converts a [BridgeDeviceSubClass] variant back into its raw `u8` representation.
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

    /// Converts a [BridgeDeviceSubClass] variant back into its raw `u8` representation.
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
    /// Serial controller.
    Serial,
    /// Parallel controller.
    Parallel,
    /// Multi-port serial controller.
    MultiPortSerial,
    /// Modem controller.
    Modem,
    /// GPIB controller.
    GPIB,
    /// Smart card controller.
    SmartCard,
    /// Other controller.
    Other,
    /// Unknown controller.
    Unknown(u8),
}

impl SimpleCommunicationSubClass {
    /// Converts a [SimpleCommunicationSubClass] variant back into its raw `u8` representation.
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

    /// Converts a [SimpleCommunicationSubClass] variant back into its raw `u8` representation.
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
    /// PIC peripheral.
    PIC,
    /// DMA peripheral.
    DMAController,
    /// Timer peripheral.
    Timer,
    /// RTC peripheral.
    RTC,
    /// PCI-Hot-Plug peripheral.
    PCIHotPlug,
    /// SD Host peripheral.
    SDHostController,
    /// Other peripheral.
    Other,
    /// Unknown peripheral.
    Unknown(u8),
}

impl BaseSystemPeripheralSubClass {
    /// Converts a [BaseSystemPeripheralSubClass] variant back into its raw `u8` representation.
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

    /// Converts a [BaseSystemPeripheralSubClass] variant back into its raw `u8` representation.
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
    /// Keyboard controller.
    KeyboardController,
    /// Digitizer controller.
    Digitizer,
    /// Mouse controller.
    MouseController,
    /// Other controller.
    Other,
    /// Unknown controller.
    Unknown(u8),
}

impl InputDeviceSubClass {
    /// Converts a [InputDeviceSubClass] variant back into its raw `u8` representation.
    pub fn from_u8(val: u8) -> Self {
        match val {
            0x00 => Self::KeyboardController,
            0x01 => Self::Digitizer,
            0x02 => Self::MouseController,
            0x80 => Self::Other,
            other => Self::Unknown(other),
        }
    }

    /// Converts a [InputDeviceSubClass] variant back into its raw `u8` representation.
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
    /// Generic docking station.
    Generic,
    /// Other docking station.
    Other,
    /// Unknown docking station.
    Unknown(u8),
}

impl DockingStationSubClass {
    /// Converts a [DockingStationSubClass] variant back into its raw `u8` representation.
    pub fn from_u8(val: u8) -> Self {
        match val {
            0x00 => Self::Generic,
            0x80 => Self::Other,
            other => Self::Unknown(other),
        }
    }

    /// Converts a [DockingStationSubClass] variant back into its raw `u8` representation.
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
    /// I386 processor.
    I386,
    /// I486 processor.
    I486,
    /// Pentium processor.
    Pentium,
    /// Alpha processor.
    Alpha,
    /// PowerPC processor.
    PowerPC,
    /// MIPS processor.
    MIPS,
    /// Co processor.
    CoProcessor,
    /// Other processor.
    Other,
    /// Unknown processor.
    Unknown(u8),
}

impl ProcessorSubClass {
    /// Converts a [ProcessorSubClass] variant back into its raw `u8` representation.
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

    /// Converts a [ProcessorSubClass] variant back into its raw `u8` representation.
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
    /// Fire wire controller.
    FireWire,
    /// Access bus controller.
    AccessBus,
    /// SSA controller.
    SSA,
    /// USB controller.
    USB,
    /// Fibre channel controller.
    FibreChannel,
    /// SM bus controller.
    SMBus,
    /// InfiniBand controller.
    InfiniBand,
    /// IPMI controller.
    IPMI,
    /// Other controller.
    Other,
    /// Unknown controller.
    Unknown(u8),
}

impl SerialBusSubClass {
    /// Converts a [SerialBusSubClass] variant back into its raw `u8` representation.
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

    /// Converts a [SerialBusSubClass] variant back into its raw `u8` representation.
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
    /// IrDA controller.
    IrDA,
    /// Consumer IR controller.
    ConsumerIR,
    /// RF controller.
    RF,
    /// Bluetooth controller.
    Bluetooth,
    /// Other controller.
    Other,
    /// Unknown controller.
    Unknown(u8),
}

impl WirelessSubClass {
    /// Converts a [WirelessSubClass] variant back into its raw `u8` representation.
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

    /// Converts a [WirelessSubClass] variant back into its raw `u8` representation.
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
    /// I20 controller.
    I2O,
    /// Other controller.
    Other,
    /// Unknown controller.
    Unknown(u8),
}

impl IntelligentIOSubClass {
    /// Converts a [IntelligentIOSubClass] variant back into its raw `u8` representation.
    pub fn from_u8(val: u8) -> Self {
        match val {
            0x00 => Self::I2O,
            0x80 => Self::Other,
            other => Self::Unknown(other),
        }
    }

    /// Converts a [IntelligentIOSubClass] variant back into its raw `u8` representation.
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
    /// Generic controller.
    Generic,
    /// Unknown controller.
    Unknown(u8),
}

impl SatelliteCommunicationSubClass {
    /// Converts a [SatelliteCommunicationSubClass] variant back into its raw `u8` representation.
    pub fn from_u8(val: u8) -> Self {
        match val {
            0x00 => Self::Generic,
            other => Self::Unknown(other),
        }
    }

    /// Converts a [SatelliteCommunicationSubClass] variant back into its raw `u8` representation.
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
    /// Generic controller.
    Generic,
    /// Unknown controller.
    Unknown(u8),
}

impl EncryptionSubClass {
    /// Converts a [EncryptionSubClass] variant back into its raw `u8` representation.
    pub fn from_u8(val: u8) -> Self {
        match val {
            0x00 => Self::Generic,
            other => Self::Unknown(other),
        }
    }

    /// Converts a [EncryptionSubClass] variant back into its raw `u8` representation.
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
    /// Generic controller.
    Generic,
    /// Unknown controller.
    Unknown(u8),
}

impl SignalProcessingSubClass {
    /// Converts a [SignalProcessingSubClass] variant back into its raw `u8` representation.
    pub fn from_u8(val: u8) -> Self {
        match val {
            0x00 => Self::Generic,
            other => Self::Unknown(other),
        }
    }

    /// Converts a [SignalProcessingSubClass] variant back into its raw `u8` representation.
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
    /// Generic accelerator.
    Generic,
    /// Unknown accelerator.
    Unknown(u8),
}

impl ProcessingAcceleratorSubClass {
    /// Converts a [ProcessingAcceleratorSubClass] variant back into its raw `u8` representation.
    pub fn from_u8(val: u8) -> Self {
        match val {
            0x00 => Self::Generic,
            other => Self::Unknown(other),
        }
    }

    /// Converts a [ProcessingAcceleratorSubClass] variant back into its raw `u8` representation.
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
    /// Generic instrumentation.
    Generic,
    /// Unknown instrumentation.
    Unknown(u8),
}

impl NonEssentialInstrumentationSubClass {
    /// Converts a [NonEssentialInstrumentationSubClass] variant back into its raw `u8` representation.
    pub fn from_u8(val: u8) -> Self {
        match val {
            0x00 => Self::Generic,
            other => Self::Unknown(other),
        }
    }

    /// Converts a [NonEssentialInstrumentationSubClass] variant back into its raw `u8` representation.
    pub fn to_u8(&self) -> u8 {
        match *self {
            Self::Generic => 0x00,
            Self::Unknown(val) => val,
        }
    }
}
