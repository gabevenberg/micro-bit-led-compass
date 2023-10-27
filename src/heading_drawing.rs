use crate::line_drawing::{draw_line, FourQuadrantMatrix, Line, UPoint};

#[derive(Debug, Clone, Copy)]
struct Sector {
    total_sectors: u8,
    sectors: u8,
}

//heading starts at north, with the positive direction being clockwise.
//Heading ranges from -pi to pi.
//
//sectors have 0 at north an proceed clockwise, always being positive.
fn heading_to_sector(sectors: u8, heading: f32) -> Sector {
    todo!()
}

fn sector_to_line(sector: Sector, radius: usize) -> Line {
    todo!()
}

pub fn draw_heading<const X: usize, const Y: usize>(
    heading: f32,
    sectors: u8,
) -> FourQuadrantMatrix<{ X }, { Y }, u8> {
    let mut ret = FourQuadrantMatrix::new(UPoint { x: X / 2, y: Y / 2 });
    draw_line::<X, Y>(
        &sector_to_line(heading_to_sector(sectors, heading), X.max(Y)),
        &mut ret,
    );
    ret
}
