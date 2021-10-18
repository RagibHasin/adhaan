//! # Prayer Schedule
//!
//! This module provide the main objects that are used for calculating
//! the prayer times.

use chrono::{prelude::*, Duration};

use crate::{
    astrolabe::{solar::SolarTime, unit::Coordinates},
    models::{Parameters, Prayer},
    PolarCircleResolution,
};

/// A data struct to hold the timing for all prayers.
#[derive(PartialEq, Debug, Clone)]
pub struct PrayerTimes {
    qiyam_yesterday: DateTime<Utc>,
    fajr: DateTime<Utc>,
    sunrise: DateTime<Utc>,
    dhuhr: DateTime<Utc>,
    asr_awwal: DateTime<Utc>,
    asr_thaani: DateTime<Utc>,
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

        let asr_awwal = solar_time.afternoon(1.0)?
            + Duration::minutes(parameters.time_adjustments(Prayer::AsrAwwal));

        let asr_thaani = solar_time.afternoon(2.0)?
            + Duration::minutes(parameters.time_adjustments(Prayer::AsrThaani));

        let maghrib =
            solar_time.sunset + Duration::minutes(parameters.time_adjustments(Prayer::Maghrib));

        let isha = parameters.method.calculate_isha(
            &parameters,
            &solar_time,
            next_night,
            coordinates,
            date,
        ) + Duration::minutes(parameters.time_adjustments(Prayer::Isha));

        let fajr_tomorrow = parameters.method.calculate_fajr(
            &parameters,
            &solar_time_tomorrow,
            next_night,
            coordinates,
            tomorrow,
        ) + Duration::minutes(parameters.time_adjustments(Prayer::Fajr));

        let next_night_short = fajr_tomorrow - maghrib;
        let middle_of_night = maghrib + next_night_short / 2;
        let last_third_of_night = maghrib + next_night_short * 2 / 3;

        let maghrib_yesterday = solar_time_yesterday.sunset
            + Duration::minutes(parameters.time_adjustments(Prayer::Maghrib));

        let last_night_short = fajr - maghrib_yesterday;
        let last_third_of_last_night = maghrib_yesterday + last_night_short * 2 / 3;

        Some(PrayerTimes {
            qiyam_yesterday: last_third_of_last_night,
            fajr,
            sunrise,
            dhuhr,
            asr_awwal,
            asr_thaani,
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
    pub fn time_of(&self, prayer: Prayer) -> Option<DateTime<Utc>> {
        use chrono::DurationRound;
        use Prayer::*;
        let one_minute = Duration::minutes(1);
        match prayer {
            QiyamYesterday => Some(self.qiyam_yesterday.duration_trunc(one_minute).unwrap()),
            Fajr => Some(self.fajr.duration_trunc(one_minute).unwrap()),
            Sunrise => Some(self.sunrise.duration_trunc(one_minute).unwrap()),
            Dhuhr => Some(self.dhuhr.duration_round(one_minute).unwrap()),
            AsrAwwal => Some(self.asr_awwal.duration_round(one_minute).unwrap()),
            AsrThaani => Some(self.asr_thaani.duration_round(one_minute).unwrap()),
            Maghrib => Some(self.maghrib.duration_round(one_minute).unwrap()),
            Isha => Some(self.isha.duration_round(one_minute).unwrap()),
            Qiyam => Some(self.qiyam.duration_trunc(one_minute).unwrap()),
            _ => None,
        }
    }

    // /// Time remaining for the current prayer at given time
    // pub fn time_remaining(&self, now: DateTime<Utc>) -> Option<Duration> {
    //     self.prayer_at(now)
    //         .next()
    //         .map(|prayer_next| self.time_of(prayer_next) - now)
    // }

    /// Current prayer at given time in the date
    pub fn prayer_at(&self, time: DateTime<Utc>) -> Prayer {
        if time >= self.fajr_tomorrow {
            Prayer::Tomorrow
        } else if time >= self.qiyam {
            Prayer::Qiyam
        } else if time >= self.isha {
            Prayer::Isha
        } else if time >= self.maghrib {
            Prayer::Maghrib
        } else if time >= self.asr_thaani {
            Prayer::AsrThaani
        } else if time >= self.asr_awwal {
            Prayer::AsrAwwal
        } else if time >= self.dhuhr {
            Prayer::Dhuhr
        } else if time >= self.sunrise {
            Prayer::Sunrise
        } else if time >= self.fajr {
            Prayer::Fajr
        } else if time >= self.qiyam_yesterday {
            Prayer::QiyamYesterday
        } else {
            Prayer::Yesterday
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::prominent_methods;

    macro_rules! prayer_should_be {
        ($prayer:ident, $hour:expr ; $minute:expr) => {
            paste::paste! {
                #[test]
                #[allow(clippy::zero_prefixed_literal)]
                fn [< current_prayer_should_be_ $prayer:snake >]() {
                    // Given the above DateTime, the Fajr prayer is at 2015-07-12T08:42:00Z
                    let local_date = NaiveDate::from_ymd(2015, 7, 12);
                    let params = Parameters::new(&prominent_methods::NorthAmerica);
                    let coordinates = Coordinates::new(35.7750, -78.6336);
                    let times = PrayerTimes::calculate(local_date, coordinates, params).unwrap();
                    //eprintln!("{:#?}", times);
                    let current_prayer_time = Utc.from_utc_date(&local_date).and_hms(0, 0, 0)
                        + Duration::hours($hour)
                        + Duration::minutes($minute);

                    assert_eq!(times.prayer_at(current_prayer_time), Prayer::$prayer);
                }
            }
        };
    }

    prayer_should_be!(Yesterday, 01;55);
    prayer_should_be!(QiyamYesterday, 06;00);
    prayer_should_be!(Fajr, 09;00);
    prayer_should_be!(Sunrise, 11;00);
    prayer_should_be!(Dhuhr, 19;00);
    prayer_should_be!(AsrAwwal, 21;10);
    prayer_should_be!(AsrThaani, 22;25);
    prayer_should_be!(Maghrib, 24;35);
    prayer_should_be!(Isha, 26;00);
    prayer_should_be!(Qiyam, 30;00);
    prayer_should_be!(Tomorrow, 32;45);

    #[test]
    fn calculate_times_for_moonsighting_method() {
        let date = NaiveDate::from_ymd(2016, 1, 31);
        let params = Parameters::new(&prominent_methods::MoonsightingCommittee);
        let coordinates = Coordinates::new(35.7750, -78.6336);
        let schedule = PrayerTimes::calculate(date, coordinates, params).unwrap();

        // fajr    = 2016-01-31 10:48:00 UTC
        // sunrise = 2016-01-31 12:16:00 UTC
        // dhuhr   = 2016-01-31 17:33:00 UTC
        // asr     = 2016-01-31 20:20:00 UTC
        // maghrib = 2016-01-31 22:43:00 UTC
        // isha    = 2016-02-01 00:05:00 UTC
        assert_eq!(
            schedule.time_of(Prayer::Fajr).unwrap(),
            Utc.ymd(2016, 1, 31).and_hms(10, 48, 0)
        );
        assert_eq!(
            schedule.time_of(Prayer::Sunrise).unwrap(),
            Utc.ymd(2016, 1, 31).and_hms(12, 15, 0)
        );
        assert_eq!(
            schedule.time_of(Prayer::Dhuhr).unwrap(),
            Utc.ymd(2016, 1, 31).and_hms(17, 33, 0)
        );
        assert_eq!(
            schedule.time_of(Prayer::AsrAwwal).unwrap(),
            Utc.ymd(2016, 1, 31).and_hms(20, 20, 0)
        );
        assert_eq!(
            schedule.time_of(Prayer::Maghrib).unwrap(),
            Utc.ymd(2016, 1, 31).and_hms(22, 43, 0)
        );
        assert_eq!(
            schedule.time_of(Prayer::Isha).unwrap(),
            Utc.ymd(2016, 2, 1).and_hms(0, 5, 0)
        );
    }

    #[test]
    fn calculate_times_for_moonsighting_method_with_high_latitude() {
        let date = NaiveDate::from_ymd(2016, 1, 1);
        let params = Parameters::new(&prominent_methods::MoonsightingCommittee);
        let coordinates = Coordinates::new(59.9094, 10.7349);
        let schedule = PrayerTimes::calculate(date, coordinates, params).unwrap();

        // fajr    = 2016-01-01 06:34:00 UTC
        // sunrise = 2016-01-01 08:19:00 UTC
        // dhuhr   = 2016-01-01 11:25:00 UTC
        // asr     = 2016-01-01 12:36:00 UTC
        // maghrib = 2016-01-01 14:25:00 UTC
        // isha    = 2016-01-01 16:02:00 UTC
        assert_eq!(
            schedule.time_of(Prayer::Fajr).unwrap(),
            Utc.ymd(2016, 1, 1).and_hms(6, 33, 0)
        );
        assert_eq!(
            schedule.time_of(Prayer::Sunrise).unwrap(),
            Utc.ymd(2016, 1, 1).and_hms(8, 18, 0)
        );
        assert_eq!(
            schedule.time_of(Prayer::Dhuhr).unwrap(),
            Utc.ymd(2016, 1, 1).and_hms(11, 25, 0)
        );
        assert_eq!(
            schedule.time_of(Prayer::AsrThaani).unwrap(),
            Utc.ymd(2016, 1, 1).and_hms(12, 36, 0)
        );
        assert_eq!(
            schedule.time_of(Prayer::Maghrib).unwrap(),
            Utc.ymd(2016, 1, 1).and_hms(14, 25, 0)
        );
        assert_eq!(
            schedule.time_of(Prayer::Isha).unwrap(),
            Utc.ymd(2016, 1, 1).and_hms(16, 2, 0)
        );
    }
}
