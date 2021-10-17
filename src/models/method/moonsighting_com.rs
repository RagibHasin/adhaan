use super::*;

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
enum IshaOption {
    Mixed,
    Redness,
    Whiteness,
}

/// Calculation method of [Moonsighting Committee](https://www.moonsighting.com/)
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub struct MoonsightingCom(IshaOption);

fn adjust(a: f64, b: f64, c: f64, d: f64, dyy: u32) -> f64 {
    match dyy {
        0..=90 => a + (b - a) / 91.0 * dyy as f64,
        91..=136 => b + (c - b) / 46.0 * (dyy - 91) as f64,
        137..=182 => c + (d - c) / 46.0 * (dyy - 137) as f64,
        183..=228 => d + (c - d) / 46.0 * (dyy - 183) as f64,
        229..=274 => c + (b - c) / 46.0 * (dyy - 229) as f64,
        _ => b + (a - b) / 91.0 * (dyy - 275) as f64,
    }
}

/// Twilight adjustment based on observational data.
fn season_adjusted_morning_twilight(date: NaiveDate, latitude: f64) -> Duration {
    let a = 75.0 + ((28.65 / 55.0) * latitude.abs());
    let b = 75.0 + ((19.44 / 55.0) * latitude.abs());
    let c = 75.0 + ((32.74 / 55.0) * latitude.abs());
    let d = 75.0 + ((48.10 / 55.0) * latitude.abs());

    Duration::minutes(adjust(a, b, c, d, days_since_solstice(date, latitude)).round() as _)
}

/// Twilight adjustment based on observational data.
fn season_adjusted_evening_twilight(date: NaiveDate, latitude: f64) -> Duration {
    let a = 75.0 + 25.60 / 55.0 * latitude.abs();
    let b = 75.0 + 2.050 / 55.0 * latitude.abs();
    let c = 75.0 - 9.210 / 55.0 * latitude.abs();
    let d = 75.0 + 6.140 / 55.0 * latitude.abs();

    Duration::minutes(adjust(a, b, c, d, days_since_solstice(date, latitude)).round() as _)
}

/// Twilight adjustment based on observational data.
fn season_adjusted_evening_twilight_white(date: NaiveDate, latitude: f64) -> Duration {
    let a = 75.0 + 25.60 / 55.0 * latitude.abs();
    let b = 75.0 + 7.160 / 55.0 * latitude.abs();
    let c = 75.0 + 36.84 / 55.0 * latitude.abs();
    let d = 75.0 + 81.84 / 55.0 * latitude.abs();

    Duration::minutes(adjust(a, b, c, d, days_since_solstice(date, latitude)).round() as _)
}

/// Twilight adjustment based on observational data.
fn season_adjusted_evening_twilight_red(date: NaiveDate, latitude: f64) -> Duration {
    let a = 62.0 + 17.40 / 55.0 * latitude.abs();
    let b = 62.0 - 7.160 / 55.0 * latitude.abs();
    let c = 62.0 + 5.120 / 55.0 * latitude.abs();
    let d = 62.0 + 19.44 / 55.0 * latitude.abs();

    Duration::minutes(adjust(a, b, c, d, days_since_solstice(date, latitude)).round() as _)
}

/// Solstice calculation to determine a date's seasonal progression.
fn days_since_solstice(date: NaiveDate, latitude: f64) -> u32 {
    (date
        - date
            .with_day(21)
            .unwrap()
            .with_month(if latitude >= 0.0 { 12 } else { 6 })
            .unwrap())
    .num_days()
    .abs() as _
}

impl Method for MoonsightingCom {
    fn adjustments(&self) -> TimeAdjustment {
        TimeAdjustment {
            dhuhr: 5,
            maghrib: 3,
            ..Default::default()
        }
    }

    fn angles(&self) -> (f64, f64) {
        (18.0, 18.0)
    }

    fn calculate_fajr(
        &self,
        parameters: &Parameters,
        solar_time: &SolarTime,
        night: Duration,
        coordinates: Coordinates,
        prayer_date: NaiveDate,
    ) -> DateTime<Utc> {
        let fajr = if coordinates.latitude.abs() >= 55.0 {
            solar_time.sunrise - night / 7
        } else {
            solar_time
                .time_for_solar_angle(Angle::new(-parameters.fajr_angle), false)
                .unwrap()
        };

        let safe_fajr = solar_time.sunrise
            - season_adjusted_morning_twilight(prayer_date, coordinates.latitude);

        std::cmp::max(fajr, safe_fajr)
    }

    fn calculate_isha(
        &self,
        parameters: &Parameters,
        solar_time: &SolarTime,
        night: Duration,
        coordinates: Coordinates,
        prayer_date: NaiveDate,
    ) -> DateTime<Utc> {
        let isha = if coordinates.latitude.abs() >= 55.0 {
            solar_time.sunset + night / 7
        } else {
            solar_time
                .time_for_solar_angle(Angle::new(-parameters.isha_angle), true)
                .unwrap()
        };

        let safe_isha = solar_time.sunset
            + match self.0 {
                IshaOption::Mixed => season_adjusted_evening_twilight,
                IshaOption::Redness => season_adjusted_evening_twilight_red,
                IshaOption::Whiteness => season_adjusted_evening_twilight_white,
            }(prayer_date, coordinates.latitude);

        std::cmp::min(isha, safe_isha)
    }
}

/// Calculation method of [Moonsighting Committee](https://www.moonsighting.com/).
#[allow(non_upper_case_globals)]
pub static MoonsightingCommittee: MoonsightingCom = MoonsightingCom(IshaOption::Mixed);

/// Calculation method of [Moonsighting Committee](https://www.moonsighting.com/).
/// Isha ends when redness recedes (Shafaq Ahmar).
#[allow(non_upper_case_globals)]
pub static MoonsightingCommitteeRedIsha: MoonsightingCom = MoonsightingCom(IshaOption::Redness);

/// Calculation method of [Moonsighting Committee](https://www.moonsighting.com/).
/// Isha ends when whiteness recedes (Shafaq Abyad).
#[allow(non_upper_case_globals)]
pub static MoonsightingCommitteeWhiteIsha: MoonsightingCom = MoonsightingCom(IshaOption::Whiteness);
