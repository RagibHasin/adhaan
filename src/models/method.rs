use chrono::{prelude::*, Duration};

use super::{Parameters, TimeAdjustment};
use crate::{
    astrolabe::{solar::SolarTime, unit::Angle},
    Coordinates,
};

/// Provide calculation method for fajr, isha and qiyam times.
#[allow(unused_variables)]
pub trait Method: std::fmt::Debug + std::panic::UnwindSafe + std::panic::RefUnwindSafe {
    /// Method intrinstic time adjustments
    fn adjustments(&self) -> TimeAdjustment {
        TimeAdjustment {
            dhuhr: 1,
            ..Default::default()
        }
    }

    /// Angles for fajr and isha calculation. Isha angle can be NaN for interval based calculation.
    fn angles(&self) -> (f64, f64) {
        (f64::NAN, f64::NAN)
    }

    /// Interval between maghrib and isha, if applicable.
    fn isha_interval(&self) -> Option<u8> {
        None
    }

    /// Calculate fajr time given all the parameters
    fn calculate_fajr(
        &self,
        parameters: &Parameters,
        solar_time: &SolarTime,
        night: Duration,
        coordinates: Coordinates,
        date: NaiveDate,
    ) -> DateTime<Utc> {
        let fajr = solar_time.time_for_solar_angle(Angle::new(-parameters.fajr_angle), false);

        let safe_fajr = solar_time.sunrise
            - Duration::milliseconds(
                (parameters.night_portions().0 * night.num_milliseconds() as f64) as _,
            );

        fajr.map_or(safe_fajr, |fajr| std::cmp::max(fajr, safe_fajr))
    }

    /// Calculate isha time given all the parameters
    fn calculate_isha(
        &self,
        parameters: &Parameters,
        solar_time: &SolarTime,
        night: Duration,
        coordinates: Coordinates,
        date: NaiveDate,
    ) -> DateTime<Utc> {
        let isha = solar_time.time_for_solar_angle(Angle::new(-parameters.isha_angle), true);

        let safe_isha = solar_time.sunset
            + Duration::milliseconds(
                (parameters.night_portions().1 * night.num_milliseconds() as f64) as _,
            );

        isha.map_or(safe_isha, |isha| std::cmp::min(isha, safe_isha))
    }
}

mod moonsighting_com;

/// Prominent calculation methods
#[allow(unused_variables)]
pub mod prominent_methods {
    use super::*;

    pub use moonsighting_com::{
        MoonsightingCommittee, MoonsightingCommitteeRedIsha, MoonsightingCommitteeWhiteIsha,
    };

    /// Common calculation method
    #[derive(Debug, PartialEq, Eq)]
    pub struct Common;

    impl Method for Common {}

    /// [Muslim World League](https://www.themwl.org/en)
    #[derive(Debug, PartialEq, Eq)]
    pub struct MuslimWorldLeague;

    impl Method for MuslimWorldLeague {
        fn angles(&self) -> (f64, f64) {
            (18.0, 17.0)
        }
    }

    /// Egyptian General Authority of Survey
    #[derive(Debug, PartialEq, Eq)]
    pub struct Egyptian;

    impl Method for Egyptian {
        fn angles(&self) -> (f64, f64) {
            (19.5, 17.5)
        }
    }

    /// University of Islamic Sciences, Karachi
    #[derive(Debug, PartialEq, Eq)]
    pub struct Karachi;

    impl Method for Karachi {
        fn angles(&self) -> (f64, f64) {
            (18.0, 18.0)
        }
    }

    /// Umm al-Qura University, Makkah
    #[derive(Debug, PartialEq, Eq)]
    pub struct UmmAlQura;

    impl Method for UmmAlQura {
        fn adjustments(&self) -> TimeAdjustment {
            Default::default()
        }

        fn angles(&self) -> (f64, f64) {
            (18.5, f64::NAN)
        }

        fn isha_interval(&self) -> Option<u8> {
            Some(90)
        }

        fn calculate_isha(
            &self,
            parameters: &Parameters,
            solar_time: &SolarTime,
            night: Duration,
            coordinates: Coordinates,
            prayer_date: NaiveDate,
        ) -> DateTime<Utc> {
            solar_time.sunset + Duration::minutes(parameters.isha_interval.unwrap_or(90) as _)
        }
    }

    /// The Gulf Region
    #[derive(Debug, PartialEq, Eq)]
    pub struct Dubai;

    impl Method for Dubai {
        fn adjustments(&self) -> TimeAdjustment {
            TimeAdjustment {
                sunrise: -3,
                dhuhr: 3,
                asr: 3,
                maghrib: 3,
                ..Default::default()
            }
        }

        fn angles(&self) -> (f64, f64) {
            (18.2, 18.2)
        }
    }

    /// ISNA
    #[derive(Debug, PartialEq, Eq)]
    pub struct NorthAmerica;

    impl Method for NorthAmerica {
        fn angles(&self) -> (f64, f64) {
            (15.0, 15.0)
        }
    }

    /// Kuwait
    #[derive(Debug, PartialEq, Eq)]
    pub struct Kuwait;

    impl Method for Kuwait {
        fn adjustments(&self) -> TimeAdjustment {
            Default::default()
        }

        fn angles(&self) -> (f64, f64) {
            (18.0, 17.5)
        }
    }

    /// Qatar
    #[derive(Debug, PartialEq, Eq)]
    pub struct Qatar;

    impl Method for Qatar {
        fn adjustments(&self) -> TimeAdjustment {
            Default::default()
        }

        fn angles(&self) -> (f64, f64) {
            (18.0, f64::NAN)
        }

        fn isha_interval(&self) -> Option<u8> {
            Some(90)
        }

        fn calculate_isha(
            &self,
            parameters: &Parameters,
            solar_time: &SolarTime,
            night: Duration,
            coordinates: Coordinates,
            prayer_date: NaiveDate,
        ) -> DateTime<Utc> {
            UmmAlQura.calculate_isha(parameters, solar_time, night, coordinates, prayer_date)
        }
    }

    /// Singapore
    #[derive(Debug, PartialEq, Eq)]
    pub struct Singapore;

    impl Method for Singapore {
        fn angles(&self) -> (f64, f64) {
            (20.0, 18.0)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use approx::assert_abs_diff_eq;

    #[test]
    fn parameters_for_muslim_world_league() {
        let method = prominent_methods::MuslimWorldLeague;

        assert_abs_diff_eq!(method.angles().0, 18.0, epsilon = 1e-10);
        assert_abs_diff_eq!(method.angles().1, 17.0, epsilon = 1e-10);
        assert_eq!(method.isha_interval(), None);
    }

    #[test]
    fn parameters_for_egyptian() {
        let method = prominent_methods::Egyptian;

        assert_abs_diff_eq!(method.angles().0, 19.5, epsilon = 1e-10);
        assert_abs_diff_eq!(method.angles().1, 17.5, epsilon = 1e-10);
        assert_eq!(method.isha_interval(), None);
    }

    #[test]
    fn parameters_for_karachi() {
        let method = prominent_methods::Karachi;

        assert_abs_diff_eq!(method.angles().0, 18.0, epsilon = 1e-10);
        assert_abs_diff_eq!(method.angles().1, 18.0, epsilon = 1e-10);
        assert_eq!(method.isha_interval(), None);
    }

    #[test]
    fn parameters_for_umm_al_qura() {
        let method = prominent_methods::UmmAlQura;

        assert_abs_diff_eq!(method.angles().0, 18.5, epsilon = 1e-10);
        assert!(method.angles().1.is_nan());
        assert_eq!(method.isha_interval(), Some(90));
    }

    #[test]
    fn parameters_for_dubai() {
        let method = prominent_methods::Dubai;

        assert_abs_diff_eq!(method.angles().0, 18.2, epsilon = 1e-10);
        assert_abs_diff_eq!(method.angles().1, 18.2, epsilon = 1e-10);
        assert_eq!(method.isha_interval(), None);
    }

    #[test]
    fn parameters_for_moonsighting_committee() {
        let method = prominent_methods::MoonsightingCommittee;

        assert_abs_diff_eq!(method.angles().0, 18.0, epsilon = 1e-10);
        assert_abs_diff_eq!(method.angles().1, 18.0, epsilon = 1e-10);
        assert_eq!(method.isha_interval(), None);
    }

    #[test]
    fn parameters_for_north_america() {
        let method = prominent_methods::NorthAmerica;

        assert_abs_diff_eq!(method.angles().0, 15.0, epsilon = 1e-10);
        assert_abs_diff_eq!(method.angles().1, 15.0, epsilon = 1e-10);
        assert_eq!(method.isha_interval(), None);
    }

    #[test]
    fn parameters_for_kuwait() {
        let method = prominent_methods::Kuwait;

        assert_abs_diff_eq!(method.angles().0, 18.0, epsilon = 1e-10);
        assert_abs_diff_eq!(method.angles().1, 17.5, epsilon = 1e-10);
        assert_eq!(method.isha_interval(), None);
    }

    #[test]
    fn parameters_for_qatar() {
        let method = prominent_methods::Qatar;

        assert_abs_diff_eq!(method.angles().0, 18.0, epsilon = 1e-10);
        assert!(method.angles().1.is_nan());
        assert_eq!(method.isha_interval(), Some(90));
    }

    #[test]
    fn parameters_for_singapore() {
        let method = prominent_methods::Singapore;

        assert_abs_diff_eq!(method.angles().0, 20.0, epsilon = 1e-10);
        assert_abs_diff_eq!(method.angles().1, 18.0, epsilon = 1e-10);
        assert_eq!(method.isha_interval(), None);
    }
}
