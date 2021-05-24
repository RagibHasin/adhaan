use chrono::{prelude::*, Duration};

use crate::{astrolabe::solar::SolarTime, Coordinates};

/// Resolution rule for much higher latitudes.
#[derive(PartialEq, Eq, Debug, Copy, Clone)]
pub enum PolarCircleResolver {
    /// Resolve to nearest city with valid times
    NearestCity,
    /// Resolve to nearest day with valid times
    NearestDay,
}

const LATITUDE_VARIATION_STEP: f64 = 0.5; // Degrees to add/remove at each resolution step
const UNSAFE_LATITUDE: f64 = 65.0; // Based on https://en.wikipedia.org/wiki/Midnight_sun

/// Result of a polar circle resolution
#[derive(PartialEq, Debug, Clone)]
#[allow(missing_docs)]
pub(crate) struct PolarCircleResolution {
    pub date: NaiveDate,
    pub coordinates: Coordinates,
    pub solar_time: Option<SolarTime>,
    pub solar_time_tomorrow: Option<SolarTime>,
    pub solar_time_yesterday: Option<SolarTime>,
}

impl PolarCircleResolver {
    /// Resolve a polar circle problem
    pub(crate) fn resolve(
        self,
        date: NaiveDate,
        coordinates: Coordinates,
    ) -> PolarCircleResolution {
        let default = || PolarCircleResolution {
            date,
            coordinates,
            solar_time: SolarTime::new(date, coordinates),
            solar_time_tomorrow: SolarTime::new(date.succ(), coordinates),
            solar_time_yesterday: SolarTime::new(date.pred(), coordinates),
        };

        match self {
            Self::NearestDay => {
                resolve_for_nearest_day(coordinates, date, 1, true).unwrap_or_else(default)
            }
            Self::NearestCity => resolve_for_nearest_city(
                coordinates,
                date,
                coordinates.latitude - (LATITUDE_VARIATION_STEP.copysign(coordinates.latitude)),
            )
            .unwrap_or_else(default),
        }
    }
}

fn resolve_for_nearest_day(
    coordinates: Coordinates,
    date: NaiveDate,
    days_added: i16,
    forward: bool,
) -> Option<PolarCircleResolution> {
    if days_added > 183 {
        return None;
    }

    let test_date = date + Duration::days(if forward { days_added } else { -days_added } as _);
    let solar_time = SolarTime::new(test_date, coordinates);
    let solar_time_tomorrow = SolarTime::new(test_date.succ(), coordinates);
    let solar_time_yesterday = SolarTime::new(test_date.pred(), coordinates);

    if solar_time.is_none() || solar_time_tomorrow.is_none() || solar_time_yesterday.is_none() {
        resolve_for_nearest_day(
            coordinates,
            date,
            days_added + (if forward { 0 } else { 1 }),
            !forward,
        )
    } else {
        Some(PolarCircleResolution {
            date,
            coordinates,
            solar_time,
            solar_time_tomorrow,
            solar_time_yesterday,
        })
    }
}

fn resolve_for_nearest_city(
    coordinates: Coordinates,
    date: NaiveDate,
    latitude: f64,
) -> Option<PolarCircleResolution> {
    let coordinates = Coordinates {
        latitude,
        ..coordinates
    };
    let solar_time = SolarTime::new(date, coordinates);
    let solar_time_tomorrow = SolarTime::new(date.succ(), coordinates);
    let solar_time_yesterday = SolarTime::new(date.pred(), coordinates);

    if solar_time.is_none() || solar_time_tomorrow.is_none() || solar_time_yesterday.is_none() {
        (latitude.abs() >= UNSAFE_LATITUDE)
            .then(|| {
                resolve_for_nearest_city(
                    coordinates,
                    date,
                    latitude - LATITUDE_VARIATION_STEP.copysign(latitude),
                )
            })
            .flatten()
    } else {
        Some(PolarCircleResolution {
            date,
            coordinates,
            solar_time,
            solar_time_tomorrow,
            solar_time_yesterday,
        })
    }
}
