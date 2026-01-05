use core::arch::asm;
use core::sync::atomic::{AtomicI8, Ordering};
use kernel_core::time::{Date, Month, OffsetDateTime, Time, UtcDateTime, UtcOffset};

static TIMEZONE_HOURS: AtomicI8 = AtomicI8::new(0);
static TIMEZONE_MINUTES: AtomicI8 = AtomicI8::new(0);
static TIMEZONE_SECONDS: AtomicI8 = AtomicI8::new(0);

pub fn set_offset(hours: i8, minutes: i8, seconds: i8) {
    TIMEZONE_HOURS.store(hours, Ordering::Relaxed);
    TIMEZONE_MINUTES.store(minutes, Ordering::Relaxed);
    TIMEZONE_SECONDS.store(seconds, Ordering::Relaxed);
}

pub fn read_local() -> OffsetDateTime {
    let utc = read_utc();

    let off_hours = TIMEZONE_HOURS.load(Ordering::Relaxed);
    let off_minutes = TIMEZONE_MINUTES.load(Ordering::Relaxed);
    let off_seconds = TIMEZONE_SECONDS.load(Ordering::Relaxed);

    utc.to_offset(
        UtcOffset::from_hms(off_hours, off_minutes, off_seconds).expect("Failed to create offset"),
    )
}

pub fn read_utc() -> UtcDateTime {
    unsafe {
        while rtc_update_in_progress() {}

        let mut sec = cmos_read(0x00);
        let mut min = cmos_read(0x02);
        let mut hour = cmos_read(0x04);
        let mut day = cmos_read(0x07);
        let mut month_raw = cmos_read(0x08);
        let mut year_raw = cmos_read(0x09);
        let reg_b = cmos_read(0x0B);

        let is_binary = (reg_b & 0x04) != 0;
        let is_24h = (reg_b & 0x02) != 0;

        // Decode BCD if necessary
        if !is_binary {
            sec = bcd_to_bin(sec);
            min = bcd_to_bin(min);
            day = bcd_to_bin(day);
            month_raw = bcd_to_bin(month_raw);
            year_raw = bcd_to_bin(year_raw);
            // Hour needs special handling for the PM bit if not in 24h mode
            if !is_24h {
                let pm = (hour & 0x80) != 0;
                hour = bcd_to_bin(hour & 0x7F);
                if pm && hour != 12 {
                    hour += 12;
                } else if !pm && hour == 12 {
                    hour = 0;
                }
            } else {
                hour = bcd_to_bin(hour);
            }
        }

        // Handle Century
        let century_byte = cmos_read(0x32);
        let full_year = if century_byte != 0 && century_byte != 0xFF {
            let cent = if is_binary {
                century_byte
            } else {
                bcd_to_bin(century_byte)
            };
            (cent as i32 * 100) + year_raw as i32
        } else {
            2000 + year_raw as i32
        };

        // Month is 1-indexed in RTC (1-12)
        let month = match month_raw {
            1 => Month::January,
            2 => Month::February,
            3 => Month::March,
            4 => Month::April,
            5 => Month::May,
            6 => Month::June,
            7 => Month::July,
            8 => Month::August,
            9 => Month::September,
            10 => Month::October,
            11 => Month::November,
            12 => Month::December,
            _ => Month::January, // Fallback for corrupt RTC
        };

        let date = Date::from_calendar_date(full_year, month, day)
            .unwrap_or(Date::from_calendar_date(2000, Month::January, 1).unwrap());

        let time = Time::from_hms(hour, min, sec).unwrap_or(Time::from_hms(0, 0, 0).unwrap());

        UtcDateTime::new(date, time)
    }
}

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
