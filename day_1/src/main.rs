use std::{
    fs::File,
    io::{self, prelude::*, BufReader},
};

fn module_fuel(mass: usize) -> usize {
    (mass / 3).saturating_sub(2)
}

fn needed_fuel<T: IntoIterator<Item = usize>>(iter: T) -> usize {
    iter.into_iter()
        .map(|mass| {
            let mut added_fuel = module_fuel(mass);
            let mut new_fuel = added_fuel;
            while new_fuel != 0 {
                new_fuel = module_fuel(new_fuel);
                added_fuel += new_fuel;
            }
            added_fuel
        })
        .sum()
}

fn main() -> io::Result<()> {
    let file = File::open("masses.txt")?;
    let reader = BufReader::new(file);
    let fuel_items = reader
        .lines()
        .map(|line| line.unwrap().parse::<usize>().unwrap());
    let fuel_sum = needed_fuel(Box::new(fuel_items));
    println!("Total fuel for the moduals is: {}", fuel_sum);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_single_module_fuel() {
        assert_eq!(module_fuel(12), 2);
        assert_eq!(module_fuel(14), 2);
        assert_eq!(module_fuel(1969), 654);
        assert_eq!(module_fuel(100756), 33583);
    }

    #[test]
    fn test_total_module_fuel() {
        assert_eq!(needed_fuel(vec![1969]), 966);
        assert_eq!(needed_fuel(vec![100756]), 50346);
    }
}
