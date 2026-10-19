use crate::control::input;
use crate::device::usb::{UsbDevice, UsbDriver, UsbError};
use crate::sync::mutex::Mutex;
use pc_keyboard::{HandleControl, UsbBootKeyboardReport, UsbKeyboard, layouts};

const NAME: &str = "usb-keyboard-driver";

/// USB Keyboard Driver.
pub struct UsbKeyboardDriver {
    inner: Mutex<UsbKeyboard<layouts::Us104Key>>,
}

impl UsbKeyboardDriver {
    /// Creates a new [UsbKeyboardDriver] instance.
    pub fn new() -> Self {
        Self {
            inner: Mutex::new(UsbKeyboard::new(
                layouts::Us104Key,
                HandleControl::MapLettersToUnicode,
            )),
        }
    }
}

impl UsbDriver for UsbKeyboardDriver {
    fn name(&self) -> &'static str {
        NAME
    }

    fn should_bind(&self, device: &UsbDevice) -> bool {
        let desc = device.descriptor();

        desc.device_class == 0x03 && desc.device_subclass == 0x01 && desc.device_protocol == 0x01
    }

    fn init(&self, device: &UsbDevice) -> Result<(), UsbError> {
        let mut dummy = [];
        device
            .handle()
            .control_transfer(0x21, 0x0B, 0x00, 0, &mut dummy)?;

        log::info!("USB Keyboard protocol set successfully. Polling endpoint 0x81...");

        let handle = device.handle();
        let mut report_buf = [0u8; 8];

        match handle.submit_interrupt_transfer(0x81, &mut report_buf) {
            Ok(bytes) if bytes > 0 => {
                self.handle_report(device, &report_buf);
            }
            _ => {}
        }

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

        self.inner.run(|keyboard| {
            for key in keyboard.handle_report(&boot_report) {
                input::INPUT.get().push(key);
            }
        });
    }
}
