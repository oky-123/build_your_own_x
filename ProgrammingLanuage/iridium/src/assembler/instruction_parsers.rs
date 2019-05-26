use nom::types::CompleteStr;

use byteorder::{ByteOrder, LittleEndian};

use crate::assembler::directive_parsers::directive_combined;
use crate::assembler::label_parsers::label_declaration;
use crate::assembler::opcode_parsers::*;
use crate::assembler::operand_parsers::operand;
use crate::assembler::{SymbolTable, Token};

#[derive(Debug, PartialEq)]
pub struct AssemblerInstruction {
    pub opcode: Option<Token>,
    pub operand1: Option<Token>,
    pub operand2: Option<Token>,
    pub operand3: Option<Token>,
    pub label: Option<Token>,
    pub directive: Option<Token>,
}

impl AssemblerInstruction {
    pub fn to_bytes(&self, symbols: &SymbolTable) -> Vec<u8> {
        let mut results = vec![];
        if let Some(Token::Op { code }) = self.opcode {
            results.push(code as u8);
        } else {
            println!("Non-opcode found in opcode field");
        }

        for operand in vec![&self.operand1, &self.operand2, &self.operand3] {
            if let Some(token) = operand {
                AssemblerInstruction::extract_operand(token, &mut results, symbols)
            }
        }

        results
    }

    fn extract_operand(t: &Token, results: &mut Vec<u8>, symbols: &SymbolTable) {
        match t {
            Token::Register { reg_num } => {
                results.push(*reg_num);
            }
            Token::IntegerOperand { value } => {
                let converted = *value as u16;
                let byte1 = converted;
                let byte2 = converted >> 8;
                results.push(byte2 as u8);
                results.push(byte1 as u8);
            }
            Token::LabelUsage { name } => {
                if let Some(value) = symbols.symbol_value(name) {
                    let converted = value as u16;
                    let byte1 = converted;
                    let byte2 = converted >> 8;
                    results.push(byte2 as u8);
                    results.push(byte1 as u8);
                } else {
                    println!("No value found for {:?}", name);
                }
            }
            _ => {
                println!("Opcode found in operand field");
                std::process::exit(1);
            }
        };
    }

    pub fn is_label(&self) -> bool {
        return self.label.is_some();
    }

    pub fn label_name(&self) -> Option<String> {
        return match &self.label {
            Some(Token::LabelDeclaration { name }) => Some(name.to_string()),
            _ => None,
        };
    }

    pub fn is_directive(&self) -> bool {
        return self.directive.is_some();
    }

    pub fn directive_name(&self) -> Option<String> {
        if let Some(Token::Directive { name }) = &self.directive {
            return Some(name.to_string());
        }
        None
    }

    pub fn is_opcode(&self) -> bool {
        return self.opcode.is_some();
    }

    pub fn has_operands(&self) -> bool {
        return self.operand1.is_some();
    }

    pub fn get_string_constant(&self) -> Option<String> {
        match &self.operand1 {
            Some(d) => match d {
                Token::IrString { name } => Some(name.to_string()),
                _ => None,
            },
            None => None,
        }
    }
}

named!(pub instruction<CompleteStr, AssemblerInstruction>,
   do_parse!(
       ins: alt!(
           instruction_combined |
           directive_combined
       ) >> ( ins )
   )
);

named!(pub instruction_combined<CompleteStr, AssemblerInstruction>,
    do_parse!(
        l: opt!(label_declaration) >>
        o: opcode_load >>
        o1: opt!(operand) >>
        o2: opt!(operand) >>
        o3: opt!(operand) >>
        (
            {
                AssemblerInstruction{
                    opcode: Some(o),
                    label: l,
                    directive: None,
                    operand1: o1,
                    operand2: o2,
                    operand3: o3,
                }
            }
        )
    )
);

#[cfg(test)]
mod tests {

    use super::super::*;
    use super::*;

    #[test]
    fn test_parse_instruction_form_one() {
        let result = instruction(CompleteStr("load $0 #100\n"));
        assert_eq!(
            result,
            Ok((
                CompleteStr(""),
                AssemblerInstruction {
                    opcode: Some(Token::Op { code: Opcode::LOAD }),
                    operand1: Some(Token::Register { reg_num: 0 }),
                    operand2: Some(Token::IntegerOperand { value: 100 }),
                    operand3: None,
                    directive: None,
                    label: None,
                }
            ))
        );
    }

    #[test]
    fn test_parse_instruction_form_two() {
        let result = instruction(CompleteStr("HLT"));
        assert_eq!(
            result,
            Ok((
                CompleteStr(""),
                AssemblerInstruction {
                    opcode: Some(Token::Op { code: Opcode::HLT }),
                    operand1: None,
                    operand2: None,
                    operand3: None,
                    directive: None,
                    label: None,
                }
            ))
        );
    }

    #[test]
    fn test_parse_instruction_form_three() {
        let result = instruction(CompleteStr("add $1 $2 $3"));
        assert_eq!(
            result,
            Ok((
                CompleteStr(""),
                AssemblerInstruction {
                    opcode: Some(Token::Op { code: Opcode::ADD }),
                    operand1: Some(Token::Register { reg_num: 1 }),
                    operand2: Some(Token::Register { reg_num: 2 }),
                    operand3: Some(Token::Register { reg_num: 3 }),
                    directive: None,
                    label: None,
                }
            ))
        );
    }

    #[test]
    fn test_parse_instruction() {
        let result = instruction(CompleteStr("hlt"));
        assert_eq!(
            result,
            Ok((
                CompleteStr(""),
                AssemblerInstruction {
                    opcode: Some(Token::Op { code: Opcode::HLT }),
                    operand1: None,
                    operand2: None,
                    operand3: None,
                    directive: None,
                    label: None,
                }
            ))
        );

        let result = instruction(CompleteStr("load $1 #100"));
        assert_eq!(
            result,
            Ok((
                CompleteStr(""),
                AssemblerInstruction {
                    opcode: Some(Token::Op { code: Opcode::LOAD }),
                    operand1: Some(Token::Register { reg_num: 1 }),
                    operand2: Some(Token::IntegerOperand { value: 100 }),
                    operand3: None,
                    directive: None,
                    label: None,
                }
            ))
        );

        let result = instruction(CompleteStr("add $1 $2 $3"));
        assert_eq!(
            result,
            Ok((
                CompleteStr(""),
                AssemblerInstruction {
                    opcode: Some(Token::Op { code: Opcode::ADD }),
                    operand1: Some(Token::Register { reg_num: 1 }),
                    operand2: Some(Token::Register { reg_num: 2 }),
                    operand3: Some(Token::Register { reg_num: 3 }),
                    directive: None,
                    label: None,
                }
            ))
        );
    }

    #[test]
    fn test_parse_instruction_combined() {
        let result = instruction_combined(CompleteStr("label: load $1 $2 $3"));
        assert_eq!(
            result,
            Ok((
                CompleteStr(""),
                AssemblerInstruction {
                    opcode: Some(Token::Op { code: Opcode::LOAD }),
                    operand1: Some(Token::Register { reg_num: 1 }),
                    operand2: Some(Token::Register { reg_num: 2 }),
                    operand3: Some(Token::Register { reg_num: 3 }),
                    directive: None,
                    label: Some(Token::LabelDeclaration {
                        name: "label".to_string()
                    }),
                }
            ))
        );

        let result = instruction_combined(CompleteStr("jmp @label"));
        assert_eq!(
            result,
            Ok((
                CompleteStr(""),
                AssemblerInstruction {
                    opcode: Some(Token::Op { code: Opcode::JMP }),
                    operand1: Some(Token::LabelUsage {
                        name: "label".to_string()
                    }),
                    operand2: None,
                    operand3: None,
                    directive: None,
                    label: None,
                }
            ))
        );

        let result = instruction(CompleteStr(".directive"));
        assert_eq!(
            result,
            Ok((
                CompleteStr(""),
                AssemblerInstruction {
                    opcode: None,
                    operand1: None,
                    operand2: None,
                    operand3: None,
                    directive: Some(Token::Directive {
                        name: "directive".to_string()
                    }),
                    label: None,
                }
            ))
        );

        if let Ok((_, instruction)) = result {
            assert_eq!(instruction.directive_name(), Some("directive".to_string()));
        }

        let result = instruction(CompleteStr("label: .asciiz 'string'"));
        assert_eq!(
            result,
            Ok((
                CompleteStr(""),
                AssemblerInstruction {
                    opcode: None,
                    operand1: Some(Token::IrString {
                        name: "string".to_string()
                    }),
                    operand2: None,
                    operand3: None,
                    directive: Some(Token::Directive {
                        name: "asciiz".to_string()
                    }),
                    label: Some(Token::LabelDeclaration {
                        name: "label".to_string()
                    }),
                }
            ))
        );

        if let Ok((_, instruction)) = result {
            assert_eq!(instruction.label_name(), Some("label".to_string()));
        }
    }
}
