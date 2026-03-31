use jiff::{Span, civil::Date};

use crate::{Coordinates, astrolabe::solar::SolarTime};

/// Resolution rule for much higher latitudes.
#[derive(PartialEq, Eq, Debug, Copy, Clone)]
pub enum PolarCircleResolver {
    /// Resolve to nearest city with valid times.
    NearestCity,
    /// Resolve to nearest day with valid times.
    NearestDay,
}

const LATITUDE_VARIATION_STEP: f64 = 0.5; // Degrees to add/remove at each resolution step
const UNSAFE_LATITUDE: f64 = 65.0; // Based on https://en.wikipedia.org/wiki/Midnight_sun

/// Result of a polar circle resolution.
#[derive(PartialEq, Debug, Clone)]
pub(crate) struct PolarCircleResolution {
    pub date: Date,
    pub coordinates: Coordinates,
    pub today: SolarTime,
    pub yesterday: SolarTime,
    pub tomorrow: SolarTime,
}

impl PolarCircleResolver {
    /// Resolves a polar circle problem.
    pub(crate) fn resolve(
        self,
        date: Date,
        coordinates: Coordinates,
    ) -> Result<PolarCircleResolution, PolarCircleResolutionError> {
        if let Ok(today) = SolarTime::new(date, coordinates)
            && let Ok(yesterday) =
                SolarTime::new(date.yesterday().expect("sensible date"), coordinates)
            && let Ok(tomorrow) =
                SolarTime::new(date.tomorrow().expect("sensible date"), coordinates)
        {
            Ok(PolarCircleResolution {
                date,
                coordinates,
                today,
                yesterday,
                tomorrow,
            })
        } else {
            match self {
                Self::NearestDay => resolve_for_nearest_day(coordinates, date, 1, true),
                Self::NearestCity => resolve_for_nearest_city(
                    coordinates.longitude,
                    date,
                    coordinates.latitude - (LATITUDE_VARIATION_STEP.copysign(coordinates.latitude)),
                ),
            }
        }
    }
}

fn resolve_for_nearest_day(
    coordinates: Coordinates,
    date: Date,
    delta_days: i16,
    forward: bool,
) -> Result<PolarCircleResolution, PolarCircleResolutionError> {
    if delta_days > 183 {
        return Err(PolarCircleResolutionError::NearestDayWontWork);
    }

    let test_date = date + Span::new().days(if forward { delta_days } else { -delta_days });

    if let Ok(today) = SolarTime::new(test_date, coordinates)
        && let Ok(yesterday) =
            SolarTime::new(test_date.yesterday().expect("sensible date"), coordinates)
        && let Ok(tomorrow) =
            SolarTime::new(test_date.tomorrow().expect("sensible date"), coordinates)
    {
        Ok(PolarCircleResolution {
            date,
            coordinates,
            today,
            yesterday,
            tomorrow,
        })
    } else {
        resolve_for_nearest_day(
            coordinates,
            date,
            delta_days + (if forward { 0 } else { 1 }),
            !forward,
        )
    }
}

fn resolve_for_nearest_city(
    longitude: f64,
    date: Date,
    latitude: f64,
) -> Result<PolarCircleResolution, PolarCircleResolutionError> {
    let coordinates = Coordinates {
        latitude,
        longitude,
    };

    if let Ok(today) = SolarTime::new(date, coordinates)
        && let Ok(yesterday) = SolarTime::new(date.yesterday().expect("sensible date"), coordinates)
        && let Ok(tomorrow) = SolarTime::new(date.tomorrow().expect("sensible date"), coordinates)
    {
        Ok(PolarCircleResolution {
            date,
            coordinates,
            today,
            yesterday,
            tomorrow,
        })
    } else {
        if latitude.abs() >= UNSAFE_LATITUDE {
            resolve_for_nearest_city(
                longitude,
                date,
                latitude - LATITUDE_VARIATION_STEP.copysign(latitude),
            )
        } else {
            Err(PolarCircleResolutionError::NearestCityWontWork)
        }
    }
}

/// Error that may arise when doing polar circle resolution.
#[derive(Debug)]
pub enum PolarCircleResolutionError {
    /// The nearest day method of polar circle resolution won't work at the specified coordinates
    /// on the specified date.
    NearestDayWontWork,

    /// The nearest city method of polar circle resolution won't work at the specified coordinates
    /// on the specified date.
    NearestCityWontWork,
}

impl std::error::Error for PolarCircleResolutionError {}

impl std::fmt::Display for PolarCircleResolutionError {
    fn fmt(&self, fmt: &mut std::fmt::Formatter) -> std::fmt::Result {
        fmt.write_str(match self {
            Self::NearestDayWontWork => "nearest day method of polar circle resolution won't work",
            Self::NearestCityWontWork => {
                "nearest city method of polar circle resolution won't work"
            }
        })
    }
}
