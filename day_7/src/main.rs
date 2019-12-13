use std::{
    convert::{TryFrom, TryInto},
    error::Error,
    fs::read_to_string,
    io::{BufRead, Read, Write, Cursor},
    str::FromStr,
};

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
    Ok(line.trim().parse::<T>()?)
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

impl Parameter {
    /// rust soruce path
    fn run<R: BufRead + Read, W: Write>(
        self,
        ip: usize,
        code: &mut [isize],
        inn: &mut R,
        out: &mut W,
    ) -> Result<usize, Box<dyn Error>> {
        match self.opcode {
            1 => {
                self.a.set_ip(
                    ip + 3,
                    code,
                    self.c.get_ip(ip + 1, code)? + self.b.get_ip(ip + 2, code)?,
                )?;
                Ok(ip + 4)
            }
            2 => {
                self.a.set_ip(
                    ip + 3,
                    code,
                    self.c.get_ip(ip + 1, code)? * self.b.get_ip(ip + 2, code)?,
                )?;
                Ok(ip + 4)
            }
            3 => {
                self.a.set_ip(ip + 1, code, get_stdin(inn)?)?;
                Ok(ip + 2)
            }
            4 => {
                writeln!(out, "{}", self.c.get_ip(ip + 1, code)?)?;
                Ok(ip + 2)
            }
            5 => match self.c.get_ip(ip + 1, code)? {
                0 => Ok(ip + 3),
                _ => Ok(self.b.get_ip(ip + 2, code)?.try_into()?),
            },
            6 => match self.c.get_ip(ip + 1, code)? {
                0 => Ok(self.b.get_ip(ip + 2, code)?.try_into()?),
                _ => Ok(ip + 3),
            },
            7 => {
                self.a.set_ip(
                    ip + 3,
                    code,
                    if self.c.get_ip(ip + 1, code)? < self.b.get_ip(ip + 2, code)? {
                        1
                    } else {
                        0
                    },
                )?;
                Ok(ip + 4)
            }
            8 => {
                self.a.set_ip(
                    ip + 3,
                    code,
                    if self.c.get_ip(ip + 1, code)? == self.b.get_ip(ip + 2, code)? {
                        1
                    } else {
                        0
                    },
                )?;
                Ok(ip + 4)
            }
            99 => Ok(std::usize::MAX),
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

fn run_instruction<R: BufRead + Read, W: Write>(
    ip: usize,
    code: &mut [isize],
    inn: &mut R,
    out: &mut W,
) -> Result<usize, Box<dyn Error>> {
    let param: Parameter = code[ip].try_into()?;
    param.run(ip, code, inn, out)
}

fn intcode_computer<R: BufRead + Read, W: Write>(codes: &mut [isize], inn: &mut R, out: &mut W) -> Result<(), Box<dyn Error>> {
    let mut ip = 0;
    loop {
        match run_instruction(ip, codes, inn, out) {
            Ok(p) => match p {
                std::usize::MAX => return Ok(()),
                _ => ip = p,
            },
            Err(n) => {
                eprintln!("code: {:?}\nip:{}", &codes[ip..ip + 4], ip);
                codes
                    .iter()
                    .enumerate()
                    .for_each(|(p, v)| eprintln!("{}: {}", p, v));
                return Err(n);
            }
        }
    }
}

fn get_amplifier_output(base_code: &[isize], phaces: &[usize]) -> Result<usize, Box<dyn Error>> {
    let mut inn;
    let mut out = Cursor::new(b"0".to_vec());
    for p in phaces {
        inn = Cursor::new(p.to_string().into_bytes());
        let inp = inn.get_mut();
        inp.extend(b"\n");
        inp.append(out.get_mut());
        inp.extend(b"\n");
        out = Cursor::new(Vec::new());
        let mut this_code: Vec<_> = base_code.to_vec();
        intcode_computer(&mut this_code, &mut inn, &mut out)?;
    }
    Ok(String::from_utf8(out.into_inner())?.trim().parse()?)
}

fn highest_input(code: &[isize]) -> Result<usize, Box<dyn Error>> {
    let values = (0usize..5)
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

    println!("{}", highest_input(&codes)?);

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

        intcode_computer(&mut a, &mut inn, &mut out).unwrap();

        let ans: isize = String::from_utf8(out.into_inner()).unwrap().trim().parse().unwrap();
        assert_eq!(ans, 0);
    }
}
