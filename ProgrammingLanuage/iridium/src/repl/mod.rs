use crate::assembler::program_parsers::program;
use crate::assembler::Assembler;
use crate::scheduler::Scheduler;
use crate::vm::VM;

use std;
use std::fs::read_to_string;
use std::io;
use std::io::Write;
use std::path::Path;

use nom::types::CompleteStr;

pub struct REPL {
    pub command_buffer: Vec<String>,
    pub vm: VM,
    pub asm: Assembler,
    pub scheduler: Scheduler,
}

impl REPL {
    pub fn new() -> REPL {
        REPL {
            vm: VM::new(),
            command_buffer: vec![],
            asm: Assembler::new(),
            scheduler: Scheduler::new(),
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
                ".ro" => {
                    println!("Listing ro of VM:");
                    println!("{:?}", self.vm.ro_data);
                    println!("End of ro Listing");
                }
                ".symbols" => {
                    println!("Listing symbols of VM:");
                    println!("{:?}", self.asm.symbols);
                    println!("End of symbols Listing");
                }
                ".find_symbol" => {
                    let mut b = String::new();
                    let _stdin = io::stdin();
                    print!("input label name: ");
                    io::stdout().flush().expect("Unable to flush stdout");
                    // Look at string from user
                    _stdin
                        .read_line(&mut b)
                        .expect("Unable to read line from user");
                    let b = b.trim();

                    println!("{}", self.asm.symbols.symbol_value(b).unwrap())
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
                ".load_file" => {
                    print!("Please enter the path to the file you wish to load: ");
                    io::stdout().flush().expect("Unable to flush stdout");
                    let mut tmp = String::new();
                    stdin
                        .read_line(&mut tmp)
                        .expect("Unable to read line from user");
                    let tmp = tmp.trim();
                    let filename = Path::new(&tmp);
                    let contents = match read_to_string(filename) {
                        Ok(f) => f,
                        Err(e) => {
                            println!("There was an error opening that file: {:?}", e);
                            continue;
                        }
                    };
                    let program = match program(CompleteStr(&contents)) {
                        // Rusts pattern matching is pretty powerful an can even be nested
                        Ok((_, program)) => program,
                        Err(e) => {
                            println!("Unable to parse input: {:?}", e);
                            continue;
                        }
                    };
                    self.vm
                        .program
                        .append(&mut program.to_bytes(&self.asm.symbols));
                }
                _ => {
                    let program = match program(buffer.into()) {
                        Ok((_, program)) => {
                            println!("{:?}", program);
                            program
                        }
                        Err(e) => {
                            println!("Unable to parse input");
                            println!("{}", e);
                            continue;
                        }
                    };

                    self.vm
                        .program
                        .append(&mut program.to_bytes(&self.asm.symbols));
                    self.vm.run_once();
                }
            }
        }
    }
}
