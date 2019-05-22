#[macro_use]
extern crate nom;
extern crate byteorder;

pub mod assembler;
pub mod instruction;
pub mod repl;
pub mod vm;

fn main() {
    start_repl();
}

fn start_repl() {
    let mut repl = repl::REPL::new();
    repl.run();
}
