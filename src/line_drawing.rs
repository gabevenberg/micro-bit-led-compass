use core::{mem::swap, ops::Index, ops::IndexMut};

#[derive(Debug, Clone, Copy)]
pub struct Point {
    pub x: isize,
    pub y: isize,
}

impl Point {
    /// converts a point (representing a point on a 4 quadrant grid) into a upoint (representing a
    /// point on a 1 quadrant grid with the origin in the bottom-left corner). Returns none if
    /// the resulting point would have either number negative.
    pub fn to_upoint(self, zero_coord: &UPoint) -> Option<UPoint> {
        Some(UPoint {
            x: zero_coord.x.checked_add_signed(self.x)?,
            y: zero_coord.y.checked_add_signed(self.y)?,
        })
    }
}

#[derive(Debug, Clone, Copy)]
pub struct UPoint {
    pub x: usize,
    pub y: usize,
}

impl UPoint {
    /// converts a upoint (representing a point on a 1 quadrant grid with the origin in the
    /// bottom-left corner) into a point( representing a point on a 4 quadrant grid)
    pub fn to_point(self, zero_coord: &UPoint) -> Point {
        Point {
            x: zero_coord.x as isize - self.x as isize,
            y: zero_coord.y as isize - self.y as isize,
        }
    }
}

/// A matrix that allows negative co-oordinates. Will panic if referencing out of bounds, just like
/// a nomral matrix.
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
        &mut self.matrix[upoint.x][upoint.y]
    }
}

impl<T, const X: usize, const Y: usize> Index<Point> for FourQuadrantMatrix<{ X }, { Y }, T> {
    type Output = T;

    fn index(&self, index: Point) -> &Self::Output {
        let upoint = index
            .to_upoint(&self.zero_coord)
            .expect("would result in negative unsigned coordinate!");
        &self.matrix[upoint.x][upoint.y]
    }
}

impl<T, const X: usize, const Y: usize> From<FourQuadrantMatrix<{ X }, { Y }, T>> for [[T; X]; Y] {
    fn from(value: FourQuadrantMatrix<{ X }, { Y }, T>) -> Self {
        value.matrix
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Line(pub Point, pub Point);

//no boxes here!
#[derive(Debug, Clone, Copy)]
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
