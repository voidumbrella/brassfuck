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
    MoveRight,
    MoveLeft,
    IncValue,
    DecValue,
    Output,
    Input,
    OpenLoop(usize),
    CloseLoop(usize),
}

const DEFAULT_MEMORY_SIZE: usize = 30_000;

impl Interpreter {
    fn new() -> Interpreter {
        Interpreter {
            memory: vec![0; DEFAULT_MEMORY_SIZE],
            mp: 0,
            ip: 0,
        }
    }

    fn execute(self: &mut Interpreter, instructions: &[Op]) {
        while let Some(instruction) = instructions.get(self.ip) {
            self.ip += 1;
            let cell = &mut self.memory[self.mp];

            match *instruction {
                Op::MoveRight => {
                    if self.mp == self.memory.len() { self.mp = 0; }
                    else { self.mp += 1; }
                    // self.mp = (self.mp + 1) % self.memory.len();
                }
                Op::MoveLeft => {
                    if self.mp == 0 { self.mp = self.memory.len() - 1; }
                    else { self.mp -= 1; }
                    // self.mp = (self.mp - 1) % self.memory.len();
                }

                Op::IncValue =>
                    *cell = cell.wrapping_add(1),
                Op::DecValue =>
                    *cell = cell.wrapping_sub(1),

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
            }
        }
    }
}

fn parse(source: &str) -> Vec<Op> {
    let mut instructions = Vec::new();

    // Stack to store loops
    let mut stack = Vec::new();

    let ops: Vec<char> = source
        .chars()
        .filter(|c| match *c {
            '>' | '<' | '+' | '-' | '.' | ',' | '[' | ']' => true,
            _ => false,
        })
        .collect();

    let mut iter = ops.iter().enumerate();
    while let Some((i, op)) = iter.next() {
        match op {
            '>' => instructions.push(Op::MoveRight),
            '<' => instructions.push(Op::MoveLeft),
            '+' => instructions.push(Op::IncValue),
            '-' => instructions.push(Op::DecValue),
            '.' => instructions.push(Op::Output),
            ',' => instructions.push(Op::Input),
            '[' => {
                // Search for address of end of loop
                let mut depth = 0;
                for j in i+1..ops.len() {
                    if ops[j] == ']' {
                        if depth == 0 {
                            // Push address of start of loop onto stack
                            stack.push(i);
                            instructions.push(Op::OpenLoop(j));
                            break;
                        }
                        else { depth -= 1; }
                    } else if ops[j] == '[' { depth += 1; }
                }
            },
            ']' => {
                // Pop the address of start of loop block from stack.
                let open_index = match stack.pop() {
                    None => panic!("unexpected end of loop"),
                    Some(x) => x,
                };
                instructions.push(Op::CloseLoop(open_index));
            },
            _ => (),
        }
    }

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
