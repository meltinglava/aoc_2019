use std::{fs::read_to_string, io};

fn main() -> io::Result<()> {
    let file = read_to_string("input.txt")?;
    let codes: Vec<_> = file
        .trim()
        .split(',')
        .map(|line| line.parse::<usize>().unwrap())
        .collect();

    println!("Part1: {}", intcode_computer(&mut codes.clone(), 12, 2));
    println!("Part2: {}", find_noun_word_combo(&codes));
    Ok(())
}

fn find_noun_word_combo(codes: &Vec<usize>) -> usize {
    for noun in 0..100 {
        for word in 0..100 {
            if intcode_computer(&mut codes.clone(), noun, word) == 19690720 {
                return format!("{}{}", noun, word).parse().unwrap();
            }
        }
    }
    unreachable!("Did not find any code that ended with: 19690720")
}

fn intcode_computer(codes: &mut [usize], noun: usize, word: usize) -> usize {
    let mut pos = 0;
    codes[1] = noun;
    codes[2] = word;
    loop {
        match codes[pos] {
            99 => break codes[0],
            1 => codes[codes[pos + 3]] = codes[codes[pos + 1]] + codes[codes[pos + 2]],
            2 => codes[codes[pos + 3]] = codes[codes[pos + 1]] * codes[codes[pos + 2]],
            n => panic!("reached unknown code: {}", n),
        }
        pos += 4;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_name() {
        assert_eq!(
            intcode_computer(
                &mut vec![1, 9, 10, 3, 2, 3, 11, 0, 99, 30, 40, 50, 1],
                12,
                2
            ),
            150
        )
    }
}
