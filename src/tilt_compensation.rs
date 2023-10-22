use libm::{atan2f, atanf, cosf, sinf};
use lsm303agr::Measurement;

#[derive(Debug)]
pub struct Attitude {
    pub pitch: f32,
    pub roll: f32,
}

#[derive(Debug)]
pub struct NedMeasurement {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

//theta=0 at north, pi/-pi at south, pi/2 at east, and -pi/2 at west
pub struct Heading(pub f32);

/// board has forward in the -y direction and right in the -x direction, and down in the -z. (SWU),  algs for tilt compensation
/// need forward in +x and right in +y (this is known as the NED (north, east, down) cordinate
/// system)
/// also converts to f32
pub fn swd_to_ned(measurement: Measurement) -> NedMeasurement {
    NedMeasurement {
        x: measurement.y as f32,
        y: measurement.x as f32,
        z: -measurement.z as f32,
    }
}

pub fn calc_attitude(measurement: &NedMeasurement) -> Attitude {
    //based off of: https://www.nxp.com/docs/en/application-note/AN4248.pdf
    let roll = atan2f(measurement.y, measurement.z);
    let pitch = atanf(-measurement.x / (measurement.y * sinf(roll) + measurement.z * cosf(roll)));
    Attitude { pitch, roll }
    // Attitude { pitch: 0.0, roll: 0.0 }
}

pub fn calc_tilt_calibrated_measurement(
    mag_measurement: NedMeasurement,
    attitde: &Attitude,
) -> Heading {
    //based off of: https://www.nxp.com/docs/en/application-note/AN4248.pdf

    let corrected_mag_y = mag_measurement.z * sinf(attitde.roll)
        - mag_measurement.y * cosf(attitde.roll);

    let corrected_mag_x = mag_measurement.x * cosf(attitde.pitch)
        + mag_measurement.y * sinf(attitde.pitch) * sinf(attitde.roll)
        + mag_measurement.z * sinf(attitde.pitch) * cosf(attitde.roll);

    Heading(atan2f(-corrected_mag_y, corrected_mag_x))
}
