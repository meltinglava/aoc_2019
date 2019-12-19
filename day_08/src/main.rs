use itertools::Itertools;
use bytecount::count;

use std::{fs::read_to_string, io};

fn get_square(layers: &[Vec<u8>], x:usize, y: usize) -> char {
    let mut ans = ' ';
    for i in layers {
        ans = match i[x + 25 * y] {
            0 => '\u{25A0}',
            1 => '\u{25A1}',
            2 => continue,
            _ => unreachable!(),
        };
        break;
    }
    ans
}

fn main() -> io::Result<()> {
    let all_digits = read_to_string("picture.txt")?;
    let layers = all_digits
        .trim()
        .chars()
        .map(|c| c.to_digit(10).unwrap() as u8)
        .chunks(25 * 6)
        .into_iter()
        .map(|c| c.collect::<Vec<_>>())
        .collect::<Vec<_>>();
    let ans = layers
        .iter()
        .min_by_key(|c| count(c, 0))
        .map(|n| count(n, 1) * count(n, 2))
        .unwrap();
    println!("{}", ans);
    for y in 0..6 {
        for x in 0..25 {
            print!("{}", get_square(&layers, x, y))
        }
        println!();
    }
    Ok(())
}
