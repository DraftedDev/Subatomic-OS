use alloc::vec::Vec;
pub use time::*;

/// Specifies a named time zone or a custom offset from UTC.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum TimeZone {
    /// Anywhere on Earth (Baker Island, Howland Island).
    AoE,
    /// Samoa Standard Time (American Samoa).
    SST,
    /// Hawaii Standard Time.
    HawaiiStandardTime,
    /// Marquesas Time.
    MarquesasTime,
    /// Alaska Standard Time.
    AlaskaStandardTime,
    /// Pacific Standard Time (US West Coast).
    PacificStandardTime,
    /// Mountain Standard Time (US).
    MountainStandardTime,
    /// Central Standard Time (US).
    CentralStandardTime,
    /// Eastern Standard Time (US).
    EasternStandardTime,
    /// Atlantic Standard Time.
    AtlanticStandardTime,
    /// Newfoundland Standard Time.
    NewfoundlandStandardTime,
    /// Brasília Time (Brazil).
    BrasiliaTime,
    /// UTC−2 (South Georgia & South Sandwich Islands).
    UTCMinus2,
    /// UTC−1 (Azores, Cape Verde).
    UTCMinus1,
    /// Coordinated Universal Time / Greenwich Mean Time.
    UTC,
    /// Central European Time.
    CentralEuropeanTime,
    /// Eastern European Time.
    EasternEuropeanTime,
    /// South Africa Standard Time.
    SouthAfricaStandardTime,
    /// Moscow Time.
    MoscowTime,
    /// Iran Standard Time.
    IranStandardTime,
    /// Gulf Standard Time (UAE, Oman).
    GulfStandardTime,
    /// Afghanistan Time.
    AfghanistanTime,
    /// Pakistan Standard Time.
    PakistanStandardTime,
    /// India Standard Time.
    IndiaStandardTime,
    /// Nepal Time.
    NepalTime,
    /// Bangladesh Standard Time.
    BangladeshStandardTime,
    /// Myanmar Time.
    MyanmarTime,
    /// Indochina Time (Thailand, Vietnam).
    IndochinaTime,
    /// China Standard Time.
    ChinaStandardTime,
    /// Australian Central Standard Time.
    AustralianCentralStandardTime,
    /// Australian Eastern Standard Time.
    AustralianEasternStandardTime,
    /// Lord Howe Standard Time.
    LordHoweStandardTime,
    /// Solomon Islands Time.
    SolomonIslandsTime,
    /// New Zealand Standard Time.
    NewZealandStandardTime,
    /// Chatham Islands Time.
    ChathamIslandsTime,
    /// Tonga Time / New Zealand Daylight Time.
    TongaTime,
    /// Line Islands Time.
    LineIslandsTime,
    /// A custom offset in hours, minutes, and seconds from UTC.
    Custom {
        /// Offset in hours.
        hours: i8,
        /// Offset in minutes.
        minutes: i8,
        /// Offset in seconds.
        seconds: i8,
    },
}

impl TimeZone {
    /// A list of all the named time zones.
    ///
    /// Includes all fields, but [TimeZone::Custom].
    pub const NAMED_ZONES: &'static [TimeZone] = &[
        TimeZone::AoE,
        TimeZone::SST,
        TimeZone::HawaiiStandardTime,
        TimeZone::MarquesasTime,
        TimeZone::AlaskaStandardTime,
        TimeZone::PacificStandardTime,
        TimeZone::MountainStandardTime,
        TimeZone::CentralStandardTime,
        TimeZone::EasternStandardTime,
        TimeZone::AtlanticStandardTime,
        TimeZone::NewfoundlandStandardTime,
        TimeZone::BrasiliaTime,
        TimeZone::UTCMinus2,
        TimeZone::UTCMinus1,
        TimeZone::UTC,
        TimeZone::CentralEuropeanTime,
        TimeZone::EasternEuropeanTime,
        TimeZone::SouthAfricaStandardTime,
        TimeZone::MoscowTime,
        TimeZone::IranStandardTime,
        TimeZone::GulfStandardTime,
        TimeZone::AfghanistanTime,
        TimeZone::PakistanStandardTime,
        TimeZone::IndiaStandardTime,
        TimeZone::NepalTime,
        TimeZone::BangladeshStandardTime,
        TimeZone::MyanmarTime,
        TimeZone::IndochinaTime,
        TimeZone::ChinaStandardTime,
        TimeZone::AustralianCentralStandardTime,
        TimeZone::AustralianEasternStandardTime,
        TimeZone::LordHoweStandardTime,
        TimeZone::SolomonIslandsTime,
        TimeZone::NewZealandStandardTime,
        TimeZone::ChathamIslandsTime,
        TimeZone::TongaTime,
        TimeZone::LineIslandsTime,
    ];

    /// Returns the symbol of the time zone.
    ///
    /// Returns [None] if the time zone is [TimeZone::Custom].
    pub fn as_symbol(&self) -> Option<&'static str> {
        match self {
            Self::AoE => Some("AoE"),
            Self::SST => Some("SST"),
            Self::HawaiiStandardTime => Some("HST"),
            Self::MarquesasTime => Some("MART"),
            Self::AlaskaStandardTime => Some("AKST"),
            Self::PacificStandardTime => Some("PST"),
            Self::MountainStandardTime => Some("MST"),
            Self::CentralStandardTime => Some("CST"),
            Self::EasternStandardTime => Some("EST"),
            Self::AtlanticStandardTime => Some("AST"),
            Self::NewfoundlandStandardTime => Some("NST"),
            Self::BrasiliaTime => Some("BRT"),
            Self::UTCMinus2 => Some("UTC-2"),
            Self::UTCMinus1 => Some("UTC-1"),
            Self::UTC => Some("UTC"),
            Self::CentralEuropeanTime => Some("CET"),
            Self::EasternEuropeanTime => Some("EET"),
            Self::SouthAfricaStandardTime => Some("SAST"),
            Self::MoscowTime => Some("MSK"),
            Self::IranStandardTime => Some("IRST"),
            Self::GulfStandardTime => Some("GST"),
            Self::AfghanistanTime => Some("AFT"),
            Self::PakistanStandardTime => Some("PKT"),
            Self::IndiaStandardTime => Some("IST"),
            Self::NepalTime => Some("NPT"),
            Self::BangladeshStandardTime => Some("BST"),
            Self::MyanmarTime => Some("MMT"),
            Self::IndochinaTime => Some("ICT"),
            Self::ChinaStandardTime => Some("CST_CN"),
            Self::AustralianCentralStandardTime => Some("ACST"),
            Self::AustralianEasternStandardTime => Some("AEST"),
            Self::LordHoweStandardTime => Some("LHST"),
            Self::SolomonIslandsTime => Some("SBT"),
            Self::NewZealandStandardTime => Some("NZST"),
            Self::ChathamIslandsTime => Some("CHAST"),
            Self::TongaTime => Some("TOT"),
            Self::LineIslandsTime => Some("LINT"),
            Self::Custom { .. } => None,
        }
    }

    /// Returns the offset from UTC in hours, minutes, and seconds.
    pub fn to_offset(&self) -> (i8, i8, i8) {
        match self {
            Self::AoE => (-12, 0, 0),
            Self::SST => (-11, 0, 0),
            Self::HawaiiStandardTime => (-10, 0, 0),
            Self::MarquesasTime => (-9, -30, 0),
            Self::AlaskaStandardTime => (-9, 0, 0),
            Self::PacificStandardTime => (-8, 0, 0),
            Self::MountainStandardTime => (-7, 0, 0),
            Self::CentralStandardTime => (-6, 0, 0),
            Self::EasternStandardTime => (-5, 0, 0),
            Self::AtlanticStandardTime => (-4, 0, 0),
            Self::NewfoundlandStandardTime => (-3, -30, 0),
            Self::BrasiliaTime => (-3, 0, 0),
            Self::UTCMinus2 => (-2, 0, 0),
            Self::UTCMinus1 => (-1, 0, 0),
            Self::UTC => (0, 0, 0),
            Self::CentralEuropeanTime => (1, 0, 0),
            Self::EasternEuropeanTime => (2, 0, 0),
            Self::SouthAfricaStandardTime => (2, 0, 0),
            Self::MoscowTime => (3, 0, 0),
            Self::IranStandardTime => (3, 30, 0),
            Self::GulfStandardTime => (4, 0, 0),
            Self::AfghanistanTime => (4, 30, 0),
            Self::PakistanStandardTime => (5, 0, 0),
            Self::IndiaStandardTime => (5, 30, 0),
            Self::NepalTime => (5, 45, 0),
            Self::BangladeshStandardTime => (6, 0, 0),
            Self::MyanmarTime => (6, 30, 0),
            Self::IndochinaTime => (7, 0, 0),
            Self::ChinaStandardTime => (8, 0, 0),
            Self::AustralianCentralStandardTime => (9, 30, 0),
            Self::AustralianEasternStandardTime => (10, 0, 0),
            Self::LordHoweStandardTime => (10, 30, 0),
            Self::SolomonIslandsTime => (11, 0, 0),
            Self::NewZealandStandardTime => (12, 0, 0),
            Self::ChathamIslandsTime => (12, 45, 0),
            Self::TongaTime => (13, 0, 0),
            Self::LineIslandsTime => (14, 0, 0),
            Self::Custom {
                hours,
                minutes,
                seconds,
            } => (*hours, *minutes, *seconds),
        }
    }

    /// Returns the `TimeZone` from its common acronym or symbol.
    ///
    /// Examples:
    /// - "AoE" => `TimeZone::AoE`
    /// - "EST" => `TimeZone::EasternStandardTime`
    /// - "LINT" => `TimeZone::LineIslandsTime`
    ///
    /// Returns `None` if the symbol is not recognized.
    pub fn from_symbol(symbol: &str) -> Option<Self> {
        match symbol {
            "AoE" => Some(Self::AoE),
            "SST" => Some(Self::SST),
            "HST" => Some(Self::HawaiiStandardTime),
            "MART" => Some(Self::MarquesasTime),
            "AKST" => Some(Self::AlaskaStandardTime),
            "PST" => Some(Self::PacificStandardTime),
            "MST" => Some(Self::MountainStandardTime),
            "CST" => Some(Self::CentralStandardTime),
            "EST" => Some(Self::EasternStandardTime),
            "AST" => Some(Self::AtlanticStandardTime),
            "NST" => Some(Self::NewfoundlandStandardTime),
            "BRT" => Some(Self::BrasiliaTime),
            "UTC-2" => Some(Self::UTCMinus2),
            "UTC-1" => Some(Self::UTCMinus1),
            "UTC" | "GMT" => Some(Self::UTC),
            "CET" => Some(Self::CentralEuropeanTime),
            "EET" => Some(Self::EasternEuropeanTime),
            "SAST" => Some(Self::SouthAfricaStandardTime),
            "MSK" => Some(Self::MoscowTime),
            "IRST" => Some(Self::IranStandardTime),
            "GST" => Some(Self::GulfStandardTime),
            "AFT" => Some(Self::AfghanistanTime),
            "PKT" => Some(Self::PakistanStandardTime),
            "IST" => Some(Self::IndiaStandardTime),
            "NPT" => Some(Self::NepalTime),
            "BST" => Some(Self::BangladeshStandardTime),
            "MMT" => Some(Self::MyanmarTime),
            "ICT" => Some(Self::IndochinaTime),
            "CST_CN" => Some(Self::ChinaStandardTime),
            "ACST" => Some(Self::AustralianCentralStandardTime),
            "AEST" => Some(Self::AustralianEasternStandardTime),
            "LHST" => Some(Self::LordHoweStandardTime),
            "SBT" => Some(Self::SolomonIslandsTime),
            "NZST" => Some(Self::NewZealandStandardTime),
            "CHAST" => Some(Self::ChathamIslandsTime),
            "TOT" => Some(Self::TongaTime),
            "LINT" => Some(Self::LineIslandsTime),
            _ => None,
        }
    }

    /// Parses a symbol using [TimeZone::from_symbol] or by parsing a custom offset
    /// with the format `+hh:+mm:+ss` or `-hh:-mm:-ss` respectively.
    ///
    /// Returns [None] if the symbol is not recognized or the offset is invalid.
    ///
    /// Example:
    ///
    /// `TimeZone::parse("UTC-2")` => `TimeZone::UTCMinus2`
    ///
    /// `TimeZone::parse("-02:+00:+00")` => `TimeZone::UTCMinus2`
    pub fn parse(s: &str) -> Option<Self> {
        if let Some(tz) = Self::from_symbol(s) {
            return Some(tz);
        }

        let parts: Vec<&str> = s.split(':').collect();
        if parts.len() != 3 {
            return None;
        }

        let parse_part = |p: &str| -> Option<i8> {
            if p.len() < 2 {
                return None;
            }
            let (sign, num_str) = p.split_at(1);
            let num: i8 = num_str.parse().ok()?;
            match sign {
                "+" => Some(num),
                "-" => Some(-num),
                _ => None,
            }
        };

        let hours = parse_part(parts[0])?;
        let minutes = parse_part(parts[1])?;
        let seconds = parse_part(parts[2])?;

        Some(Self::Custom {
            hours,
            minutes,
            seconds,
        })
    }
}
