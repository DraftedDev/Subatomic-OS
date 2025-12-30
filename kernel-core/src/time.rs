use crate::api;
use core::time::Duration;

/// Represents the time of the system clock.
#[derive(Debug, Copy, Clone)]
pub struct Time {
    /// Seconds. Maximum 59.
    pub sec: u8,
    /// Minutes. Maximum 59.
    pub min: u8,
    /// Hours. Maximum 23.
    pub hour: u8,
    /// Days. Maximum 31.
    pub day: u8,
    /// Months. Maximum 12.
    pub month: u8,
    /// Years. Maximum 99. Only represents the last two digits. So `50` may mean `1950` or `2050`.
    pub year: u16,
}

impl Time {
    /// Read the current time from the [api::TimeApi].
    pub fn read() -> Self {
        api::time().read()
    }

    /// Check if this is a leap year.
    pub fn is_leap_year(year: u16) -> bool {
        year % 4 == 0
    }

    /// Number of days in the month (1–12).
    pub fn days_in_month(month: u8, year: u16) -> u32 {
        match month {
            1 => 31,
            2 => {
                if Self::is_leap_year(year) {
                    29
                } else {
                    28
                }
            }
            3 => 31,
            4 => 30,
            5 => 31,
            6 => 30,
            7 => 31,
            8 => 31,
            9 => 30,
            10 => 31,
            11 => 30,
            12 => 31,
            _ => 0,
        }
    }

    /// Convert a calendar date/time into a Duration since the epoch:
    /// 00:00:00 on 01-01-00.
    pub fn to_duration(&self) -> Duration {
        // --- Accumulate days from full years ---
        let mut days: u32 = 0;
        for y in 0..self.year {
            days += if Self::is_leap_year(y) { 366 } else { 365 };
        }

        // --- Accumulate days from full months ---
        for m in 1..self.month {
            days += Self::days_in_month(m, self.year);
        }

        // --- Add days within the month (day is 1-based) ---
        days += (self.day - 1) as u32;

        // --- Convert everything to seconds ---
        let secs =
            days as u64 * 86400 + self.hour as u64 * 3600 + self.min as u64 * 60 + self.sec as u64;

        Duration::from_secs(secs)
    }

    /// Create a Time from a Duration relative to the epoch
    /// 00:00:00 on 01-01-00.
    pub fn from_duration(dur: Duration) -> Self {
        let mut secs = dur.as_secs();

        // --- Extract H/M/S ---
        let sec = (secs % 60) as u8;
        secs /= 60;

        let min = (secs % 60) as u8;
        secs /= 60;

        let hour = (secs % 24) as u8;
        secs /= 24;

        // Remaining seconds represent total days
        let mut days = secs as u32;

        // --- Compute year ---
        let mut year = 0u16;
        loop {
            let year_days = if Self::is_leap_year(year) { 366 } else { 365 };
            if days < year_days {
                break;
            }
            days -= year_days;
            year += 1;
            year %= 100; // wrap at 100 since your year is 0–99
        }

        // --- Compute month ---
        let mut month = 1u8;
        loop {
            let dm = Self::days_in_month(month, year);
            if days < dm {
                break;
            }
            days -= dm;
            month += 1;
        }

        // Remaining days → 1-based day of month
        let day = (days + 1) as u8;

        Self {
            sec,
            min,
            hour,
            day,
            month,
            year,
        }
    }
}
