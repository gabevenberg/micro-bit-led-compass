#![allow(unused)]
use core::f32::consts::PI;

use crate::line_drawing::{draw_line, FourQuadrantMatrix, Line, Point, UPoint};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Sector {
    total_sectors: usize,
    sector: usize,
}

//heading starts at north, with the positive direction being clockwise.
//Heading ranges from -pi to pi.
//
//sectors have 0 at north an proceed clockwise, always being positive.
fn heading_to_sector(sectors: usize, heading: f32) -> Sector {
    let half_sector = PI / sectors as f32;
    let sector_size = 2.0 * half_sector;
    Sector {
        total_sectors: sectors,
        sector: (modulo(heading + half_sector, 2.0 * PI) / (sector_size)) as usize,
    }
}

fn modulo(a: f32, b: f32) -> f32 {
    ((a % b) + b) % b
}

fn heading_to_line(heading: f32, square_size: usize) -> Line {
    todo!()
}

pub fn draw_heading<const X: usize, const Y: usize>(
    heading: f32,
) -> FourQuadrantMatrix<{ X }, { Y }, u8> {
    let mut ret = FourQuadrantMatrix::new(UPoint { x: X / 2, y: Y / 2 });
    draw_line::<X, Y>(&heading_to_line(heading, X.min(Y)), &mut ret);
    ret
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sectors() {
        assert_eq!(heading_to_sector(4,0.0).sector, 0);
        assert_eq!(heading_to_sector(4,PI/2.0).sector, 1);
        assert_eq!(heading_to_sector(4,-PI/2.0).sector, 3);
        assert_eq!(heading_to_sector(4,PI).sector, 2);
        assert_eq!(heading_to_sector(4,-PI).sector, 2);
    }
}
