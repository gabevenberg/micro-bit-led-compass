use core::f32::consts::PI;
use crate::tilt_compensation::Heading;

#[derive(Debug)]
pub enum Direction {
    North,
    NorthEast,
    East,
    SouthEast,
    South,
    SouthWest,
    West,
    NorthWest,
}

const NORTH: [[u8; 5]; 5] = [
    [0, 0, 1, 0, 0],
    [0, 1, 1, 1, 0],
    [1, 0, 1, 0, 1],
    [0, 0, 1, 0, 0],
    [0, 0, 1, 0, 0],
];

const NORTH_EAST: [[u8; 5]; 5] = [
    [1, 1, 1, 0, 0],
    [1, 1, 0, 0, 0],
    [1, 0, 1, 0, 0],
    [0, 0, 0, 1, 0],
    [0, 0, 0, 0, 1],
];

const EAST: [[u8; 5]; 5] = [
    [0, 0, 1, 0, 0],
    [0, 1, 0, 0, 0],
    [1, 1, 1, 1, 1],
    [0, 1, 0, 0, 0],
    [0, 0, 1, 0, 0],
];

const SOUTH_EAST: [[u8; 5]; 5] = [
    [0, 0, 0, 0, 1],
    [0, 0, 0, 1, 0],
    [1, 0, 1, 0, 0],
    [1, 1, 0, 0, 0],
    [1, 1, 1, 0, 0],
];

const SOUTH: [[u8; 5]; 5] = [
    [0, 0, 1, 0, 0],
    [0, 0, 1, 0, 0],
    [1, 0, 1, 0, 1],
    [0, 1, 1, 1, 0],
    [0, 0, 1, 0, 0],
];

const SOUTH_WEST: [[u8; 5]; 5] = [
    [1, 0, 0, 0, 0],
    [0, 1, 0, 0, 0],
    [0, 0, 1, 0, 1],
    [0, 0, 0, 1, 1],
    [0, 0, 1, 1, 1],
];

const WEST: [[u8; 5]; 5] = [
    [0, 0, 1, 0, 0],
    [0, 0, 0, 1, 0],
    [1, 1, 1, 1, 1],
    [0, 0, 0, 1, 0],
    [0, 0, 1, 0, 0],
];

const NORTH_WEST: [[u8; 5]; 5] = [
    [0, 0, 1, 1, 1],
    [0, 0, 0, 1, 1],
    [0, 0, 1, 0, 1],
    [0, 1, 0, 0, 0],
    [1, 0, 0, 0, 0],
];

pub fn direction_to_led(direction: Direction) -> [[u8; 5]; 5] {
    match direction {
        Direction::North => NORTH,
        Direction::NorthEast => NORTH_EAST,
        Direction::East => EAST,
        Direction::SouthEast => SOUTH_EAST,
        Direction::South => SOUTH,
        Direction::SouthWest => SOUTH_WEST,
        Direction::West => WEST,
        Direction::NorthWest => NORTH_WEST,
    }
}

pub fn theta_to_direction(heading: Heading) -> Direction {
    if heading.0 < (-7. * PI / 8.) {
        Direction::North
    } else if heading.0 < (-5. * PI / 8.) {
        Direction::NorthWest
    } else if heading.0 < (-3. * PI / 8.) {
        Direction::West
    } else if heading.0 < (-PI / 8.) {
        Direction::SouthWest
    } else if heading.0 < (PI / 8.) {
        Direction::South
    } else if heading.0 < (3. * PI / 8.) {
        Direction::SouthEast
    } else if heading.0 < (5. * PI / 8.) {
        Direction::East
    } else if heading.0 < (7. * PI / 8.) {
        Direction::NorthEast
    } else {
        Direction::North
    }
}
