use core::{
    mem::swap,
    ops::{Index, IndexMut},
};
#[cfg(test)]
use std::dbg;

/// a signed point in 2d space
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Point {
    pub x: isize,
    pub y: isize,
}

impl Point {
    /// converts a point (representing a point on a 4 quadrant grid with positive xy in the
    /// top-right) into a upoint (representing a point on a 1 quadrant grid with the origin in the
    /// top-left corner). Returns none if the resulting point would have either number negative.
    pub fn to_upoint(self, zero_coord: &UPoint) -> Option<UPoint> {
        Some(UPoint {
            x: zero_coord.x.checked_add_signed(self.x)?,
            y: zero_coord.y.checked_add_signed(-self.y)?,
        })
    }
}

/// an unsigned point in 2d space
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct UPoint {
    pub x: usize,
    pub y: usize,
}

impl UPoint {
    /// converts a upoint (representing a point on a 1 quadrant grid with the origin in the
    /// top-left corner) into a point( representing a point on a 4 quadrant grid with positive xy
    /// in the top-right)
    pub fn to_point(self, zero_coord: &UPoint) -> Point {
        Point {
            x: -(zero_coord.x as isize - self.x as isize),
            y: zero_coord.y as isize - self.y as isize,
        }
    }
}

/// A matrix that allows negative co-oordinates. Will panic if referencing out of bounds, just like
/// a normal 2d array.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FourQuadrantMatrix<const X: usize, const Y: usize, T> {
    matrix: [[T; X]; Y],
    max_point: Point,
    min_point: Point,
    zero_coord: UPoint,
}

impl<const X: usize, const Y: usize, T> FourQuadrantMatrix<{ X }, { Y }, T>
where
    T: Copy,
    T: Default,
{
    /// generates a new FourQuadrantMatrix with a given zero point (the point in the underlying 2d
    /// array considered to be (0,0))
    pub fn new(zero_coord: UPoint) -> FourQuadrantMatrix<{ X }, { Y }, T> {
        FourQuadrantMatrix {
            matrix: [[T::default(); X]; Y],
            max_point: UPoint { x: X - 1, y: 0 }.to_point(&zero_coord),
            min_point: UPoint { x: 0, y: Y - 1 }.to_point(&zero_coord),
            zero_coord,
        }
    }

    pub fn zero_coord(&self) -> UPoint {
        self.zero_coord
    }

    pub fn min_point(&self) -> Point {
        self.min_point
    }

    pub fn max_point(&self) -> Point {
        self.max_point
    }

    /// makes sure a point is in bounds and if not, brings it in bounds.
    pub fn bound_point(&self, point: &mut Point) {
        if point.x > self.max_point.x {
            point.x = self.max_point.x
        }

        if point.y > self.max_point.y {
            point.y = self.max_point.y
        }

        if point.x < self.min_point.x {
            point.x = self.min_point.x
        }

        if point.y < self.min_point.y {
            point.y = self.min_point.y
        }
    }

    /// checks if the point is in bounds.
    pub fn is_in_bounds(&self, point: &Point) -> bool {
        point.x <= self.max_point.x
            && point.y <= self.max_point.y
            && point.x >= self.min_point.x
            && point.y >= self.min_point.y
    }
    /// fills the matrix with the Ts default value.
    pub fn reset_matrix(&mut self) {
        self.matrix = [[T::default(); X]; Y];
    }
}

impl<T, const X: usize, const Y: usize> IndexMut<Point> for FourQuadrantMatrix<{ X }, { Y }, T> {
    fn index_mut(&mut self, index: Point) -> &mut Self::Output {
        let upoint = index
            .to_upoint(&self.zero_coord)
            .expect("would result in negative unsigned coordinate!");
        &mut self.matrix[upoint.y][upoint.x]
    }
}

impl<T, const X: usize, const Y: usize> Index<Point> for FourQuadrantMatrix<{ X }, { Y }, T> {
    type Output = T;

    fn index(&self, index: Point) -> &Self::Output {
        let upoint = index
            .to_upoint(&self.zero_coord)
            .expect("would result in negative unsigned coordinate!");
        &self.matrix[upoint.y][upoint.x]
    }
}

impl<T, const X: usize, const Y: usize> From<FourQuadrantMatrix<{ X }, { Y }, T>> for [[T; X]; Y] {
    fn from(value: FourQuadrantMatrix<{ X }, { Y }, T>) -> Self {
        value.matrix
    }
}

/// a line segment in 2d space, described by its two endpoints
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Line(pub Point, pub Point);

/// Renders a line into a matrix of pixels.
/// Will not attempt to mutate outside bounds of the matrix, so it is safe to draw lines that
/// extend past its edges.
pub fn draw_line<const X: usize, const Y: usize>(
    line: &Line,
    matrix: &mut FourQuadrantMatrix<{ X }, { Y }, u8>,
) {
    let mut line = *line;
    #[cfg(test)]
    dbg!(line);

    // Is it steeper than 45°? If so, we transpose the line. This essentially guarantees we are
    // drawing a line less steep than 45°.
    #[cfg(test)]
    dbg!((line.0.x - line.1.x).abs() < (line.0.y - line.1.y).abs());
    let steep = (line.0.x - line.1.x).abs() < (line.0.y - line.1.y).abs();
    if steep {
        swap(&mut line.0.x, &mut line.0.y);
        swap(&mut line.1.x, &mut line.1.y);
    }

    // If our line is running right-to-left, flip the points
    // so we start on the left.
    if line.0.x > line.1.x {
        swap(&mut line.0.x, &mut line.1.x);
        swap(&mut line.0.y, &mut line.1.y)
    }

    #[cfg(test)]
    dbg!((line, steep));
    let dx = line.1.x - line.0.x;
    let dy = line.1.y - line.0.y;
    let derror2 = dy.abs() * 2;
    let mut error2 = 0;
    let mut y = line.0.y;
    let mut draw_point: Point;

    //if the first point is out of bounds, we want to wait for us to get in bounds before arming
    //the early return.
    let mut prev_out_of_bounds = !matrix.is_in_bounds(&line.0);

    // For each X coordinate (which is actually a Y coordinate if the line is steep), we calculate
    // the Y coordinate the same way as before
    for x in line.0.x..=line.1.x {
        #[cfg(test)]
        dbg!((dx, dy, derror2, error2, y, steep, prev_out_of_bounds));
        if steep {
            // Remember the transpose? This is where we undo it, by swapping our y and x
            // coordinates again
            draw_point = Point { x: y, y: x };
        } else {
            draw_point = Point { x, y };
        }

        #[cfg(test)]
        dbg!(draw_point);

        if matrix.is_in_bounds(&draw_point) {
            matrix[draw_point] = 1;
            prev_out_of_bounds = false;
        } else {
            if !prev_out_of_bounds {
                break;
            }
            prev_out_of_bounds = true;
        }

        error2 += derror2;

        #[cfg(test)]
        dbg!((dx, dy, derror2, error2, y, steep, prev_out_of_bounds));

        if error2 > dx {
            y += if line.1.y > line.0.y { 1 } else { -1 };
            error2 -= dx * 2
        }
        #[cfg(test)]
        dbg!((dx, dy, derror2, error2, y, steep, prev_out_of_bounds));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn point_upoint_conv() {
        let zero_coord = UPoint { x: 2, y: 2 };
        let point = Point { x: -1, y: -1 };
        let upoint = point.to_upoint(&zero_coord).unwrap();
        assert_eq!(upoint, UPoint { x: 1, y: 3 });
        assert_eq!(upoint.to_point(&zero_coord), point);

        let point = Point { x: -2, y: 1 };
        let upoint = point.to_upoint(&zero_coord).unwrap();
        assert_eq!(upoint, UPoint { x: 0, y: 1 });
        assert_eq!(upoint.to_point(&zero_coord), point);

        let point = Point { x: 2, y: 2 };
        let upoint = point.to_upoint(&zero_coord).unwrap();
        assert_eq!(upoint, UPoint { x: 4, y: 0 });
        assert_eq!(upoint.to_point(&zero_coord), point);
    }

    #[test]
    fn four_quadrant_matrix() {
        let mut canvas: FourQuadrantMatrix<5, 5, u8> =
            FourQuadrantMatrix::new(UPoint { x: 2, y: 2 });
        canvas[Point { x: 0, y: 0 }] = 1;
        assert_eq!(
            <FourQuadrantMatrix<5, 5, u8> as Into<[[u8; 5]; 5]>>::into(canvas),
            [
                [0, 0, 0, 0, 0],
                [0, 0, 0, 0, 0],
                [0, 0, 1, 0, 0],
                [0, 0, 0, 0, 0],
                [0, 0, 0, 0, 0],
            ]
        );
        canvas[Point { x: -2, y: 1 }] = 1;
        assert_eq!(
            <FourQuadrantMatrix<5, 5, u8> as Into<[[u8; 5]; 5]>>::into(canvas),
            [
                [0, 0, 0, 0, 0],
                [1, 0, 0, 0, 0],
                [0, 0, 1, 0, 0],
                [0, 0, 0, 0, 0],
                [0, 0, 0, 0, 0]
            ]
        );
    }

    #[test]
    fn diagonal_unsigned_line() {
        let mut canvas: FourQuadrantMatrix<5, 5, u8> =
            FourQuadrantMatrix::new(UPoint { x: 0, y: 4 });
        draw_line(
            &Line(Point { x: 0, y: 0 }, Point { x: 4, y: 4 }),
            &mut canvas,
        );
        assert_eq!(
            <FourQuadrantMatrix<5, 5, u8> as Into<[[u8; 5]; 5]>>::into(canvas),
            [
                [0, 0, 0, 0, 1],
                [0, 0, 0, 1, 0],
                [0, 0, 1, 0, 0],
                [0, 1, 0, 0, 0],
                [1, 0, 0, 0, 0],
            ]
        )
    }

    #[test]
    fn diagonal_signed_line() {
        let mut canvas: FourQuadrantMatrix<5, 5, u8> =
            FourQuadrantMatrix::new(UPoint { x: 2, y: 2 });
        draw_line(
            &Line(Point { x: -2, y: -2 }, Point { x: 2, y: 2 }),
            &mut canvas,
        );
        assert_eq!(
            <FourQuadrantMatrix<5, 5, u8> as Into<[[u8; 5]; 5]>>::into(canvas),
            [
                [0, 0, 0, 0, 1],
                [0, 0, 0, 1, 0],
                [0, 0, 1, 0, 0],
                [0, 1, 0, 0, 0],
                [1, 0, 0, 0, 0],
            ]
        )
    }

    #[test]
    fn diagonal_signed_both_oob_line() {
        let mut canvas: FourQuadrantMatrix<5, 5, u8> =
            FourQuadrantMatrix::new(UPoint { x: 2, y: 2 });
        draw_line(
            &Line(Point { x: -10, y: -10 }, Point { x: 10, y: 10 }),
            &mut canvas,
        );
        assert_eq!(
            <FourQuadrantMatrix<5, 5, u8> as Into<[[u8; 5]; 5]>>::into(canvas),
            [
                [0, 0, 0, 0, 1],
                [0, 0, 0, 1, 0],
                [0, 0, 1, 0, 0],
                [0, 1, 0, 0, 0],
                [1, 0, 0, 0, 0],
            ]
        );
    }

    #[test]
    fn diagonal_signed_first_oob_line() {
        let mut canvas: FourQuadrantMatrix<5, 5, u8> =
            FourQuadrantMatrix::new(UPoint { x: 2, y: 2 });
        draw_line(
            &Line(Point { x: -10, y: -10 }, Point { x: 2, y: 2 }),
            &mut canvas,
        );
        assert_eq!(
            <FourQuadrantMatrix<5, 5, u8> as Into<[[u8; 5]; 5]>>::into(canvas),
            [
                [0, 0, 0, 0, 1],
                [0, 0, 0, 1, 0],
                [0, 0, 1, 0, 0],
                [0, 1, 0, 0, 0],
                [1, 0, 0, 0, 0],
            ]
        );
    }

    #[test]
    fn diagonal_signed_second_oob_line() {
        let mut canvas: FourQuadrantMatrix<5, 5, u8> =
            FourQuadrantMatrix::new(UPoint { x: 2, y: 2 });
        draw_line(
            &Line(Point { x: -2, y: -2 }, Point { x: 10, y: 10 }),
            &mut canvas,
        );
        assert_eq!(
            <FourQuadrantMatrix<5, 5, u8> as Into<[[u8; 5]; 5]>>::into(canvas),
            [
                [0, 0, 0, 0, 1],
                [0, 0, 0, 1, 0],
                [0, 0, 1, 0, 0],
                [0, 1, 0, 0, 0],
                [1, 0, 0, 0, 0],
            ]
        );
    }

    #[test]
    fn vertical_signed_both_oob_line() {
        let mut canvas: FourQuadrantMatrix<5, 5, u8> =
            FourQuadrantMatrix::new(UPoint { x: 2, y: 2 });
        draw_line(
            &Line(Point { x: 0, y: -10 }, Point { x: 0, y: 10 }),
            &mut canvas,
        );
        assert_eq!(
            <FourQuadrantMatrix<5, 5, u8> as Into<[[u8; 5]; 5]>>::into(canvas),
            [
                [0, 0, 1, 0, 0],
                [0, 0, 1, 0, 0],
                [0, 0, 1, 0, 0],
                [0, 0, 1, 0, 0],
                [0, 0, 1, 0, 0],
            ]
        );
    }

    #[test]
    fn vertical_signed_first_oob_line() {
        let mut canvas: FourQuadrantMatrix<5, 5, u8> =
            FourQuadrantMatrix::new(UPoint { x: 2, y: 2 });
        draw_line(
            &Line(Point { x: 0, y: -10 }, Point { x: 0, y: 0 }),
            &mut canvas,
        );
        assert_eq!(
            <FourQuadrantMatrix<5, 5, u8> as Into<[[u8; 5]; 5]>>::into(canvas),
            [
                [0, 0, 0, 0, 0],
                [0, 0, 0, 0, 0],
                [0, 0, 1, 0, 0],
                [0, 0, 1, 0, 0],
                [0, 0, 1, 0, 0],
            ]
        );
    }

    #[test]
    fn vertical_signed_second_oob_line() {
        let mut canvas: FourQuadrantMatrix<5, 5, u8> =
            FourQuadrantMatrix::new(UPoint { x: 2, y: 2 });
        draw_line(
            &Line(Point { x: 0, y: 0 }, Point { x: 0, y: 10 }),
            &mut canvas,
        );
        assert_eq!(
            <FourQuadrantMatrix<5, 5, u8> as Into<[[u8; 5]; 5]>>::into(canvas),
            [
                [0, 0, 1, 0, 0],
                [0, 0, 1, 0, 0],
                [0, 0, 1, 0, 0],
                [0, 0, 0, 0, 0],
                [0, 0, 0, 0, 0],
            ]
        );
    }

    #[test]
    fn cross_signed_line() {
        let mut canvas: FourQuadrantMatrix<5, 5, u8> =
            FourQuadrantMatrix::new(UPoint { x: 2, y: 2 });
        draw_line(
            &Line(Point { x: 0, y: -2 }, Point { x: 0, y: 2 }),
            &mut canvas,
        );
        draw_line(
            &Line(Point { x: -2, y: 0 }, Point { x: 2, y: 0 }),
            &mut canvas,
        );
        assert_eq!(
            <FourQuadrantMatrix<5, 5, u8> as Into<[[u8; 5]; 5]>>::into(canvas),
            [
                [0, 0, 1, 0, 0],
                [0, 0, 1, 0, 0],
                [1, 1, 1, 1, 1],
                [0, 0, 1, 0, 0],
                [0, 0, 1, 0, 0],
            ]
        )
    }
}
