use std::cmp::{max, min};

pub fn median3(a: u16, b: u16, c: u16) -> u16 {
    max(min(a, b), min(c, max(a, b)))
}

pub fn median5(v: [u16; 5]) -> u16 {
    median3(
        v[4],
        max(min(v[0], v[1]), min(v[2], v[3])),
        min(max(v[0], v[1]), max(v[2], v[3])),
    )
}
