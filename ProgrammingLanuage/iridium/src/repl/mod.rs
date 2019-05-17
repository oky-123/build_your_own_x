use crate::assembler::program_parsers::program;
use crate::vm::VM;

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
                ".run_once" => {
                    self.vm.run_once();
                }
                ".run" => {
                    self.vm.run();
                }
                _ => {
                    let program = match program(buffer.into()) {
                        Ok((_, program)) => program,
                        Err(e) => {
                            println!("Unable to parse input");
                            println!("{}", e);
                            continue;
                        }
                    };

                    self.vm.program.append(&mut program.to_bytes());
                    // self.vm.run_once();
                }
            }
        }
    }
}
