use std::env;
use std::fs::File;
use std::io::{Read, Write};

struct Interpreter {
    memory: Vec<u8>,
    mp: usize,
    ip: usize,
}

#[derive(Debug)]
enum Op {
    MoveRight(usize),
    MoveLeft(usize),
    Add(u8),
    Sub(u8),
    Output,
    Input,
    OpenLoop(usize),
    CloseLoop(usize),

    // Optimizations of common idioms
    Clear,
}

const DEFAULT_MEMORY_SIZE: usize = 65_536;

impl Interpreter {
    fn new() -> Interpreter {
        Interpreter {
            memory: vec![0; DEFAULT_MEMORY_SIZE],
            mp: 0,
            ip: 0,
        }
    }

    fn execute(&mut self, instructions: &[Op]) {
        while let Some(instruction) = instructions.get(self.ip) {
            self.ip += 1;
            let cell = &mut self.memory[self.mp];

            match *instruction {
                Op::MoveRight(n) =>
                    self.mp += n,
                Op::MoveLeft(n) =>
                    self.mp -= n,

                Op::Add(n) =>
                    *cell = cell.wrapping_add(n),
                Op::Sub(n) =>
                    *cell = cell.wrapping_sub(n),

                Op::Output => {
                    print!("{}", *cell as char);
                }
                Op::Input => {
                    std::io::stdout().flush().unwrap();
                    let input = match std::io::stdin().bytes().next() {
                        Some(b) => b.unwrap(),
                        None => 0,
                    };
                    self.memory[self.mp] = input;
                },

                Op::OpenLoop(goto) => {
                    if *cell == 0 { self.ip = goto; }
                },

                Op::CloseLoop(goto) => {
                    if *cell != 0 { self.ip = goto; }
                },

                Op::Clear => *cell = 0,
            }
        }
    }
}

fn match_ops(source: &[char], to_match: &str) -> bool {
    for (s, c) in to_match.chars().zip(source.iter()) {
        if s != *c { return false; }
    }
    source.len() >= to_match.len()
}

fn parse(source: &str) -> Vec<Op> {
    let mut instructions = Vec::new();

    // Stack to store loops
    // let mut stack = Vec::new();

    let source: Vec<char> = source
        .chars()
        .filter(|c| match *c {
            '>' | '<' | '+' | '-' | '.' | ',' | '[' | ']' => true,
            _ => false,
        })
        .collect();

    // Compress instructions
    let mut iter = source.iter().enumerate().peekable();
    while let Some((i, c)) = iter.next() {
        let mut n = 1;
        match c {
            '>' => {
                while let Some((_, '>')) = iter.peek() {
                    iter.next();
                    n += 1;
                }
                instructions.push(Op::MoveRight(n));
            }
            '<' => {
                while let Some((_, '<')) = iter.peek() {
                    iter.next();
                    n += 1;
                }
                instructions.push(Op::MoveLeft(n));
            }
            '+' => {
                while let Some((_, '+')) = iter.peek() {
                    iter.next();
                    n += 1;
                }
                instructions.push(Op::Add(n as u8));
            }
            '-' => {
                while let Some((_, '-')) = iter.peek() {
                    iter.next();
                    n += 1;
                }
                instructions.push(Op::Sub(n as u8));
            }
            '.' => instructions.push(Op::Output),
            ',' => instructions.push(Op::Input),

            // Loop instructions are stored with temporary values since
            // the address to jump to are unknown currently
            // Kinda weird but can't think of a better design right now
            '[' => {
                if match_ops(&source[i+1..], "-]") {
                    instructions.push(Op::Clear);
                    iter.nth(1);
                }
                else { instructions.push(Op::OpenLoop(0)); }
            }
            ']' => instructions.push(Op::CloseLoop(0)),
            _ => (),
        }
    }

    // TODO: Syntax errors should not panic
    
    // Determine the addresses where each loop instructions should jump to.
    let mut stack: Vec<(usize, &mut Op)> = Vec::new();
    for (i, op) in instructions.iter_mut().enumerate() {
        // Push address of any start of loops onto the stack.
        // If an end of loop is found, pop the matching start
        // and set the jump addresses for both to each other.
        match op {
            Op::OpenLoop(_) => stack.push((i, op)),
            Op::CloseLoop(_) => {
                let (goto, open) = stack.pop()
                    .unwrap_or_else(|| panic!("Unexpected end of loop"));
                *op = Op::CloseLoop(goto);
                *open = Op::OpenLoop(i);
            }
            _ => (),
        }
    }
    assert!(stack.is_empty(), "Unterminated loop");

    instructions
}

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() != 2 {
        eprintln!("usage: bf <file>");
        std::process::exit(1);
    }

    let filename = &args[1];

    let mut file = File::open(filename)
        .unwrap_or_else(|err| {
            eprintln!("Error opening file: {}", err);
            std::process::exit(1);
        });
    
    let mut source = String::new();
    file.read_to_string(&mut source)
        .unwrap_or_else(|err| {
            eprintln!("Error reading from file: {}", err);
            std::process::exit(1);
        });

    let program = parse(&source);
    let mut interpreter = Interpreter::new();
    interpreter.execute(&program);
}
