use super::*;

/// Settings that are used for determining the
/// the correct prayer time.
///
/// It is recommended to use [Configuration](struct.Configuration.html) to build
/// the parameters that are need.
#[derive(Debug, Clone, Copy)]
pub struct Parameters {
    /// Calculation method
    pub method: &'static dyn Method,
    /// Fajr angle
    pub fajr_angle: f64,
    /// Isha angle
    pub isha_angle: f64,
    /// Isha interval, if applicable
    pub isha_interval: Option<u8>,
    /// High latitude rule
    pub high_latitude_rule: HighLatitudeRule,
    /// User adjustments
    pub user_adjustments: TimeAdjustment,
    /// Polar circle resolution rule
    pub polar_circle_resolver: PolarCircleResolver,
}

impl Parameters {
    /// Make a new set of parameters with calculation method and madhhab
    pub fn new(method: &'static dyn Method) -> Self {
        let (fajr_angle, isha_angle) = method.angles();
        Parameters {
            fajr_angle,
            isha_angle,
            method,
            isha_interval: method.isha_interval(),
            high_latitude_rule: HighLatitudeRule::MiddleOfTheNight,
            user_adjustments: TimeAdjustment::default(),
            polar_circle_resolver: PolarCircleResolver::NearestCity,
        }
    }

    /// Make a new set of parameters with the given one and new angle values
    pub fn with_angles(self, fajr_angle: f64, isha_angle: f64) -> Self {
        Parameters {
            fajr_angle,
            isha_angle,
            ..self
        }
    }

    /// Make a new set of parameters with the given one and new adjustments
    pub fn with_adjustments(self, adjustments: TimeAdjustment) -> Self {
        Parameters {
            user_adjustments: adjustments,
            ..self
        }
    }

    /// Make a new set of parameters with the given one and new high latitude rules
    pub fn with_high_latitude_rule(self, rule: HighLatitudeRule) -> Self {
        Parameters {
            high_latitude_rule: rule,
            ..self
        }
    }

    /// Make a new set of parameters with the given one and new polar circle resolver
    pub fn with_polar_circle_resolver(self, polar_circle_resolver: PolarCircleResolver) -> Self {
        Parameters {
            polar_circle_resolver,
            ..self
        }
    }

    /// Get night portions for fajr and isha
    pub fn night_portions(&self) -> (f64, f64) {
        match self.high_latitude_rule {
            HighLatitudeRule::MiddleOfTheNight => (0.5, 0.5),
            HighLatitudeRule::SeventhOfTheNight => (1. / 7., 1. / 7.),
            HighLatitudeRule::TwilightAngle => (self.fajr_angle / 60.0, self.isha_angle / 60.0),
        }
    }

    /// Get total time adjustment for a specific prayer
    pub fn time_adjustments(&self, prayer: Prayer) -> i64 {
        match prayer {
            Prayer::Fajr => self.user_adjustments.fajr + self.method.adjustments().fajr,
            Prayer::Sunrise => self.user_adjustments.sunrise + self.method.adjustments().sunrise,
            Prayer::Dhuhr => self.user_adjustments.dhuhr + self.method.adjustments().dhuhr,
            Prayer::AsrAwwal | Prayer::AsrThaani => {
                self.user_adjustments.asr + self.method.adjustments().asr
            }
            Prayer::Maghrib => self.user_adjustments.maghrib + self.method.adjustments().maghrib,
            Prayer::Isha => self.user_adjustments.isha + self.method.adjustments().isha,
            _ => 0,
        }
    }
}

impl PartialEq for Parameters {
    fn eq(&self, other: &Parameters) -> bool {
        self.fajr_angle == other.fajr_angle
            && self.isha_angle == other.isha_angle
            && self.isha_interval == other.isha_interval
            && self.high_latitude_rule == other.high_latitude_rule
            && self.user_adjustments == other.user_adjustments
            && self.method.adjustments() == other.method.adjustments()
    }
}
