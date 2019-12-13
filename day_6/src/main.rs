use std::{
    collections::{HashMap, HashSet},
    fs::File,
    io::{self, BufRead, BufReader},
};

type PlanetMap = HashMap<String, (Option<String>, HashSet<String>)>;

fn hash_sum(map: &PlanetMap, key: &str, level: usize) -> usize {
    map.get(key)
        .map(|keys| {
            keys.1.iter()
                .map(|m_key| hash_sum(map, m_key, level + 1))
                .sum::<usize>() + level
        })
        .unwrap_or(level)
}

fn path_to_com(map: &PlanetMap, level: usize, target: &str, path: &mut HashMap<String, usize>) {
    path.insert(target.to_string(), level);
    match &map[target].0 {
        Some(t) => path_to_com(map, level +1, &t, path),
        None => (),
    }
}

fn main() -> io::Result<()> {
    let reader = BufReader::new(File::open("orbit.txt")?);
    let mut planets = PlanetMap::new();
    for line in reader.lines() {
        let line = line?;
        let (center, planet) = line.split_at(3);
        let (_, planet) = planet.split_at(1);
        planets
            .entry(center.to_string())
            .or_insert_with(|| (None, HashSet::new()))
            .1.insert(planet.to_string());
        planets
            .entry(planet.to_string())
            .or_insert_with(|| (None, HashSet::new()))
            .0 = Some(center.to_string());
    }
    println!("Part1: {}", hash_sum(&planets, "COM", 0));
    let mut san_path = HashMap::new();
    let mut you_path = HashMap::new();
    path_to_com(&planets, 0, "SAN", &mut san_path);
    path_to_com(&planets, 0, "YOU", &mut you_path);
    let minimum = you_path.into_iter()
                          .filter_map(|(key, value)|{
                              san_path.get(&key).map(|n| n + value - 2)
                          })
                          .min();
    println!("part2: {}", minimum.unwrap());
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_name() {
        let input = "A)B
B)C
C)D
D)E
E)F
B)G
G)H
D)I
E)J
J)K
K)L";
        let mut planets = PlanetMap::new();
        for line in input.lines() {
            let center: String = line[..1].to_string();
            let planet: String = line[2..].to_string();
            planets
                .entry(center)
                .or_insert_with(|| (None, HashSet::new()))
                .insert(planet);
        }
        println!("{:#?}", &planets);
        assert_eq!(hash_sum(&planets, "A", 0), 42);
    }
}
