mod method;
mod parameters;
mod polar_cirlcle_resolution;

pub use method::*;
pub use parameters::*;
pub use polar_cirlcle_resolution::*;

/// Time adjustment for all prayer times.
/// The value is specified in *minutes* and
/// can be either positive or negative.
#[derive(Default, PartialEq, Eq, Debug, Copy, Clone)]
pub struct TimeAdjustment {
    /// Fajr
    pub fajr: i64,
    /// Sunrise
    pub sunrise: i64,
    /// Dhuhr
    pub dhuhr: i64,
    /// Asr
    pub asr: i64,
    /// Maghrib
    pub maghrib: i64,
    /// Isha
    pub isha: i64,
}

/// Rule for approximating Fajr and Isha at high latitudes
#[derive(PartialEq, Eq, Debug, Copy, Clone, Hash)]
pub enum HighLatitudeRule {
    /// Middle of the night
    MiddleOfTheNight,
    /// Seventh of the night
    SeventhOfTheNight,
    /// Twilight angle
    TwilightAngle,
}

/// Names of all obligatory prayers, sunrise, and Qiyam.
#[derive(PartialEq, Eq, Debug, Copy, Clone, Hash)]
pub enum Prayer {
    /// Anywhere yesterday
    Yesterday,
    /// Qiyam, yesterday
    QiyamYesterday,
    /// Fajr
    Fajr,
    /// Sunrise
    Sunrise,
    /// Dhuhr
    Dhuhr,
    /// Asr awwal
    AsrAwwal,
    /// Asr thaani
    AsrThaani,
    /// Maghrib
    Maghrib,
    /// Isha
    Isha,
    /// Qiyam
    Qiyam,
    /// Anywhere tomorrow
    Tomorrow,
}

// impl Prayer {
//     /// The next prayer within date
//     pub fn next(self) -> Option<Prayer> {
//         match self {
//             Prayer::Fajr => Some(Prayer::Sunrise),
//             Prayer::Sunrise => Some(Prayer::Dhuhr),
//             Prayer::Dhuhr => Some(Prayer::AsrAwwal),
//             Prayer::AsrAwwal | Prayer::AsrThaani => Some(Prayer::Maghrib),
//             Prayer::Maghrib => Some(Prayer::Isha),
//             Prayer::Isha => Some(Prayer::Qiyam),
//             _ => None,
//         }
//     }

//     /// The previous prayer within date
//     pub fn prev(self) -> Option<Prayer> {
//         match self {
//             Prayer::Sunrise => Some(Prayer::Fajr),
//             Prayer::Dhuhr => Some(Prayer::Sunrise),
//             Prayer::AsrAwwal | Prayer::AsrThaani => Some(Prayer::Dhuhr),
//             Prayer::Maghrib => Some(Prayer::AsrThaani),
//             Prayer::Isha => Some(Prayer::Maghrib),
//             Prayer::Qiyam => Some(Prayer::Isha),
//             _ => None,
//         }
//     }
// }
