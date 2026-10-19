use crate::control::input::{self, KeyLayout};
use crate::device::usb::{USB_HUB, UsbDevice, UsbDriver, UsbError};
use crate::sync::mutex::Mutex;
use pc_keyboard::{HandleControl, Modifiers, UsbBootKeyboardReport, UsbKeyboard};

const NAME: &str = "usb-keyboard-driver";

/// USB Keyboard Driver.
pub struct UsbKeyboardDriver {
    inner: Mutex<UsbKeyboard<KeyLayout>>,
}

impl UsbKeyboardDriver {
    /// Fetch the driver from the global USB device hub and run the closure.
    ///
    /// Will panic if the driver is not registered and initialized yet.
    pub fn fetch<R>(f: impl FnOnce(&mut UsbKeyboard<KeyLayout>) -> R) -> R {
        unsafe { USB_HUB.get_unsafe() }.run(|hub| {
            let driver = unsafe { hub.get_driver_cast::<UsbKeyboardDriver>(NAME) }
                .expect("Driver not initialized");

            driver.run(f)
        })
    }

    /// Creates a new [UsbKeyboardDriver] instance.
    pub fn new() -> Self {
        Self {
            inner: Mutex::new(UsbKeyboard::new(
                // TODO: Make this configurable.
                KeyLayout::De105Key,
                HandleControl::MapLettersToUnicode,
            )),
        }
    }

    /// Locks the inner state and runs the given closure.
    pub fn run<R>(&self, f: impl FnOnce(&mut UsbKeyboard<KeyLayout>) -> R) -> R {
        self.inner.run(f)
    }
}

impl UsbDriver for UsbKeyboardDriver {
    fn name(&self) -> &'static str {
        NAME
    }

    fn should_bind(&self, device: &UsbDevice) -> bool {
        let desc = device.descriptor();

        (desc.device_class == 0x03 || desc.device_class == 0x00)
            && (desc.device_subclass == 0x01 || desc.device_subclass == 0x00)
            && (desc.device_protocol == 0x01 || desc.device_protocol == 0x00)
    }

    fn init(&self, device: &UsbDevice) -> Result<(), UsbError> {
        let mut dummy = [];
        let _ = device
            .handle()
            .control_transfer(0x21, 0x0B, 0x00, 0, &mut dummy);

        log::info!("USB Keyboard driver initialized.");

        Ok(())
    }

    fn handle_report(&self, _device: &UsbDevice, report: &[u8]) {
        if report.len() < 8 {
            return;
        }

        let mut keys_array = [0u8; 6];
        keys_array.copy_from_slice(&report[2..8]);

        let boot_report = UsbBootKeyboardReport {
            modifiers: report[0],
            keys: keys_array,
        };

        let mods = Modifiers {
            lshift: (report[0] & (1 << 1)) != 0,
            rshift: (report[0] & (1 << 5)) != 0,
            lctrl: (report[0] & (1 << 0)) != 0,
            rctrl: (report[0] & (1 << 4)) != 0,
            rctrl2: false,
            numlock: false,
            capslock: false,
            lalt: (report[0] & (1 << 2)) != 0,
            ralt: (report[0] & (1 << 6)) != 0,
        };

        self.inner.run(|keyboard| {
            for key in keyboard.handle_report(&boot_report) {
                input::INPUT.get().push(key);
            }
        });

        input::INPUT.get().set_modifiers(&mods);
    }
}
