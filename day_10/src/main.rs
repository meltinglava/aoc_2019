use itertools::Itertools;

use std::{
    cmp::Ordering,
    collections::{HashMap, HashSet},
    convert::{TryFrom, TryInto},
    f64::consts::FRAC_PI_4,
    fs::read_to_string,
    io,
};

#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy)]
struct Point {
    x: usize,
    y: usize,
}

impl Point {
    fn new<T: TryInto<usize>>(x: T, y: T) -> Self {
        let x = match x.try_into() {
            Ok(n) => n,
            Err(_) => panic!("Bad convertion"),
        };
        let y = match y.try_into() {
            Ok(n) => n,
            Err(_) => panic!("Bad convertion"),
        };
        Self { x, y }
    }

    fn with_distance(&self, d: Direction, n: isize, max_x: usize, max_y: usize) -> Option<Self> {
        let x = isize::try_from(self.x).unwrap() + d.d_x * n;
        let y = isize::try_from(self.y).unwrap() + d.d_y * n;
        if (0isize..isize::try_from(max_x).unwrap()).contains(&x)
            && (0isize..isize::try_from(max_y).unwrap()).contains(&y)
        {
            Some(Point::new(x, y))
        } else {
            None
        }
    }
}

#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy)]
struct Direction {
    d_x: isize,
    d_y: isize,
}

fn gcd(a: isize, b: isize) -> isize {
    let a = a.abs();
    let b = b.abs();
    match b {
        0 => a,
        _ => gcd(b, a % b),
    }
}

fn shorten(d_x: &mut isize, d_y: &mut isize) {
    let x = d_x.abs();
    let y = d_y.abs();
    let divisor = gcd(x, y);
    *d_x /= divisor;
    *d_y /= divisor;
}

impl Direction {
    fn new(you: Point, other: Point) -> Self {
        let mut d_x = isize::try_from(other.x).unwrap() - isize::try_from(you.x).unwrap();
        let mut d_y = isize::try_from(other.y).unwrap() - isize::try_from(you.y).unwrap();
        shorten(&mut d_x, &mut d_y);
        Self { d_x, d_y }
    }

    fn tan(&self) -> f64 {
        let x = self.d_x as f64;
        let y = self.d_y as f64;
        ((match y.atan2(x) / FRAC_PI_4 {
            x if x.is_sign_negative() => 8.0 + x,
            x => x,
        }) + 2.0)
            % 8.0
    }
}

impl PartialOrd for Direction {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.tan().partial_cmp(&other.tan())
    }
}

#[derive(Debug)]
struct Lazer {
    heading: usize,
    home: Point,
    astroids: HashSet<Point>,
    astroids_directions: Vec<Direction>,
    shot: HashSet<Point>,
    d_x: usize,
    d_y: usize,
}

impl Lazer {
    fn new(
        home: Point,
        astroid_data: HashMap<Point, HashSet<Direction>>,
        d_x: usize,
        d_y: usize,
    ) -> Self {
        let astroids = astroid_data
            .keys()
            .filter(|&k| *k != home)
            .cloned()
            .collect();
        let astroids_directions = astroid_data
            .get(&home)
            .unwrap()
            .iter()
            .sorted_by(|s, o| s.partial_cmp(o).unwrap())
            .cloned()
            .collect();
        dbg!(home);
        Self {
            heading: 0,
            home,
            astroids,
            astroids_directions,
            shot: HashSet::new(),
            d_x,
            d_y,
        }
    }
}

impl Iterator for Lazer {
    type Item = Point;

    fn next(&mut self) -> Option<Self::Item> {
        let unshot = self
            .astroids
            .symmetric_difference(&self.shot)
            .collect::<HashSet<_>>();
        match unshot.len() {
            0 => None,
            _ => loop {
                let direction = self.astroids_directions.get(self.heading).unwrap();
                let ans = (1..)
                    .map(|n| self.home.with_distance(*direction, n, self.d_x, self.d_y))
                    .take_while(|point| point.is_some())
                    .map(|c| c.unwrap())
                    .find(|p| unshot.contains(&p));
                self.heading = if self.heading + 1 == self.astroids_directions.len() {
                    0
                } else {
                    self.heading + 1
                };
                if let Some(p) = ans {
                    eprintln!(
                        "{:?}, {:.3}, {:?}, {}",
                        Direction::new(self.home, p),
                        Direction::new(self.home, p).tan(),
                        p,
                        self.shot.len() + 1
                    );
                    self.shot.insert(p);
                    return Some(p);
                }
            },
        }
    }
}

fn astroid_field(map: &str) -> (HashSet<Point>, usize, usize) {
    let mut x = 0;
    let mut y = 0;
    let astroids: HashSet<_> = map
        .trim()
        .split('\n')
        .enumerate()
        .inspect(|(n, line)| {
            x = line.chars().count();
            y = *n;
        })
        .map(|(y, n)| {
            n.chars().enumerate().filter_map(move |(x, c)| match c {
                '#' => Some(Point::new(x, y)),
                '.' => None,
                s => unreachable!("Astroid fields only contains: \"#.\", got: {}", s),
            })
        })
        .flatten()
        .collect();
    (astroids, x, y + 1)
}

fn find_sighted(map: HashSet<Point>) -> HashMap<Point, HashSet<Direction>> {
    map.iter()
        .map(|n| {
            (
                *n,
                map.iter()
                    .filter(|&c| c != n)
                    .map(|c| Direction::new(*n, *c))
                    .collect(),
            )
        })
        .collect()
}

fn find_the_most_seen(map: HashSet<Point>) -> (Point, usize) {
    find_sighted(map)
        .iter()
        .map(|(p, d)| (*p, d.len()))
        .max_by_key(|(_, key)| *key)
        .unwrap()
}

fn find_200th(point: Point, map: HashSet<Point>, x: usize, y: usize) -> usize {
    let mut lazer = Lazer::new(point, find_sighted(map), x, y);
    let point = dbg!(lazer.nth(199).unwrap());
    point.x * 100 + point.y
}

fn main() -> io::Result<()> {
    let map = astroid_field(&read_to_string("map.txt")?);
    let (point, seen) = find_the_most_seen(map.clone().0);
    println!("part1: {}", seen);
    let last_shot = find_200th(point, map.0, map.1, map.2);
    println!("part2: {}", last_shot);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_read() {
        let t1 = ".#..#
.....
#####
....#
...##";
        let t2 = "......#.#.
#..#.#....
..#######.
.#.#.###..
.#..#.....
..#....#.#
#..#....#.
.##.#..###
##...#..#.
.#....####";
        let t3 = "#.#...#.#.
.###....#.
.#....#...
##.#.#.#.#
....#.#.#.
.##..###.#
..#...##..
..##....##
......#...
.####.###.";
        let t4 = ".#..#..###
####.###.#
....###.#.
..###.##.#
##.##.#.#.
....###..#
..#.#..#.#
#..#.#.###
.##...##.#
.....#.#..";
        let t5 = ".#..##.###...#######
##.############..##.
.#.######.########.#
.###.#######.####.#.
#####.##.#.##.###.##
..#####..#.#########
####################
#.####....###.#.#.##
##.#################
#####.##.###..####..
..######..##.#######
####.##.####...##..#
.#####..#.######.###
##...#.##########...
#.##########.#######
.####.#.###.###.#.##
....##.##.###..#####
.#.#.###########.###
#.#.#.#####.####.###
###.##.####.##.#..##";
        let str_fields = &[t1, t2, t3, t4, t5];
        let set_fields = str_fields
            .iter()
            .map(|n| astroid_field(*n))
            .collect::<Vec<_>>();
        let mut n = 0;
        str_fields
            .iter()
            .zip(set_fields.iter())
            .for_each(|(st, set)| {
                assert_eq!(st.chars().filter(|&c| c == '#').count(), set.0.len());
                n += 1
            });
        let fields_ans = &[8usize, 33, 35, 41, 210];
        set_fields
            .iter()
            .zip(fields_ans.iter())
            .for_each(|(set, ans)| {
                assert_eq!(find_the_most_seen(set.0.clone()).1, *ans);
                n += 1
            });
        assert_eq!(n, 10, "Not all tests was run")
    }

    #[test]
    fn test_tan() {
        let tan = vec![
            Direction { d_x: 0, d_y: -1 },
            Direction { d_x: 1, d_y: -1 },
            Direction { d_x: 1, d_y: 0 },
            Direction { d_x: 1, d_y: 1 },
            Direction { d_x: 0, d_y: 1 },
            Direction { d_x: -1, d_y: 1 },
            Direction { d_x: -1, d_y: 0 },
            Direction { d_x: -1, d_y: -1 },
        ];
        assert_eq!(
            tan,
            tan.iter()
                .cloned()
                .sorted_by(|this, other| this.partial_cmp(other).unwrap())
                .collect::<Vec<_>>()
        )
    }

    #[test]
    fn test_200th() {
        let t5 = ".#..##.###...#######
##.############..##.
.#.######.########.#
.###.#######.####.#.
#####.##.#.##.###.##
..#####..#.#########
####################
#.####....###.#.#.##
##.#################
#####.##.###..####..
..######..##.#######
####.##.####...##..#
.#####..#.######.###
##...#.##########...
#.##########.#######
.####.#.###.###.#.##
....##.##.###..#####
.#.#.###########.###
#.#.#.#####.####.###
###.##.####.##.#..##";
        let map = astroid_field(t5);
        dbg!(&map);
        let (point, _) = dbg!(find_the_most_seen(map.clone().0));
        let last_shot = find_200th(point, map.0, map.1, map.2);
        assert_eq!(last_shot, 802)
    }
}
