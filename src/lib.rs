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
//! let params        = Parameters::new(&prominent_methods::NorthAmerica);
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
    fn calculate_prayer_times_raleigh_usa() {
        let local_date = NaiveDate::from_ymd(2015, 7, 12);
        let params = Parameters::new(&prominent_methods::NorthAmerica);
        let coordinates = Coordinates::new(35.7750, -78.6336);
        let schedule = PrayerTimes::calculate(local_date, coordinates, params).unwrap();

        assert_eq!(
            schedule.time_of(Prayer::Fajr).unwrap(),
            Utc.ymd(2015, 7, 12).and_hms(8, 42, 0),
        );
        assert_eq!(
            schedule.time_of(Prayer::Sunrise).unwrap(),
            Utc.ymd(2015, 7, 12).and_hms(10, 7, 0),
        );
        assert_eq!(
            schedule.time_of(Prayer::Dhuhr).unwrap(),
            Utc.ymd(2015, 7, 12).and_hms(17, 21, 0),
        );
        assert_eq!(
            schedule.time_of(Prayer::AsrThaani).unwrap(),
            Utc.ymd(2015, 7, 12).and_hms(22, 22, 0),
        );
        assert_eq!(
            schedule.time_of(Prayer::Maghrib).unwrap(),
            Utc.ymd(2015, 7, 13).and_hms(0, 32, 0),
        );
        assert_eq!(
            schedule.time_of(Prayer::Isha).unwrap(),
            Utc.ymd(2015, 7, 13).and_hms(1, 57, 0),
        );
    }

    #[test]
    fn calculate_prayer_times_rajshahi_bd() {
        let local_date = NaiveDate::from_ymd(2021, 5, 24);
        let params = Parameters::new(&prominent_methods::Karachi);
        let coordinates = Coordinates::new(24.383144, 88.583183);
        let schedule = PrayerTimes::calculate(local_date, coordinates, params).unwrap();

        assert_eq!(
            schedule.time_of(Prayer::Fajr).unwrap(),
            Utc.ymd(2021, 5, 23).and_hms(21, 53, 0),
        );
        assert_eq!(
            schedule.time_of(Prayer::Sunrise).unwrap(),
            Utc.ymd(2021, 5, 23).and_hms(23, 18, 0),
        );
        assert_eq!(
            schedule.time_of(Prayer::Dhuhr).unwrap(),
            Utc.ymd(2021, 5, 24).and_hms(6, 4, 0),
        );
        assert_eq!(
            schedule.time_of(Prayer::AsrThaani).unwrap(),
            Utc.ymd(2021, 5, 24).and_hms(10, 43, 0),
        );
        assert_eq!(
            schedule.time_of(Prayer::Maghrib).unwrap(),
            Utc.ymd(2021, 5, 24).and_hms(12, 46, 0),
        );
        assert_eq!(
            schedule.time_of(Prayer::Isha).unwrap(),
            Utc.ymd(2021, 5, 24).and_hms(14, 12, 0),
        );
    }

    #[test]
    fn calculate_qiyam_times() {
        let date = NaiveDate::from_ymd(2015, 7, 12);
        // let qiyam_date  = NaiveDate::from_ymd(2015, 7, 13);
        let params = Parameters::new(&prominent_methods::NorthAmerica);
        let coordinates = Coordinates::new(35.7750, -78.6336);
        let schedule = PrayerTimes::calculate(date, coordinates, params).unwrap();

        // Today's Maghrib: 2015-07-13T00:32:00Z
        // Tomorrow's Fajr: 2015-07-13T08:43:00Z
        // Middle of Night: 2015-07-13T04:38:00Z
        // Last Third     : 2015-07-13T05:59:00Z
        assert_eq!(
            schedule.time_of(Prayer::Qiyam).unwrap(),
            Utc.ymd(2015, 7, 13).and_hms(5, 59, 0)
        );
    }
}
