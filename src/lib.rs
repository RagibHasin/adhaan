//! An Islamic prayer time implementation based on the [Adhan](https://github.com/batoulapps/Adhan) library by Batoul Apps.
//! While it has a lot of commnalities with the interface has
//! been changed slightly to make it more ergonomic and intuitive.
//!
//! ##### Example
//!
//! ```
//! use adhaan::*;
//!
//! let new_york_city = Coordinates::new(40.7128, -74.0059);
//! let date          = chrono::NaiveDate::from_ymd(2019, 1, 25);
//! let params        = Parameters::new(&prominent_methods::NorthAmerica, Madhhab::Hanafi);
//! let prayers       = PrayerTimes::calculate(date, new_york_city, params);
//! ```

#![forbid(unsafe_code, future_incompatible, deprecated_in_future)]
#![deny(unused, missing_docs)]

mod astrolabe;
mod models;
mod schedule;

pub use astrolabe::{qiblah::qiblah_direction, unit::Coordinates};
pub use models::*;
pub use schedule::PrayerTimes;

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::prelude::*;

    #[test]
    fn calculate_prayer_times() {
        let local_date = NaiveDate::from_ymd(2015, 7, 12);
        let params = Parameters::new(&prominent_methods::NorthAmerica, Madhhab::Hanafi);
        let coordinates = Coordinates::new(35.7750, -78.6336);
        let schedule = PrayerTimes::calculate(local_date, coordinates, params).unwrap();

        assert_eq!(
            schedule
                .time_of(Prayer::Fajr)
                .format("%-l:%M %p")
                .to_string(),
            "8:42 AM"
        );
        assert_eq!(
            schedule
                .time_of(Prayer::Sunrise)
                .format("%-l:%M %p")
                .to_string(),
            "10:08 AM"
        );
        assert_eq!(
            schedule
                .time_of(Prayer::Dhuhr)
                .format("%-l:%M %p")
                .to_string(),
            "5:21 PM"
        );
        assert_eq!(
            schedule
                .time_of(Prayer::Asr)
                .format("%-l:%M %p")
                .to_string(),
            "10:22 PM"
        );
        assert_eq!(
            schedule
                .time_of(Prayer::Maghrib)
                .format("%-l:%M %p")
                .to_string(),
            "12:32 AM"
        );
        assert_eq!(
            schedule
                .time_of(Prayer::Isha)
                .format("%-l:%M %p")
                .to_string(),
            "1:57 AM"
        );
    }

    #[test]
    fn calculate_qiyam_times() {
        let date = NaiveDate::from_ymd(2015, 7, 12);
        // let qiyam_date  = NaiveDate::from_ymd(2015, 7, 13);
        let params = Parameters::new(&prominent_methods::NorthAmerica, Madhhab::Hanafi);
        let coordinates = Coordinates::new(35.7750, -78.6336);
        let schedule = PrayerTimes::calculate(date, coordinates, params).unwrap();

        // Today's Maghrib: 2015-07-13T00:32:00Z
        // Tomorrow's Fajr: 2015-07-13T08:43:00Z
        // Middle of Night: 2015-07-13T04:38:00Z
        // Last Third     : 2015-07-13T05:59:00Z
        assert_eq!(
            schedule
                .time_of(Prayer::Maghrib)
                .format("%-l:%M %p")
                .to_string(),
            "12:32 AM"
        );
        assert_eq!(
            schedule
                .time_of(Prayer::Qiyam)
                .format("%-l:%M %p")
                .to_string(),
            "5:59 AM"
        );
    }
}
