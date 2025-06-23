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
    //Gp{xyz} is the acellerometer measurements.
    let roll = atan2f(measurement.y, measurement.z);
    let pitch = atanf(-measurement.x / (measurement.y * sinf(roll) + measurement.z * cosf(roll)));
    Attitude { pitch, roll }
}

pub fn calc_tilt_calibrated_measurement(
    mag_measurement: NedMeasurement,
    attitde: &Attitude,
) -> NedMeasurement {
    //based off of: https://www.nxp.com/docs/en/application-note/AN4248.pdf
    // ø=roll,
    // θ=pitch,
    // Bp{xyz} is magnometer readings,
    // V{xyz} is the magnometers constant hard-iron compnent, (currently taken care by micro:bits
    // library)

    let corrected_mag_x = mag_measurement.x * cosf(attitde.pitch)
        + mag_measurement.y * sinf(attitde.pitch) * sinf(attitde.roll)
        + mag_measurement.z * sinf(attitde.pitch) * cosf(attitde.roll);
    let corrected_mag_y =
        mag_measurement.y * cosf(attitde.roll) - mag_measurement.z * sinf(attitde.roll);
    let corrected_mag_z = -mag_measurement.x * sinf(attitde.pitch)
        + mag_measurement.y * cosf(attitde.pitch) * sinf(attitde.roll)
        + mag_measurement.z * cosf(attitde.pitch) * cosf(attitde.roll);

    NedMeasurement {
        x: corrected_mag_x,
        y: corrected_mag_y,
        z: corrected_mag_z,
    }
}

//0 is the top sector and positive is clockwise, negative is counterclockwise.
pub fn heading_from_measurement(measurement: &NedMeasurement) -> Heading {
    Heading(atan2f(-measurement.y, measurement.x))
}

//I have no freaking clue how to test this...
