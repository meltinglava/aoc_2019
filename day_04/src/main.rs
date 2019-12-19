use itertools::Itertools;

use std::{
    cmp::Ordering::{self, Equal, Greater, Less},
    collections::HashMap,
};

fn generate_digits(mut num: usize) -> Vec<usize> {
    let mut v = Vec::new();
    while num != 0 {
        v.push(num % 10);
        num /= 10;
    }
    v.reverse();
    v
}

fn generate_orderings_part1(nums: &[usize]) -> HashMap<Ordering, usize> {
    let mut map = HashMap::new();
    map.insert(Equal, 0);
    map.insert(Greater, 0);
    map.insert(Less, 0);
    for i in 0..nums.len() - 1 {
        *map.get_mut(&nums[i].cmp(&nums[i + 1])).unwrap() += 1;
    }
    map
}

fn generate_orderings_part2(nums: &[usize]) -> Vec<Ordering> {
    let mut orders = Vec::new();
    for i in 0..nums.len() - 1 {
        orders.push(nums[i].cmp(&nums[i + 1]));
    }
    orders
}

fn valid_password_part1(num: &usize) -> bool {
    let digits = generate_digits(*num);
    if digits.len() != 6 {
        return false;
    }
    let orderings = generate_orderings_part1(&digits);
    orderings.into_iter().all(|(o, n)| match o {
        Less => true,
        Equal => n >= 1,
        Greater => n == 0,
    })
}

fn valid_password_part2(num: &usize) -> bool {
    let digits = generate_digits(*num);
    let orders = generate_orderings_part2(&digits);
    let mut found = 0;
    for (key, group) in &orders.into_iter().group_by(|ord| *ord) {
        match key {
            Equal if group.count() == 1 => found += 1,
            Greater => return false,
            _ => (),
        }
    }
    found >= 1
}

fn main() {
    let part1 = (347_312..=805_915)
        .filter(valid_password_part1)
        .count();
    println!("{}", part1);
    let part2 = (347_312..=805_915)
        .filter(valid_password_part2)
        .count();
    println!("{}", part2);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_digits() {
        assert_eq!(generate_digits(12345), vec![1, 2, 3, 4, 5]);
        assert_eq!(generate_digits(123450), vec![1, 2, 3, 4, 5, 0]);
    }

    #[test]
    fn test_other_user() {
        let part1 = (134564..=585159)
            .into_iter()
            .filter(valid_password_part1)
            .count();
        assert_eq!(part1, 1929);
    }
}
