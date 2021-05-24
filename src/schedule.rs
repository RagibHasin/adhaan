//! # Prayer Schedule
//!
//! This module provide the main objects that are used for calculating
//! the prayer times.

use chrono::{prelude::*, Duration, DurationRound};

use crate::{
    astrolabe::{solar::SolarTime, unit::Coordinates},
    models::{Parameters, Prayer},
    PolarCircleResolution,
};

/// A data struct to hold the timing for all prayers.
#[derive(PartialEq, Debug, Clone)]
pub struct PrayerTimes {
    fajr: DateTime<Utc>,
    sunrise: DateTime<Utc>,
    dhuhr: DateTime<Utc>,
    asr: DateTime<Utc>,
    maghrib: DateTime<Utc>,
    isha: DateTime<Utc>,
    middle_of_night: DateTime<Utc>,
    qiyam: DateTime<Utc>,
    fajr_tomorrow: DateTime<Utc>,
    coordinates: Coordinates,
    date: NaiveDate,
    parameters: Parameters,
}

impl PrayerTimes {
    /// Calculate prayer times on a date, at a specific location with given parameters
    pub fn calculate(
        date: NaiveDate,
        coordinates: Coordinates,
        parameters: Parameters,
    ) -> Option<PrayerTimes> {
        let tomorrow = date.succ();
        let solar_time = SolarTime::new(date, coordinates);
        let solar_time_tomorrow = SolarTime::new(tomorrow, coordinates);
        let solar_time_yesterday = SolarTime::new(date.pred(), coordinates);

        let PolarCircleResolution {
            date: _resolved_date,
            coordinates,
            solar_time,
            solar_time_tomorrow,
            solar_time_yesterday,
        } = if solar_time.is_none()
            || solar_time_tomorrow.is_none()
            || solar_time_yesterday.is_none()
        {
            parameters.polar_circle_resolver.resolve(date, coordinates)
        } else {
            PolarCircleResolution {
                date,
                coordinates,
                solar_time,
                solar_time_tomorrow,
                solar_time_yesterday,
            }
        };

        let solar_time = solar_time?;
        let solar_time_tomorrow = solar_time_tomorrow?;
        let solar_time_yesterday = solar_time_yesterday?;

        let tonight = solar_time.sunrise - solar_time_yesterday.sunset;
        let next_night = solar_time_tomorrow.sunrise - solar_time.sunset;

        let fajr =
            parameters
                .method
                .calculate_fajr(&parameters, &solar_time, tonight, coordinates, date)
                + Duration::minutes(parameters.time_adjustments(Prayer::Fajr));

        let sunrise =
            solar_time.sunrise + Duration::minutes(parameters.time_adjustments(Prayer::Sunrise));

        let dhuhr =
            solar_time.transit + Duration::minutes(parameters.time_adjustments(Prayer::Dhuhr));

        let asr = solar_time.afternoon(parameters.madhhab.shadow().into())?
            + Duration::minutes(parameters.time_adjustments(Prayer::Asr));

        let maghrib =
            solar_time.sunset + Duration::minutes(parameters.time_adjustments(Prayer::Maghrib));

        let isha =
            parameters
                .method
                .calculate_isha(&parameters, &solar_time, tonight, coordinates, date)
                + Duration::minutes(parameters.time_adjustments(Prayer::Isha));

        let fajr_tomorrow = parameters.method.calculate_fajr(
            &parameters,
            &solar_time_tomorrow,
            next_night,
            coordinates,
            tomorrow,
        ) + Duration::minutes(parameters.time_adjustments(Prayer::Fajr));

        let next_night_short = fajr_tomorrow - maghrib;
        let middle_of_night = (maghrib + next_night_short / 2)
            .duration_round(Duration::minutes(1))
            .unwrap();
        let last_third_of_night = (maghrib + next_night_short * 2 / 3)
            .duration_round(Duration::minutes(1))
            .unwrap();

        Some(PrayerTimes {
            fajr,
            sunrise,
            dhuhr,
            asr,
            maghrib,
            isha,
            middle_of_night,
            qiyam: last_third_of_night,
            fajr_tomorrow,
            coordinates,
            date,
            parameters,
        })
    }

    /// Get time of tha prayer
    pub fn time_of(&self, prayer: Prayer) -> DateTime<Utc> {
        match prayer {
            Prayer::Fajr => self.fajr,
            Prayer::Sunrise => self.sunrise,
            Prayer::Dhuhr => self.dhuhr,
            Prayer::Asr => self.asr,
            Prayer::Maghrib => self.maghrib,
            Prayer::Isha => self.isha,
            Prayer::Qiyam => self.qiyam,
            Prayer::FajrTomorrow => self.fajr_tomorrow,
        }
    }

    /// Time remaining for the current prayer at given time
    pub fn time_remaining(&self, now: DateTime<Utc>) -> Option<Duration> {
        self.prayer_at(now)
            .map(|prayer_now| {
                prayer_now
                    .next()
                    .map(|prayer_next| self.time_of(prayer_next) - self.time_of(prayer_now))
            })
            .flatten()
    }

    /// Current prayer at given time in the date
    pub fn prayer_at(&self, time: DateTime<Utc>) -> Option<Prayer> {
        if self.fajr_tomorrow <= time {
            None
        } else if self.qiyam <= time {
            Some(Prayer::Qiyam)
        } else if self.isha <= time {
            Some(Prayer::Isha)
        } else if self.maghrib <= time {
            Some(Prayer::Maghrib)
        } else if self.asr <= time {
            Some(Prayer::Asr)
        } else if self.dhuhr <= time {
            Some(Prayer::Dhuhr)
        } else if self.sunrise <= time {
            Some(Prayer::Sunrise)
        } else if self.fajr <= time {
            Some(Prayer::Fajr)
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{prominent_methods, Madhhab};

    #[test]
    fn current_prayer_should_be_fajr() {
        // Given the above DateTime, the Fajr prayer is at 2015-07-12T08:42:00Z
        let local_date = NaiveDate::from_ymd(2015, 7, 12);
        let params = Parameters::new(&prominent_methods::NorthAmerica, Madhhab::Hanafi);
        let coordinates = Coordinates::new(35.7750, -78.6336);
        let times = PrayerTimes::calculate(local_date, coordinates, params).unwrap();
        let current_prayer_time = Utc.from_utc_date(&local_date).and_hms(9, 0, 0);

        assert_eq!(times.prayer_at(current_prayer_time), Some(Prayer::Fajr));
    }

    #[test]
    fn current_prayer_should_be_sunrise() {
        // Given the below DateTime, sunrise is at 2015-07-12T10:08:00Z
        let local_date = NaiveDate::from_ymd(2015, 7, 12);
        let params = Parameters::new(&prominent_methods::NorthAmerica, Madhhab::Hanafi);
        let coordinates = Coordinates::new(35.7750, -78.6336);
        let times = PrayerTimes::calculate(local_date, coordinates, params).unwrap();
        let current_prayer_time = Utc.from_utc_date(&local_date).and_hms(11, 0, 0);

        assert_eq!(times.prayer_at(current_prayer_time), Some(Prayer::Sunrise));
    }

    #[test]
    fn current_prayer_should_be_dhuhr() {
        // Given the above DateTime, dhuhr prayer is at 2015-07-12T17:21:00Z
        let local_date = NaiveDate::from_ymd(2015, 7, 12);
        let params = Parameters::new(&prominent_methods::NorthAmerica, Madhhab::Hanafi);
        let coordinates = Coordinates::new(35.7750, -78.6336);
        let times = PrayerTimes::calculate(local_date, coordinates, params).unwrap();
        let current_prayer_time = Utc.from_utc_date(&local_date).and_hms(19, 0, 0);

        assert_eq!(times.prayer_at(current_prayer_time), Some(Prayer::Dhuhr));
    }

    #[test]
    fn current_prayer_should_be_asr() {
        // Given the below DateTime, asr is at 2015-07-12T22:22:00Z
        let local_date = NaiveDate::from_ymd(2015, 7, 12);
        let params = Parameters::new(&prominent_methods::NorthAmerica, Madhhab::Hanafi);
        let coordinates = Coordinates::new(35.7750, -78.6336);
        let times = PrayerTimes::calculate(local_date, coordinates, params).unwrap();
        let current_prayer_time = Utc.from_utc_date(&local_date).and_hms(22, 26, 0);

        assert_eq!(times.prayer_at(current_prayer_time), Some(Prayer::Asr));
    }

    #[test]
    fn current_prayer_should_be_maghrib() {
        // Given the below DateTime, maghrib is at 2015-07-13T00:32:00Z
        let local_date = NaiveDate::from_ymd(2015, 7, 12);
        let params = Parameters::new(&prominent_methods::NorthAmerica, Madhhab::Hanafi);
        let coordinates = Coordinates::new(35.7750, -78.6336);
        let times = PrayerTimes::calculate(local_date, coordinates, params).unwrap();
        let current_prayer_time = Utc.ymd(2015, 7, 13).and_hms(1, 0, 0);

        assert_eq!(times.prayer_at(current_prayer_time), Some(Prayer::Maghrib));
    }

    #[test]
    fn current_prayer_should_be_isha() {
        // Given the below DateTime, isha is at 2015-07-13T01:57:00Z
        let local_date = NaiveDate::from_ymd(2015, 7, 12);
        let params = Parameters::new(&prominent_methods::NorthAmerica, Madhhab::Hanafi);
        let coordinates = Coordinates::new(35.7750, -78.6336);
        let times = PrayerTimes::calculate(local_date, coordinates, params).unwrap();
        let current_prayer_time = Utc.ymd(2015, 7, 13).and_hms(2, 0, 0);

        assert_eq!(times.prayer_at(current_prayer_time), Some(Prayer::Isha));
    }

    #[test]
    fn current_prayer_should_be_none() {
        let local_date = NaiveDate::from_ymd(2015, 7, 12);
        let params = Parameters::new(&prominent_methods::NorthAmerica, Madhhab::Hanafi);
        let coordinates = Coordinates::new(35.7750, -78.6336);
        let times = PrayerTimes::calculate(local_date, coordinates, params).unwrap();
        let current_prayer_time = Utc.from_utc_date(&local_date).and_hms(8, 0, 0);

        assert_eq!(times.prayer_at(current_prayer_time), None);
    }

    #[test]
    fn calculate_times_for_moonsighting_method() {
        let date = NaiveDate::from_ymd(2016, 1, 31);
        let params = Parameters::new(&prominent_methods::MoonsightingCommittee, Madhhab::Shafii);
        let coordinates = Coordinates::new(35.7750, -78.6336);
        let schedule = PrayerTimes::calculate(date, coordinates, params).unwrap();

        // fajr    = 2016-01-31 10:48:00 UTC
        // sunrise = 2016-01-31 12:16:00 UTC
        // dhuhr   = 2016-01-31 17:33:00 UTC
        // asr     = 2016-01-31 20:20:00 UTC
        // maghrib = 2016-01-31 22:43:00 UTC
        // isha    = 2016-02-01 00:05:00 UTC
        assert_eq!(
            schedule
                .time_of(Prayer::Fajr)
                .format("%-l:%M %p")
                .to_string(),
            "10:48 AM"
        );
        assert_eq!(
            schedule
                .time_of(Prayer::Sunrise)
                .format("%-l:%M %p")
                .to_string(),
            "12:16 PM"
        );
        assert_eq!(
            schedule
                .time_of(Prayer::Dhuhr)
                .format("%-l:%M %p")
                .to_string(),
            "5:33 PM"
        );
        assert_eq!(
            schedule
                .time_of(Prayer::Asr)
                .format("%-l:%M %p")
                .to_string(),
            "8:20 PM"
        );
        assert_eq!(
            schedule
                .time_of(Prayer::Maghrib)
                .format("%-l:%M %p")
                .to_string(),
            "10:43 PM"
        );
        assert_eq!(
            schedule
                .time_of(Prayer::Isha)
                .format("%-l:%M %p")
                .to_string(),
            "12:05 AM"
        );
    }

    #[test]
    fn calculate_times_for_moonsighting_method_with_high_latitude() {
        let date = NaiveDate::from_ymd(2016, 1, 1);
        let params = Parameters::new(&prominent_methods::MoonsightingCommittee, Madhhab::Hanafi);
        let coordinates = Coordinates::new(59.9094, 10.7349);
        let schedule = PrayerTimes::calculate(date, coordinates, params).unwrap();

        // fajr    = 2016-01-01 06:34:00 UTC
        // sunrise = 2016-01-01 08:19:00 UTC
        // dhuhr   = 2016-01-01 11:25:00 UTC
        // asr     = 2016-01-01 12:36:00 UTC
        // maghrib = 2016-01-01 14:25:00 UTC
        // isha    = 2016-01-01 16:02:00 UTC
        assert_eq!(
            schedule
                .time_of(Prayer::Fajr)
                .format("%-l:%M %p")
                .to_string(),
            "6:34 AM"
        );
        assert_eq!(
            schedule
                .time_of(Prayer::Sunrise)
                .format("%-l:%M %p")
                .to_string(),
            "8:19 AM"
        );
        assert_eq!(
            schedule
                .time_of(Prayer::Dhuhr)
                .format("%-l:%M %p")
                .to_string(),
            "11:25 AM"
        );
        assert_eq!(
            schedule
                .time_of(Prayer::Asr)
                .format("%-l:%M %p")
                .to_string(),
            "12:36 PM"
        );
        assert_eq!(
            schedule
                .time_of(Prayer::Maghrib)
                .format("%-l:%M %p")
                .to_string(),
            "2:25 PM"
        );
        assert_eq!(
            schedule
                .time_of(Prayer::Isha)
                .format("%-l:%M %p")
                .to_string(),
            "4:02 PM"
        );
    }
}
