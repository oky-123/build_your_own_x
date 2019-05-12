pub mod instruction;
pub mod repl;
pub mod vm;

fn main() {
    println!("Hello, world!");
    let mut repl = repl::REPL::new();
    repl.run();
}
