use libm::{cosf, roundf, sinf};

use crate::line_drawing::{draw_line, FourQuadrantMatrix, Line, Point};
use crate::tilt_compensation::Heading;

fn heading_to_line(heading: Heading, square_size: usize) -> Line {
    Line(
        Point { x: 0, y: 0 },
        Point {
            x: roundf((square_size as f32) * sinf(heading.0)) as isize,
            y: roundf((square_size as f32) * cosf(heading.0)) as isize,
        },
    )
}

// given the compass heading that the board '0' is facing,
// draws a line always pointing towards heading 0
pub fn draw_constant_heading<const X: usize, const Y: usize>(
    heading: Heading,
    matrix: &mut FourQuadrantMatrix<{ X }, { Y }, bool>,
) {
    draw_line::<X, Y>(&heading_to_line(heading, X.min(Y)), matrix);
}
