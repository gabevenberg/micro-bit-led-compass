use libm::{atan2f, atanf, cosf, sinf};

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

///theta=0 at north, pi/-pi at south, pi/2 at east, and -pi/2 at west
pub struct Heading(pub f32);

pub fn calc_attitude(measurement: &NedMeasurement) -> Attitude {
    //based off of: https://www.nxp.com/docs/en/application-note/AN4248.pdf
    let roll = atan2f(measurement.y, measurement.z);
    let pitch = atanf(-measurement.x / (measurement.y * sinf(roll) + measurement.z * cosf(roll)));
    Attitude { pitch, roll }
}

pub fn calc_tilt_calibrated_measurement(
    mag_measurement: NedMeasurement,
    attitde: &Attitude,
) -> NedMeasurement {
    //based off of: https://www.nxp.com/docs/en/application-note/AN4248.pdf

    let corrected_mag_y =
        mag_measurement.z * sinf(attitde.roll) - mag_measurement.y * cosf(attitde.roll);

    let corrected_mag_x = mag_measurement.x * cosf(attitde.pitch)
        + mag_measurement.y * sinf(attitde.pitch) * sinf(attitde.roll)
        + mag_measurement.z * sinf(attitde.pitch) * cosf(attitde.roll);

    NedMeasurement {
        x: corrected_mag_x,
        y: corrected_mag_y,
        z: 0.0,
    }
}

//0 is the top sector and positive is clockwise, negative is counterclockwise.
pub fn heading_from_measurement(measurement: NedMeasurement) -> Heading {
    Heading(atan2f(-measurement.y, measurement.x))
}

//I have no freaking clue how to test this...
