use core::arch::asm;
use kernel_core::time::Time;

/// Read one CMOS register (port 0x70/0x71)
unsafe fn cmos_read(register: u8) -> u8 {
    let value: u8;

    unsafe {
        asm!(
        "out 0x70, al",   // select register
        "in  al, 0x71",   // read register
        in("al") register,
        lateout("al") value,
        options(nostack, nomem)
        );
    }

    value
}

fn bcd_to_bin(value: u8) -> u8 {
    ((value >> 4) * 10) + (value & 0x0F)
}

unsafe fn rtc_update_in_progress() -> bool {
    (unsafe { cmos_read(0x0A) } & 0x80) != 0
}

/// Reads the RTC *safely* using the double-read algorithm and returns a Time.
pub fn read() -> Time {
    unsafe {
        // Wait until RTC is not updating
        while rtc_update_in_progress() {}

        // first snapshot + read reg_b
        let sec1 = cmos_read(0x00);
        let min1 = cmos_read(0x02);
        let hour1 = cmos_read(0x04);
        let day1 = cmos_read(0x07);
        let mon1 = cmos_read(0x08);
        let year1 = cmos_read(0x09);
        let reg_b = cmos_read(0x0B);

        // read again until stable
        let (sec2, min2, hour2, day2, mon2, year2) = loop {
            while rtc_update_in_progress() {}

            let s = cmos_read(0x00);
            let m = cmos_read(0x02);
            let h = cmos_read(0x04);
            let d = cmos_read(0x07);
            let mo = cmos_read(0x08);
            let y = cmos_read(0x09);

            if s == sec1 && m == min1 && h == hour1 && d == day1 && mo == mon1 && y == year1 {
                break (s, m, h, d, mo, y);
            }
        };

        // raw bytes (contain PM flag in bit 7 when 12-hour mode)
        let mut sec = sec2;
        let mut min = min2;
        let mut hour = hour2; // keep raw copy until PM detection/conversion
        let mut day = day2;
        let mut month = mon2;
        let mut year = year2;

        // Determine if RTC uses binary mode or BCD
        let is_binary = (reg_b & 0x04) != 0;

        // If in BCD mode, convert all fields (but preserve PM/AM bit in raw_hour for now)
        if !is_binary {
            sec = bcd_to_bin(sec);
            min = bcd_to_bin(min);
            hour = bcd_to_bin(hour2 & 0x7F); // convert only the numeric bits
            day = bcd_to_bin(day);
            month = bcd_to_bin(month);
            year = bcd_to_bin(year);
        } else {
            // binary mode: numeric value is already fine, but drop PM flag bit if present
            hour = hour2 & 0x7F;
        }

        // Handle 12-hour mode -> convert to 24-hour using the PM bit present in raw_hour
        let is_24h = (reg_b & 0x02) != 0;
        if !is_24h {
            let pm = (hour2 & 0x80) != 0;
            // `hour` currently equals numeric hour in range 1..12
            if pm {
                if hour != 12 {
                    hour = hour.wrapping_add(12);
                }
            } else {
                // AM case: 12 AM should become 0
                if hour == 12 {
                    hour = 0;
                }
            }
        }

        // Compute full year using century register (0x32) if present.
        // Many firmwares supply register 0x32 as the century (e.g., 20 for 20xx).
        let full_year: u16 = {
            let mut full = year as u16;
            // Try to read century register. This is BIOS-specific and may be absent.
            let century_reg = 0x32u8;
            // Attempt to read it — if it returns 0 or 0xFF it might be unused; treat accordingly.
            let century_byte = cmos_read(century_reg);
            if century_byte != 0 && century_byte != 0xFF {
                let cent = if !is_binary {
                    bcd_to_bin(century_byte)
                } else {
                    century_byte
                } as u16;
                full = cent * 100 + full;
            } else {
                // fallback: expand two-digit year; assume 2000..2099 (you can change policy)
                full = 2000 + full;
            }
            full
        };

        Time {
            sec,
            min,
            hour,
            day,
            month,
            year: full_year,
        }
    }
}
