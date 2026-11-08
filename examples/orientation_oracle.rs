use ae2_horizon::ae2_orientation::{self as rules, DIRECTIONS};
use std::io::{self, BufRead};
fn main() {
    for line in io::stdin().lock().lines() {
        let line = line.unwrap();
        let a: Vec<_> = line.split_whitespace().collect();
        let (facing, spin) = if a[0] == "Y" {
            let float = |v| f32::from_bits(u32::from_str_radix(v, 16).unwrap());
            rules::placement(float(a[1]), float(a[2]))
        } else {
            rules::rotate_full(
                DIRECTIONS[a[1].parse::<usize>().unwrap()],
                a[2].parse().unwrap(),
                DIRECTIONS[a[3].parse::<usize>().unwrap()],
            )
            .unwrap()
        };
        println!("{facing} {spin}");
    }
}
