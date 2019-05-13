use crate::assembler::program_parsers::program;
use crate::vm::VM;

use nom::types::CompleteStr;
use std;
use std::io;
use std::io::Write;

pub struct REPL {
    command_buffer: Vec<String>,
    vm: VM,
}

impl REPL {
    pub fn new() -> REPL {
        REPL {
            vm: VM::new(),
            command_buffer: vec![],
        }
    }

    pub fn run(&mut self) {
        println!("Welcome to Iridium! Let's be productive.");
        loop {
            let mut buffer = String::new();
            let stdin = io::stdin();

            print!(">>> ");
            io::stdout().flush().expect("Unable to flush stdout");

            // Look at string from user
            stdin
                .read_line(&mut buffer)
                .expect("Unable to read line from user");

            let buffer = buffer.trim();
            self.command_buffer.push(buffer.to_string());
            match buffer {
                ".quit" => {
                    println!("Farewell! Have a great day!");
                    std::process::exit(0);
                }
                ".history" => {
                    for command in &self.command_buffer {
                        println!("{}", command);
                    }
                }
                ".program" => {
                    println!("Listing instructions currently in VM's program vector:");
                    for instruction in &self.vm.program {
                        println!("{}", instruction);
                    }
                    println!("End of Program Listing");
                }
                ".registers" => {
                    println!("Listing registers and all contents:");
                    println!("{:#?}", self.vm.registers);
                    println!("End of Register Listing")
                }
                ".pc" => {
                    println!("{}", self.vm.pc);
                }
                _ => {
                    let parsed_program = program(CompleteStr(buffer));
                    println!("{:?}", parsed_program);
                    if !parsed_program.is_ok() {
                        println!("Unable to parse input");
                        continue;
                    }
                    let (_, result) = parsed_program.unwrap();
                    let bytecode = result.to_bytes();
                    println!("{:?}", bytecode);
                    self.vm.add_bytes(bytecode);
                    self.vm.run_once();
                }
            }
        }
    }
}
