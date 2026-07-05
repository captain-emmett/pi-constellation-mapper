use crate::catalog::Star;

#[derive(Clone, Copy, Debug)]
pub struct Observer {
    pub latitude_deg: f64,
    pub longitude_deg: f64,
}

#[derive(Clone, Copy, Debug)]
pub struct HorizontalCoordinates {
    pub azimuth_deg: f64,
    pub altitude_deg: f64,
}

pub fn equatorial_to_horizontal(
    star: &Star,
    observer: Observer,
    unix_seconds: f64,
) -> HorizontalCoordinates {
    let latitude = observer.latitude_deg.to_radians();
    let declination = star.declination_deg.to_radians();
    let local_sidereal_deg =
        (greenwich_sidereal_degrees(unix_seconds) + observer.longitude_deg).rem_euclid(360.0);
    let hour_angle =
        normalize_signed_degrees(local_sidereal_deg - star.right_ascension_hours * 15.0)
            .to_radians();

    let sin_altitude =
        declination.sin() * latitude.sin() + declination.cos() * latitude.cos() * hour_angle.cos();
    let altitude = sin_altitude.clamp(-1.0, 1.0).asin();

    // Azimuth is measured clockwise from true north.
    let azimuth = (-hour_angle.sin() * declination.cos()).atan2(
        declination.sin() * latitude.cos() - declination.cos() * latitude.sin() * hour_angle.cos(),
    );

    HorizontalCoordinates {
        azimuth_deg: azimuth.to_degrees().rem_euclid(360.0),
        altitude_deg: altitude.to_degrees(),
    }
}

pub fn greenwich_sidereal_degrees(unix_seconds: f64) -> f64 {
    let julian_date = unix_seconds / 86_400.0 + 2_440_587.5;
    let days_since_j2000 = julian_date - 2_451_545.0;
    (280.460_618_37 + 360.985_647_366_29 * days_since_j2000).rem_euclid(360.0)
}

fn normalize_signed_degrees(value: f64) -> f64 {
    (value + 180.0).rem_euclid(360.0) - 180.0
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::catalog::Star;

    const TEST_STAR: Star = Star {
        name: "Test star",
        right_ascension_hours: 0.0,
        declination_deg: 0.0,
        magnitude: 1.0,
        color_rgb: (255, 255, 255),
    };

    #[test]
    fn unix_epoch_has_expected_julian_sidereal_value() {
        let value = greenwich_sidereal_degrees(0.0);
        assert!((value - 100.229).abs() < 0.01);
    }

    #[test]
    fn star_on_local_meridian_is_at_zenith_for_matching_latitude() {
        let unix_seconds = 946_728_000.0; // J2000: 2000-01-01 12:00 UTC
        let mut star = TEST_STAR;
        star.right_ascension_hours = greenwich_sidereal_degrees(unix_seconds) / 15.0;

        let horizontal = equatorial_to_horizontal(
            &star,
            Observer {
                latitude_deg: 0.0,
                longitude_deg: 0.0,
            },
            unix_seconds,
        );

        assert!((horizontal.altitude_deg - 90.0).abs() < 0.001);
    }
}
