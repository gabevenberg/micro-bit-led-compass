#![deny(unsafe_code)]
#![no_main]
#![no_std]

use core::f32::consts::PI;
use defmt::{debug, info};
use embassy_executor::Spawner;
use embassy_time::Timer;
use microbit_bsp::{
    Microbit,
    display::{Brightness, Frame},
    embassy_nrf::{bind_interrupts, peripherals::TWISPI0, twim::InterruptHandler},
    lsm303agr,
    motion::new_lsm303agr,
};
use {defmt_rtt as _, panic_probe as _};

use independent_logic::{
    heading_drawing::draw_constant_heading,
    line_drawing::{FourQuadrantMatrix, UPoint},
    tilt_compensation::{
        Heading, NedMeasurement, calc_attitude, calc_tilt_calibrated_measurement,
        heading_from_measurement,
    },
};

#[embassy_executor::main]
async fn main(_s: Spawner) {
    let board = Microbit::default();
    defmt::info!("Application started!");

    let mut display = board.display;
    display.set_brightness(Brightness::MAX);

    // Bind interrupt to the TWI/SPI peripheral.
    bind_interrupts!(
        struct InterruptRequests {
            TWISPI0 => InterruptHandler<TWISPI0>;
        }
    );

    let irqs = InterruptRequests {};
    let mut sensor = new_lsm303agr(board.twispi0, irqs, board.i2c_int_sda, board.i2c_int_scl);
    sensor.init().await.unwrap();
    sensor.enable_mag_offset_cancellation().await.unwrap();
    sensor
        .set_mag_mode_and_odr(
            &mut embassy_time::Delay,
            lsm303agr::MagMode::HighResolution,
            lsm303agr::MagOutputDataRate::Hz50,
        )
        .await
        .unwrap();
    let Ok(mut sensor) = sensor.into_mag_continuous().await else {
        panic!("Failed to set sensor to continuous mode");
    };
    sensor
        .set_accel_mode_and_odr(
            &mut embassy_time::Delay,
            lsm303agr::AccelMode::Normal,
            lsm303agr::AccelOutputDataRate::Hz50,
        )
        .await
        .unwrap();

    Timer::after_secs(2).await;

    loop {
        let (x, y, z) = sensor.magnetic_field().await.unwrap().xyz_nt();
        let mag_measurement = to_ned(x, y, z);
        let (x, y, z) = sensor.acceleration().await.unwrap().xyz_mg();
        let accel_measurement = to_ned(x, y, z);
        debug!("Mag: {}, Accel: {}", mag_measurement, accel_measurement);
        Timer::after_millis(250).await;
        let attitude = calc_attitude(&accel_measurement);
        let mag_measurement = calc_tilt_calibrated_measurement(mag_measurement, &attitude);
        let heading = heading_from_measurement(&mag_measurement);
        debug!("Attitude: {}, Heading: {}", attitude, heading.0*(180.0/PI));
    }
}

pub fn to_ned(x: i32, y: i32, z: i32) -> NedMeasurement {
    NedMeasurement {
        x: -y as f32,
        y: x as f32,
        z: -z as f32,
    }
}
