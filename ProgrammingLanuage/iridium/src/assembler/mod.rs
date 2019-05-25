use crate::assembler::program_parsers::{program, Program};
use crate::instruction::Opcode;
use nom::types::CompleteStr;

pub mod directive_parsers;
pub mod instruction_parsers;
pub mod integer_parsers;
pub mod label_parsers;
pub mod opcode_parsers;
pub mod operand_parsers;
pub mod program_parsers;
pub mod register_parsers;

const PIE_HEADER_PREFIX: [u8; 4] = [45, 50, 49, 45];
const PIE_HEADER_LENGTH: usize = 64;

#[derive(Debug, PartialEq)]
pub enum Token {
    Op { code: Opcode },
    Register { reg_num: u8 },
    IntegerOperand { value: i32 },
    LabelDeclaration { name: String },
    LabelUsage { name: String },
    Directive { name: String },
}

#[derive(Debug)]
pub enum SymbolType {
    Label,
}

#[derive(Debug)]
pub struct Symbol {
    name: String,
    offset: u32,
    symbol_type: SymbolType,
}

impl Symbol {
    pub fn new(name: String, symbol_type: SymbolType, offset: u32) -> Symbol {
        Symbol {
            name,
            symbol_type,
            offset,
        }
    }
}

#[derive(Debug)]
pub struct SymbolTable {
    symbols: Vec<Symbol>,
}

impl SymbolTable {
    pub fn new() -> SymbolTable {
        SymbolTable { symbols: vec![] }
    }

    pub fn add_symbol(&mut self, s: Symbol) {
        self.symbols.push(s);
    }

    pub fn symbol_value(&self, s: &str) -> Option<u32> {
        for symbol in &self.symbols {
            if symbol.name == s {
                return Some(symbol.offset);
            }
        }
        None
    }
}

#[derive(Debug, PartialEq)]
pub enum AssemblerPhase {
    First,
    Second,
}

// #[derive(Debug)]
// pub struct Assembler {
//     phase: AssemblerPhase,
// }

#[derive(Debug)]
pub struct Assembler {
    pub phase: AssemblerPhase,
    pub symbols: SymbolTable,
}

impl Assembler {
    pub fn new() -> Assembler {
        Assembler {
            phase: AssemblerPhase::First,
            symbols: SymbolTable::new(),
        }
    }

    fn write_pie_header(&self) -> Vec<u8> {
        let mut header = vec![];
        for byte in PIE_HEADER_PREFIX.into_iter() {
            header.push(byte.clone());
        }
        while header.len() <= PIE_HEADER_LENGTH {
            header.push(0 as u8);
        }
        header
    }

    pub fn assemble(&mut self, raw: &str) -> Option<Vec<u8>> {
        match program(CompleteStr(raw)) {
            Ok((_remainder, program)) => {
                let mut assembled_program = self.write_pie_header();
                self.process_first_phase(&program);
                let mut body = self.process_second_phase(&program);
                assembled_program.append(&mut body);
                Some(assembled_program)
            }
            Err(e) => {
                println!("There was an error assembling the code: {:?}", e);
                None
            }
        }
    }

    // Extract all labels, build symbol table
    fn process_first_phase(&mut self, p: &Program) {
        self.extract_labels(p);
        self.phase = AssemblerPhase::Second;
    }

    // Build program(byte code)
    fn process_second_phase(&mut self, p: &Program) -> Vec<u8> {
        let mut program = vec![];
        for i in &p.instructions {
            let mut bytes = i.to_bytes(&self.symbols);
            program.append(&mut bytes);
        }
        program
    }

    fn extract_labels(&mut self, p: &Program) {
        let mut c = 0;
        for i in &p.instructions {
            if i.is_label() {
                match i.label_name() {
                    Some(name) => {
                        let symbol = Symbol::new(name, SymbolType::Label, c);
                        self.symbols.add_symbol(symbol);
                    }
                    None => {}
                };
            }
            c += 4;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::super::*;
    use super::*;
    use crate::vm::VM;

    #[test]
    fn test_symbol_table() {
        let mut sym = SymbolTable::new();
        let new_symbol = Symbol::new("test".to_string(), SymbolType::Label, 12);
        sym.add_symbol(new_symbol);
        assert_eq!(sym.symbols.len(), 1);
        let v = sym.symbol_value("test");
        assert_eq!(true, v.is_some());
        let v = v.unwrap();
        assert_eq!(v, 12);
        let v = sym.symbol_value("does_not_exist");
        assert_eq!(v.is_some(), false);
    }

    #[test]
    fn test_assemble_program() {
        let mut asm = Assembler::new();
        // TODO jmpe @test
        let test_string =
            "load $0 #100\n load $1 #1\n load $2 #0\n test: inc $0\nneq $0 $2\n jmpe @test\n hlt";
        let program = asm.assemble(test_string).unwrap();
        let mut vm = VM::new();
        assert_eq!(program.len(), 19 + 65);
        vm.add_bytes(program);
        assert_eq!(vm.program.len(), 19 + 65);
    }

    #[test]
    fn test_write_pie_header() {
        let mut asm = Assembler::new();
        let header = asm.write_pie_header();
        assert_eq!(header.len(), 65);
    }
}
