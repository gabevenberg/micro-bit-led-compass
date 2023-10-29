use core::{
    mem::swap,
    ops::{Index, IndexMut},
};

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
/// a nomral matrix.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FourQuadrantMatrix<const X: usize, const Y: usize, T> {
    matrix: [[T; X]; Y],
    pub zero_coord: UPoint,
}

impl<const X: usize, const Y: usize, T> FourQuadrantMatrix<{ X }, { Y }, T>
where
    T: Copy,
    T: Default,
{
    pub fn new(zero_coord: UPoint) -> FourQuadrantMatrix<{ X }, { Y }, T> {
        FourQuadrantMatrix {
            matrix: [[T::default(); X]; Y],
            zero_coord,
        }
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Line(pub Point, pub Point);

//no boxes here!
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ULine(pub UPoint, pub UPoint);

/// Renders a line into a matrix of pixels.
pub fn draw_line<const X: usize, const Y: usize>(
    line: &Line,
    matrix: &mut FourQuadrantMatrix<{ X }, { Y }, u8>,
) {
    let mut line = *line;
    let steep = (line.0.x - line.1.x).abs() < (line.0.y - line.1.x).abs();

    if steep {
        swap(&mut line.0.x, &mut line.0.y);
        swap(&mut line.1.x, &mut line.1.y);
    }

    if line.0.x > line.1.x {
        swap(&mut line.0.x, &mut line.1.x);
        swap(&mut line.0.y, &mut line.1.y)
    }

    let dx = line.1.x - line.0.x;
    let dy = line.1.y - line.0.y;
    let derror2 = dy.abs() * 2;
    let mut error2 = 0;
    let mut y = line.0.y;

    for x in line.0.x..=line.1.x {
        if steep {
            matrix[Point { x: y, y: x }] = 1;
        } else {
            matrix[Point { x, y }] = 1;
        }

        error2 += derror2;

        if error2 > dx {
            y += if line.1.y > line.0.y { 1 } else { -1 };
            error2 -= dx * 2
        }
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
