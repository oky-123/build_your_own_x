#[macro_use]
extern crate nom;
extern crate byteorder;

#[macro_use]
extern crate clap;

pub mod assembler;
pub mod instruction;
pub mod repl;
pub mod vm;

use clap::App;
use std::fs::{read_to_string, File};
use std::path::Path;

fn main() {
    let yaml = load_yaml!("../cli.yml");
    let matches = App::from_yaml(yaml).get_matches();
    let target_file = matches.value_of("INPUT_FILE");
    match target_file {
        Some(filename) => {
            let program = read_file(filename);
            let mut asm = assembler::Assembler::new();
            let mut vm = vm::VM::new();
            let program = asm.assemble(&program);
            match program {
                Ok(p) => {
                    vm.add_bytes(p);
                    vm.run();
                    resume_repl(vm, asm);
                }
                _ => {}
            }
        }
        None => {
            start_repl();
        }
    }
}

fn start_repl() {
    let mut repl = repl::REPL::new();
    repl.run();
}

fn resume_repl(vm: vm::VM, asm: assembler::Assembler) {
    let mut repl = repl::REPL::new();
    repl.vm = vm;
    repl.asm = asm;
    repl.run();
}

fn read_file(tmp: &str) -> String {
    let filename = Path::new(tmp);
    match File::open(filename) {
        Ok(_) => match read_to_string(filename) {
            Ok(contents) => {
                return contents;
            }
            Err(e) => {
                println!("There was an error reading file: {:?}", e);
                std::process::exit(1);
            }
        },
        Err(e) => {
            println!("File not found: {:?}", e);
            std::process::exit(1)
        }
    }
}
