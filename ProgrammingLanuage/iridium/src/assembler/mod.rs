use crate::assembler::assembler_errors::AssemblerError;
use crate::assembler::instruction_parsers::AssemblerInstruction;
use crate::assembler::program_parsers::{program, Program};
use crate::instruction::Opcode;
use nom::types::CompleteStr;

pub mod assembler_errors;
pub mod directive_parsers;
pub mod instruction_parsers;
pub mod integer_parsers;
pub mod irstring_parsers;
pub mod label_parsers;
pub mod opcode_parsers;
pub mod operand_parsers;
pub mod program_parsers;
pub mod register_parsers;

pub const PIE_HEADER_PREFIX: [u8; 4] = [45, 50, 49, 45];
pub const PIE_HEADER_LENGTH: usize = 64;

#[derive(Debug, PartialEq)]
pub enum Token {
    Op { code: Opcode },
    Register { reg_num: u8 },
    IntegerOperand { value: i32 },
    LabelDeclaration { name: String },
    LabelUsage { name: String },
    Directive { name: String },
    IrString { name: String },
}

#[derive(Debug)]
pub enum SymbolType {
    Label,
}

#[derive(Debug)]
pub struct Symbol {
    name: String,
    offset: Option<u32>,
    symbol_type: SymbolType,
}

impl Symbol {
    pub fn new(name: String, symbol_type: SymbolType, offset: u32) -> Symbol {
        Symbol {
            name,
            symbol_type,
            offset: Some(offset),
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
                return symbol.offset;
            }
        }
        None
    }

    pub fn has_symbol(&self, s: &str) -> bool {
        for symbol in &self.symbols {
            // TODO: Find out if there is a way to not have to specify `return true;` in an if
            if symbol.name == s {
                return true;
            }
        }
        false
    }

    pub fn set_symbol_offset(&mut self, s: &str, offset: u32) -> bool {
        for symbol in &mut self.symbols {
            if symbol.name == s {
                symbol.offset = Some(offset);
                return true;
            }
        }
        false
    }
}

#[derive(Debug, PartialEq)]
pub enum AssemblerPhase {
    First,
    Second,
}

#[derive(Debug, PartialEq, Clone)]
pub enum AssemblerSection {
    Data { starting_instruction: Option<u32> },
    Code { starting_instruction: Option<u32> },
    Unknown,
}

impl Default for AssemblerSection {
    fn default() -> Self {
        AssemblerSection::Unknown
    }
}

impl<'a> From<&'a str> for AssemblerSection {
    fn from(name: &str) -> AssemblerSection {
        match name {
            "data" => AssemblerSection::Data {
                starting_instruction: None,
            },
            "code" => AssemblerSection::Code {
                starting_instruction: None,
            },
            _ => AssemblerSection::Unknown,
        }
    }
}

#[derive(Debug)]
pub struct Assembler {
    /// Tracks which phase the assember is in
    pub phase: AssemblerPhase,
    /// Symbol table for constants and variables
    pub symbols: SymbolTable,
    /// The read-only data section constants are put in
    pub ro: Vec<u8>,
    /// The compiled bytecode generated from the assembly instructions
    pub bytecode: Vec<u8>,
    /// Tracks the current offset of the read-only section
    ro_offset: u32,
    /// A list of all the sections we've seen in the code
    sections: Vec<AssemblerSection>,
    /// The current section the assembler is in
    current_section: Option<AssemblerSection>,
    /// The current instruction the assembler is converting to bytecode
    current_instruction: u32,
    /// Any errors we find along the way. At the end, we'll present them to the user.
    errors: Vec<AssemblerError>,
}

impl Assembler {
    pub fn new() -> Assembler {
        Assembler {
            phase: AssemblerPhase::First,
            symbols: SymbolTable::new(),
            ro: vec![],
            bytecode: vec![],
            current_instruction: 0,
            current_section: None,
            sections: vec![],
            errors: vec![],
            ro_offset: 0,
        }
    }

    fn write_pie_header(&self) -> Vec<u8> {
        let mut header = vec![];
        for byte in PIE_HEADER_PREFIX.into_iter() {
            header.push(byte.clone());
        }
        while header.len() < PIE_HEADER_LENGTH {
            header.push(0 as u8);
        }
        header
    }

    pub fn assemble(&mut self, raw: &str) -> Result<Vec<u8>, Vec<AssemblerError>> {
        // Runs the raw program through our `nom` parser
        match program(CompleteStr(raw)) {
            // If there were no parsing errors, we now have a `Vec<AssemblyInstructions>` to process.
            // `remainder` _should_ be "".
            // TODO: Add a check for `remainder`, make sure it is "".
            Ok((_remainder, program)) => {
                println!("{:?}", program);
                // Start processing the AssembledInstructions. This is the first pass of our two-pass assembler.
                // We pass a read-only reference down to another function.
                self.process_first_phase(&program);

                // If we accumulated any errors in the first pass, return them and don't try to do the second pass
                if !self.errors.is_empty() {
                    // TODO: Can we avoid a clone here?
                    return Err(self.errors.clone());
                };

                // Make sure that we have at least one data section and one code section
                if self.sections.len() != 2 {
                    // TODO: Detail out which one(s) are missing
                    println!("Did not find at least two sections.");
                    self.errors.push(AssemblerError::InsufficientSections);
                    // TODO: Can we avoid a clone here?
                    return Err(self.errors.clone());
                }
                // Run the second pass, which translates opcodes and associated operands into the bytecode
                let mut body = self.process_second_phase(&program);

                // Get the header
                let mut assembled_program = self.write_pie_header();
                // Merge the header with the populated body vector
                assembled_program.append(&mut body);
                Ok(assembled_program)
            }
            // If there were parsing errors, bad syntax, etc, this arm is run
            Err(e) => {
                println!("There was an error parsing the code: {:?}", e);
                Err(vec![AssemblerError::ParseError {
                    error: e.to_string(),
                }])
            }
        }
    }

    // Extract all labels, build symbol table
    fn process_first_phase(&mut self, p: &Program) {
        // Iterate over every instruction, even though in the first phase we care about labels and directives but nothing else
        for i in &p.instructions {
            if i.is_label() {
                // TODO: Factor this out into another function? Put it in `process_label_declaration`?
                if self.current_section.is_some() {
                    // If we have hit a segment header already (e.g., `.code`) then we are ok
                    self.process_label_declaration(&i);
                } else {
                    // If we have *not* hit a segment header yet, then we have a label outside of a segment, which is not allowed
                    self.errors.push(AssemblerError::NoSegmentDeclarationFound {
                        instruction: self.current_instruction,
                    });
                }
            }

            if i.is_directive() {
                self.process_directive(i);
            }
            // This is used to keep track of which instruction we hit an error on
            // TODO: Do we really need to track this?
            self.current_instruction += 1;
        }
        // Once we're done with this function, set the phase to second
        self.phase = AssemblerPhase::Second;
    }

    /// Handles the declaration of a label such as:
    /// hello: .asciiz 'Hello'
    fn process_label_declaration(&mut self, i: &AssemblerInstruction) {
        // Check if the label is None or String
        println!("{:?}", i.label_name());
        let name = match i.label_name() {
            Some(name) => name,
            None => {
                self.errors
                    .push(AssemblerError::StringConstantDeclaredWithoutLabel {
                        instruction: self.current_instruction,
                    });
                return;
            }
        };

        // Check if label is already in use (has an entry in the symbol table)
        // TODO: Is there a cleaner way to do this?
        if self.symbols.has_symbol(&name) {
            self.errors.push(AssemblerError::SymbolAlreadyDeclared);
            return;
        }

        // If we make it here, it isn't a symbol we've seen before, so stick it in the table
        let symbol = Symbol::new(name, SymbolType::Label, (self.current_instruction * 4) + 60);
        self.symbols.add_symbol(symbol);
    }

    fn process_directive(&mut self, i: &AssemblerInstruction) {
        // First let’s make sure we have a parseable nae
        let directive_name = match i.directive_name() {
            Some(name) => name,
            None => {
                println!("Directive has an invalid name: {:?}", i);
                return;
            }
        };

        // Now check if there were any operands.
        if i.has_operands() {
            // If it _does_ have operands, we need to figure out which directive it was
            match directive_name.as_ref() {
                // If this is the operand, we're declaring a null terminated string
                "asciiz" => {
                    self.handle_asciiz(i);
                }
                _ => {
                    self.errors.push(AssemblerError::UnknownDirectiveFound {
                        directive: directive_name.clone(),
                    });
                    return;
                }
            }
        } else {
            // If there were not any operands, (e.g., `.code`), then we know it is a section header
            self.process_section_header(&directive_name);
        }
    }

    /// Handles a declaration of a section header, such as:
    /// .code
    fn process_section_header(&mut self, header_name: &str) {
        let new_section: AssemblerSection = header_name.into();
        // Only specific section names are allowed
        if new_section == AssemblerSection::Unknown {
            println!(
                "Found an section header that is unknown: {:#?}",
                header_name
            );
            return;
        }
        // TODO: Check if we really need to keep a list of all sections seen
        self.sections.push(new_section.clone());
        self.current_section = Some(new_section);
    }

    /// Handles a declaration of a null-terminated string:
    /// hello: .asciiz 'Hello!'
    fn handle_asciiz(&mut self, i: &AssemblerInstruction) {
        // Being a constant declaration, this is only meaningful in the first pass
        if self.phase != AssemblerPhase::First {
            return;
        }

        // In this case, operand1 will have the entire string we need to read in to RO memory
        match i.get_string_constant() {
            Some(s) => {
                match i.label_name() {
                    Some(name) => {
                        self.symbols.set_symbol_offset(&name, self.ro_offset);
                    }
                    None => {
                        // This would be someone typing:
                        // .asciiz 'Hello'
                        println!("Found a string constant with no associated label!");
                        return;
                    }
                };
                // We'll read the string into the read-only section byte-by-byte
                for byte in s.as_bytes() {
                    self.ro.push(*byte);
                    self.ro_offset += 1;
                }
                // This is the null termination bit we are using to indicate a string has ended
                self.ro.push(0);
                self.ro_offset += 1;
            }
            None => {
                // This just means someone typed `.asciiz` for some reason
                println!("String constant following an .asciiz was empty");
            }
        }
    }

    // Build program(byte code)
    fn process_second_phase(&mut self, p: &Program) -> Vec<u8> {
        self.current_instruction = 0;
        let mut program = vec![];
        for i in &p.instructions {
            if i.is_opcode() {
                let mut bytes = i.to_bytes(&self.symbols);
                program.append(&mut bytes);
            }
            if i.is_directive() {
                // In this phase, we can have directives but of different types than we care about in the first pass. The Directive itself can check which pass the Assembler
                // is in and decide what to do about it
                self.process_directive(i);
            }
            self.current_instruction += 1
        }
        program
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
        let test_string = r"
            .data
            .code
            load $0 #100
            load $1 #1
            load $2 #0
            test: inc $0
            neq $0 $2
            ";
        let program = asm.assemble(test_string).unwrap();
        let mut vm = VM::new();
        assert_eq!(program.len(), 17 + PIE_HEADER_LENGTH);
        vm.add_bytes(program);
        assert_eq!(vm.program.len(), 17 + PIE_HEADER_LENGTH);
    }

    #[test]
    fn test_write_pie_header() {
        let mut asm = Assembler::new();
        let header = asm.write_pie_header();
        assert_eq!(header.len(), PIE_HEADER_LENGTH);
    }
}
