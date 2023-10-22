use core::{iter::Iterator, mem::swap};

struct Point {
    x: isize,
    y: isize,
}



fn draw_line(mut p0: Point, mut p1: Point, matrix: &mut [[u8; 5]; 5]) {
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
