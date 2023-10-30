use libm::{cosf, roundf, sinf};

use crate::line_drawing::{draw_line, FourQuadrantMatrix, Line, Point};

fn heading_to_line(heading: f32, square_size: usize) -> Line {
    Line(
        Point { x: 0, y: 0 },
        Point {
            x: roundf((square_size as f32) * sinf(heading)) as isize,
            y: roundf((square_size as f32) * cosf(heading)) as isize,
        },
    )
}

pub fn draw_heading<const X: usize, const Y: usize>(
    heading: f32,
    matrix: &mut FourQuadrantMatrix<{ X }, { Y }, u8>,
) {
    draw_line::<X, Y>(&heading_to_line(heading, X.min(Y)), matrix);
}
