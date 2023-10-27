use core::mem::swap;

struct Point {
    x: isize,
    y: isize,
}

impl Point {
    /// converts a point (representing a point on a 4 quadrant grid) into a upoint (representing a
    /// point on a 1 quadrant grid with the origin in the bottom-left corner). Returns none if
    /// the resulting point would have either number negative.
    fn to_upoint(&self, zero_coord: &UPoint) -> Option<UPoint> {
        Some(UPoint {
            x: zero_coord.x.checked_add_signed(self.x)?,
            y: zero_coord.y.checked_add_signed(self.y)?,
        })
    }
}

struct UPoint {
    x: usize,
    y: usize,
}

impl UPoint {
    /// converts a upoint (representing a point on a 1 quadrant grid with the origin in the
    /// bottom-left corner) into a point( representing a point on a 4 quadrant grid
    pub fn to_point(&self, zero_coord: &UPoint) -> Point {
        Point {
            x: zero_coord.x as isize - self.x as isize,
            y: zero_coord.y as isize - self.y as isize,
        }
    }
}

fn draw_line<const X: usize, const Y: usize>(
    mut p0: Point,
    mut p1: Point,
    matrix: &mut [[u8; X]; Y],
) {
    let steep = (p0.x - p1.x).abs() < (p0.y - p1.x).abs();

    if steep {
        swap(&mut p0.x, &mut p0.y);
        swap(&mut p1.x, &mut p1.y);
    }

    if p0.x > p1.x {
        swap(&mut p0, &mut p1)
    }

    let dx = p1.x - p0.x;
    let dy = p1.y - p0.y;
    let derror2 = dy.abs() * 2;
    let mut error2 = 0;
    let mut y = p0.y;

    for x in p0.x..=p1.x {
        if steep {
            matrix[y as usize][x as usize] = 1;
        } else {
            matrix[x as usize][y as usize] = 1;
        }

        error2 += derror2;

        if error2 > dx {
            y += if p1.y > p0.y { 1 } else { -1 };
            error2 -= dx * 2
        }
    }
}

fn heading_to_sector(sectors: u8, heading: f32) -> u8 {
    todo!()
}
