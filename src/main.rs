#![deny(unsafe_code)]
#![no_main]
#![no_std]

use core::f32::consts::PI;

use cortex_m_rt::entry;
use panic_rtt_target as _;
use rtt_target::{rprintln, rtt_init_print};

mod calibration;
mod led;
mod tilt_compensation;

use microbit::{display::blocking::Display, hal::Timer};

#[cfg(feature = "v1")]
use microbit::{hal::twi, pac::twi0::frequency::FREQUENCY_A};

#[cfg(feature = "v2")]
use microbit::{hal::twim, pac::twim0::frequency::FREQUENCY_A};

use lsm303agr::{AccelOutputDataRate, Lsm303agr, MagOutputDataRate};

#[cfg(feature = "calibration")]
use crate::calibration::calc_calibration;

use crate::led::{direction_to_led, theta_to_direction};
use crate::tilt_compensation::{calc_attitude, calc_tilt_calibrated_measurement, swd_to_ned};

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

    let mut sensor = Lsm303agr::new_with_i2c(i2c);
    sensor.init().unwrap();
    sensor.set_mag_odr(MagOutputDataRate::Hz10).unwrap();
    sensor.set_accel_odr(AccelOutputDataRate::Hz10).unwrap();
    let mut sensor = sensor.into_mag_continuous().ok().unwrap();

    //TODO: re-callibrate with button.
    #[cfg(feature = "calibration")]
    let calibration = calc_calibration(&mut sensor, &mut display, &mut timer);
    #[cfg(not(feature = "calibration"))]
    let calibration = calibration::Calibration::default();
    rprintln!("Calibration: {:?}", calibration);

    loop {
        while !(sensor.mag_status().unwrap().xyz_new_data
            && sensor.accel_status().unwrap().xyz_new_data)
        {}
        let mag_data = sensor.mag_data().unwrap();
        let mag_data = calibration::calibrated_measurement(mag_data, &calibration);
        let acel_data = sensor.accel_data().unwrap();

        let ned_mag_data = swd_to_ned(mag_data);
        let ned_acel_data = swd_to_ned(acel_data);

        let attitude = calc_attitude(&ned_acel_data);

        //theta=0 at north, pi/-pi at south, pi/2 at east, and -pi/2 at west
        let heading = calc_tilt_calibrated_measurement(ned_mag_data, &attitude);

        #[cfg(not(feature = "calibration"))]
        rprintln!(
            "pitch: {:<+5.0}, roll: {:<+5.0}, heading: {:<+5.0}",
            attitude.pitch * (180.0 / PI),
            attitude.roll * (180.0 / PI),
            heading.0 * (180.0 / PI),
        );
        rprintln!("x: {:<+16}, y: {:<+16}, z: {:<+16}", ned_acel_data.x, ned_acel_data.y, ned_acel_data.z);

        display.show(
            &mut timer,
            direction_to_led(theta_to_direction(heading)),
            DELAY,
        )
    }
}
