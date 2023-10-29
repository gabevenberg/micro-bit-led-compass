#![deny(unsafe_code)]
#![no_main]
#![no_std]

use core::f32::consts::PI;

use calibration::Calibration;
use cortex_m_rt::entry;
use lsm303agr::interface::I2cInterface;
use lsm303agr::mode::MagContinuous;
use lsm303agr::{AccelOutputDataRate, Lsm303agr, MagOutputDataRate, Measurement};
use microbit::hal::{gpiote::Gpiote, Twim};
use microbit::pac::TWIM0;
use panic_rtt_target as _;
use rtt_target::{rprintln, rtt_init_print};

mod calibration;

use microbit::{display::blocking::Display, hal::Timer};

#[cfg(feature = "v1")]
use microbit::{hal::twi, pac::twi0::frequency::FREQUENCY_A};

#[cfg(feature = "v2")]
use microbit::{hal::twim, pac::twim0::frequency::FREQUENCY_A};

use crate::calibration::calc_calibration;

use independent_logic::{
    led::{direction_to_led, theta_to_direction},
    tilt_compensation::{
        calc_attitude, calc_tilt_calibrated_measurement, heading_from_measurement, Heading,
        NedMeasurement,
    },
};

const DELAY: u32 = 100;

#[entry]
fn main() -> ! {
    rtt_init_print!();
    let board = microbit::Board::take().unwrap();

    #[cfg(feature = "v1")]
    let i2c = { twi::Twi::new(board.TWI0, board.i2c.into(), FREQUENCY_A::K100) };

    #[cfg(feature = "v2")]
    let i2c = { twim::Twim::new(board.TWIM0, board.i2c_internal.into(), FREQUENCY_A::K100) };

    let mut timer = Timer::new(board.TIMER0);
    let mut display = Display::new(board.display_pins);

    let gpiote = Gpiote::new(board.GPIOTE);
    let channel_button_a = gpiote.channel0();
    channel_button_a
        .input_pin(&board.buttons.button_a.degrade())
        .hi_to_lo();
    channel_button_a.reset_events();

    let channel_button_b = gpiote.channel1();
    channel_button_b
        .input_pin(&board.buttons.button_b.degrade())
        .hi_to_lo();
    channel_button_b.reset_events();

    let mut sensor = Lsm303agr::new_with_i2c(i2c);
    sensor.init().unwrap();
    sensor.set_mag_odr(MagOutputDataRate::Hz10).unwrap();
    sensor.set_accel_odr(AccelOutputDataRate::Hz10).unwrap();
    let mut sensor = sensor.into_mag_continuous().ok().unwrap();

    //TODO: re-callibrate with button.
    #[cfg(feature = "calibration")]
    let mut calibration = calc_calibration(&mut sensor, &mut display, &mut timer);
    #[cfg(not(feature = "calibration"))]
    let mut calibration = calibration::Calibration::default();
    rprintln!("Calibration: {:?}", calibration);

    let mut tilt_correction_enabled: bool = true;

    loop {
        if channel_button_b.is_event_triggered() {
            calibration = calc_calibration(&mut sensor, &mut display, &mut timer);
            channel_button_b.reset_events();
            rprintln!("Calibration: {:?}", calibration);
        }
        if channel_button_a.is_event_triggered() {
            //toggles the bool.
            tilt_correction_enabled ^= true;
            channel_button_a.reset_events()
        }

        let heading = calc_heading(&mut sensor, &calibration, &tilt_correction_enabled);
        display.show(
            &mut timer,
            direction_to_led(theta_to_direction(heading)),
            DELAY,
        )
    }
}

/// board has forward in the y direction and right in the -x direction, and down in the -z. (ENU),  algs for tilt compensation
/// need forward in +x and right in +y (this is known as the NED (north, east, down) cordinate
/// system)
/// also converts to f32
pub fn swd_to_ned(measurement: Measurement) -> NedMeasurement {
    NedMeasurement {
        x: -measurement.y as f32,
        y: -measurement.x as f32,
        z: -measurement.z as f32,
    }
}

fn calc_heading(
    sensor: &mut Lsm303agr<I2cInterface<Twim<TWIM0>>, MagContinuous>,
    mag_calibration: &Calibration,
    tilt_correction_enabled: &bool,
) -> Heading {
    while !(sensor.mag_status().unwrap().xyz_new_data
        && sensor.accel_status().unwrap().xyz_new_data)
    {}
    let mag_data = sensor.mag_data().unwrap();
    let mag_data = calibration::calibrated_measurement(mag_data, mag_calibration);
    let acel_data = sensor.accel_data().unwrap();

    let mut ned_mag_data = swd_to_ned(mag_data);
    let ned_acel_data = swd_to_ned(acel_data);

    let attitude = calc_attitude(&ned_acel_data);

    if *tilt_correction_enabled {
        ned_mag_data = calc_tilt_calibrated_measurement(ned_mag_data, &attitude);
    }
    //theta=0 at north, pi/-pi at south, pi/2 at east, and -pi/2 at west
    let heading = heading_from_measurement(ned_mag_data);

    #[cfg(not(feature = "calibration"))]
    rprintln!(
        "pitch: {:<+5.0}, roll: {:<+5.0}, heading: {:<+5.0}",
        attitude.pitch * (180.0 / PI),
        attitude.roll * (180.0 / PI),
        heading.0 * (180.0 / PI),
    );
    #[cfg(not(feature = "calibration"))]
    rprintln!(
        "x: {:<+16}, y: {:<+16}, z: {:<+16}",
        ned_acel_data.x,
        ned_acel_data.y,
        ned_acel_data.z
    );
    heading
}
