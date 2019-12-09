use beginner_tools::get_stdin;
use std::{
    convert::{TryFrom, TryInto},
    error::Error,
    fs::read_to_string,
};

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
    fn run(self, ip: usize, code: &mut [isize]) -> Result<usize, Box<dyn Error>> {
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
                     self.a.set_ip(ip + 3, code, get_stdin()?)?;
                Ok(ip + 2)
            }
            4 => {
                     println!("{}", self.c.get_ip(ip + 1, code)?);
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

fn run_instruction(ip: usize, code: &mut [isize]) -> Result<usize, Box<dyn Error>> {
    let param: Parameter = code[ip].try_into()?;
    param.run(ip, code)
}

fn intcode_computer(codes: &mut [isize]) -> Result<(), Box<dyn Error>> {
    let mut ip = 0;
    loop {
        match run_instruction(ip, codes) {
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

fn main() -> Result<(), Box<dyn Error>> {
    let file = read_to_string("input.txt")?;
    let mut codes: Vec<_> = file
        .trim()
        .split(',')
        .map(|line| line.parse::<isize>().unwrap())
        .collect();

    intcode_computer(&mut codes)?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_run() {
        let mut a = [1002, 4, 3, 4, 33];
        //intcode_computer(&mut a);
        run_instruction(0, &mut a).unwrap();
        assert_eq!(a[4], 99);
    }
}
