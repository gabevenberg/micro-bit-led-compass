#![deny(unsafe_code)]
#![no_main]
#![no_std]

use core::f32::consts::PI;
use defmt::{debug, info};
use embassy_executor::Spawner;
use embassy_sync::{blocking_mutex::raw::CriticalSectionRawMutex, signal::Signal};
use embassy_time::{Duration, Ticker};
use microbit_bsp::{
    LedMatrix, Microbit,
    display::{Bitmap, Brightness, Frame},
    embassy_nrf::{
        bind_interrupts,
        peripherals::TWISPI0,
        twim::{InterruptHandler, Twim},
    },
    lsm303agr::{self, Lsm303agr, interface::I2cInterface, mode::MagContinuous},
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

static HEADING: Signal<CriticalSectionRawMutex, Heading> = Signal::new();

#[embassy_executor::main]
async fn main(s: Spawner) {
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
    s.must_spawn(get_data(sensor));
    s.must_spawn(display_data(display));
}

#[embassy_executor::task]
async fn display_data(mut display: LedMatrix) {
    let mut display_matrix: FourQuadrantMatrix<5, 5, bool> =
        FourQuadrantMatrix::new(UPoint { x: 2, y: 2 });
    loop {
        let heading = HEADING.wait().await;
        info!("Heading: {}", heading.0 * (180.0 / PI));
        draw_constant_heading(heading, &mut display_matrix);
        display
            .display(to_frame(&display_matrix), Duration::from_hz(25))
            .await;
    }
}

#[embassy_executor::task]
async fn get_data(mut sensor: Lsm303agr<I2cInterface<Twim<'static, TWISPI0>>, MagContinuous>) {
    let mut ticker = Ticker::every(Duration::from_hz(25));
    loop {
        let (x, y, z) = sensor
            .magnetic_field()
            .await
            .expect("didnt get mag data")
            .xyz_nt();
        let mag_measurement = to_ned(x, y, z);
        let (x, y, z) = sensor
            .acceleration()
            .await
            .expect("didnt get accel data")
            .xyz_mg();
        let accel_measurement = to_ned(x, y, z);
        debug!("Mag: {}, Accel: {}", mag_measurement, accel_measurement);
        let attitude = calc_attitude(&accel_measurement);
        let mag_measurement = calc_tilt_calibrated_measurement(mag_measurement, &attitude);
        HEADING.signal(heading_from_measurement(&mag_measurement));
        ticker.next().await;
    }
}

// TODO: make the line drawing lib produce a slice of bitmaps directly.
fn to_frame(matrix: &FourQuadrantMatrix<5, 5, bool>) -> Frame<5, 5> {
    Frame::new(
        core::convert::Into::<&[[bool; 5]; 5]>::into(matrix).map(|bools| {
            let mut bit: u8 = 0;
            for (i, bool) in bools.into_iter().enumerate() {
                bit |= (bool as u8) << i;
            }
            Bitmap::new(bit, 5)
        }),
    )
}

fn to_ned(x: i32, y: i32, z: i32) -> NedMeasurement {
    NedMeasurement {
        x: -y as f32,
        y: x as f32,
        z: -z as f32,
    }
}
