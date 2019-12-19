use std::{
    convert::{TryFrom, TryInto},
    error::Error,
    fs::read_to_string,
    io::{BufRead, Read, Write, Cursor},
    str::FromStr,
};

fn make_input(p: usize) -> Cursor<Vec<u8>> {
    Cursor::new(format!("{}\n", p).into_bytes())
}

#[derive(Debug)]
struct IntcodeComputer<'a> {
    code: &'a [isize],
    ip: usize,
    status: Status,
    inn: Cursor<Vec<u8>>
}

impl<'a> IntcodeComputer<'a> {
    fn new(code: &'a [isize], p: usize) -> Self {
        Self {
            code,
            ip: 0,
            status: Status::NotYetStarted,
            inn: make_input(p),
        }
    }

    fn run_instruction<W: Write>(
        &mut self,
        out: &mut W,
    ) -> Result<Status, Box<dyn Error>> {
        let param: Parameter = self.code[self.ip].try_into()?;
        param.run(&mut self.ip, &mut self.code, &mut self.inn, out)
    }
}

use itertools::Itertools;

#[derive(Debug, Eq, PartialEq, Clone, Copy)]
enum ParameterMode {
    PositionMode,
    ImmediateMode,
}

impl TryFrom<usize> for ParameterMode {
    type Error = String;

    fn try_from(num: usize) -> Result<Self, Self::Error> {
        match num {
            0 => Ok(ParameterMode::PositionMode),
            1 => Ok(ParameterMode::ImmediateMode),
            n => Err(format!(
                "Unkown ParameterMode (0 and 1 is valid), got: {}",
                n
            )),
        }
    }
}

pub fn get_stdin<T, R: BufRead + Read>(inn: &mut R) -> Result<T, Box<dyn Error>>
where
    T: FromStr,
    T::Err: 'static + Error,
{

    let mut line = String::new();
    inn.read_line(&mut line)?;
    Ok(dbg!(line.trim()).parse::<T>()?)
}

impl Default for ParameterMode {
    fn default() -> Self {
        Self::PositionMode
    }
}

impl ParameterMode {
    fn get_ip(self, ip: usize, code: &[isize]) -> Result<isize, Box<dyn Error>> {
        match self {
            ParameterMode::PositionMode => Ok(code[usize::try_from(code[ip])?]),
            ParameterMode::ImmediateMode => Ok(code[ip]),
        }
    }
    fn set_ip(self, ip: usize, code: &mut [isize], value: isize) -> Result<(), Box<dyn Error>> {
        match self {
            ParameterMode::PositionMode => code[usize::try_from(code[ip])?] = value,
            ParameterMode::ImmediateMode => code[ip] = value,
        }
        Ok(())
    }
}

#[derive(Debug)]
struct Parameter {
    opcode: usize,
    a: ParameterMode,
    b: ParameterMode,
    c: ParameterMode,
}

#[derive(Debug, PartialEq, Eq)]
enum Status {
    Blocking,
    Running,
    Hault,
    NotYetStarted,
}

impl Parameter {
    /// rust soruce path
    fn run<R: BufRead + Read, W: Write>(
        self,
        ip: &mut usize,
        code: &mut [isize],
        inn: &mut R,
        out: &mut W,
    ) -> Result<Status, Box<dyn Error>> {
        dbg!(&ip);
        match dbg!(self.opcode) {
            1 => { // pluss
                self.a.set_ip(
                    *ip + 3,
                    code,
                    self.c.get_ip(*ip + 1, code)? + self.b.get_ip(*ip + 2, code)?,
                )?;
                *ip += 4;
                Ok(Status::Running)
            }
            2 => { // multiply
                self.a.set_ip(
                    *ip + 3,
                    code,
                    self.c.get_ip(*ip + 1, code)? * self.b.get_ip(*ip + 2, code)?,
                )?;
                *ip += 4;
                Ok(Status::Running)
            }
            3 => { // read stdin(like)
                match get_stdin(inn) {
                    Ok(n) => {
                        self.a.set_ip(*ip + 1, code, n)?;
                        *ip += 2;
                        Ok(Status::Running)
                    }
                    Err(_) => Ok(Status::Blocking)
                }
            }
            4 => { // print(strout_like)
                writeln!(out, "{}", self.c.get_ip(*ip + 1, code)?)?;
                *ip += 2;
                Ok(Status::Running)
            }
            5 => { // jmp if not 0
                match self.c.get_ip(*ip + 1, code)? {
                    0 => *ip += 3,
                    _ => *ip = self.b.get_ip(*ip + 2, code)?.try_into()?,
                };
                Ok(Status::Running)
            }
            6 => { // jump if 0
                match self.c.get_ip(*ip + 1, code)? {
                    0 => *ip = self.b.get_ip(*ip + 2, code)?.try_into()?,
                    _ => *ip += 3,
                }
                Ok(Status::Running)
            },
            7 => { //cmp lt
                self.a.set_ip(
                    *ip + 3,
                    code,
                    if self.c.get_ip(*ip + 1, code)? < self.b.get_ip(*ip + 2, code)? {
                        1
                    } else {
                        0
                    },
                )?;
                *ip += 4;
                Ok(Status::Running)
            }
            8 => { // cmp equals
                self.a.set_ip(
                    *ip + 3,
                    code,
                    if self.c.get_ip(*ip + 1, code)? == self.b.get_ip(*ip + 2, code)? {
                        1
                    } else {
                        0
                    },
                )?;
                *ip += 4;
                Ok(Status::Running)
            }
            99 => Ok(Status::Hault),
            n => Err(format!("Unknown opcode: {}\n", n).into()),
        }
    }
}

fn mod_and_divide(num: &mut usize, mod_by: usize) -> usize {
    let ans = *num % mod_by;
    *num /= mod_by;
    ans
}

impl TryFrom<isize> for Parameter {
    type Error = Box<dyn Error>;

    fn try_from(input: isize) -> Result<Self, Self::Error> {
        let mut input: usize = input.try_into()?;
        let opcode = mod_and_divide(&mut input, 100);
        let c = mod_and_divide(&mut input, 10).try_into()?;
        let b = mod_and_divide(&mut input, 10).try_into()?;
        let a = mod_and_divide(&mut input, 10).try_into()?;
        Ok(Self { opcode, a, b, c })
    }
}

fn intcode_computer<W: Write>(
    computer: &mut IntcodeComputer,
    out: &mut W
) -> Result<Status, Box<dyn Error>> {
    loop {
        match computer.run_instruction(out) {
            Ok(p) => match p {
                Status::Hault | Status::Blocking => return dbg!(Ok(p)),
                Status::Running => (),
                Status::NotYetStarted => unreachable!(),
            },
            Err(n) => {
                eprintln!("Computer: {:#?}", &computer);
                computer.code
                    .iter()
                    .enumerate()
                    .for_each(|(p, v)| eprintln!("{}: {}", p, v));
                return Err(n);
            }
        }
    }
}

fn get_amplifier_output(base_code: &[isize], phaces: &[usize]) -> Result<usize, Box<dyn Error>> {
    let mut codes: Vec<_> = phaces
        .iter()
        .map(|p| IntcodeComputer::new(base_code, *p))
        .take(5)
        .collect();
    let mut out = Cursor::new(b"0".to_vec());
    let mut haults = 0;
    for mashine in (0..5).cycle() {
        let mut computer = codes.get_mut(mashine).unwrap();
        computer.inn.get_mut().append(out.get_mut());
        out = Cursor::new(Vec::new());
        intcode_computer(&mut computer, &mut out)?;
        match computer.status {
            Status::Hault => match haults {
                5 => break,
                _ => haults += 1,
            }
            Status::Blocking => (),
            Status::Running => unreachable!("We should never have an running intcode at the moment where we are Running"),
            Status::NotYetStarted => unreachable!("We should never have an running intcode at the moment where we are Running"),
        };
        dbg!(haults);
    }
    Ok(String::from_utf8(out.into_inner())?.trim().parse()?)
}

fn highest_input_part1(code: &[isize]) -> Result<usize, Box<dyn Error>> {
    let values = (0usize..5)
        .permutations(5)
        .map(|n| get_amplifier_output(code, &n))
        .collect::<Result<Vec<_>, Box<dyn Error>>>()?;
    Ok(values
       .into_iter()
       .max()
       .ok_or("No output given")?)
}

fn highest_input_part2(code: &[isize]) -> Result<usize, Box<dyn Error>> {
    let values = (5..10)
        .permutations(5)
        .map(|n| get_amplifier_output(code, &n))
        .collect::<Result<Vec<_>, Box<dyn Error>>>()?;
    Ok(values
       .into_iter()
       .max()
       .ok_or("No output given")?)
}

fn main() -> Result<(), Box<dyn Error>> {
    let file = read_to_string("input.txt")?;
    let codes: Vec<_> = file
        .trim()
        .split(',')
        .map(|line| line.parse::<isize>().unwrap())
        .collect();
    println!("Part1: {}", highest_input_part1(&codes)?);

    println!("Part2: {}", highest_input_part2(&codes)?);

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_run() {
        let mut a = [3,3,1105,-1,9,1101,0,0,12,4,12,99,1];

        let mut inn = Cursor::new("0");
        let mut out = Cursor::new(Vec::new());

        intcode_computer(&mut a, &mut 0, &mut inn, &mut out).unwrap();

        let ans: isize = String::from_utf8(out.into_inner()).unwrap().trim().parse().unwrap();
        assert_eq!(ans, 0);
    }
}
