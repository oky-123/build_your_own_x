use nom::multispace;
use nom::types::CompleteStr;

use crate::assembler::directive_parsers::directive;
use crate::assembler::integer_parsers::integer;
use crate::assembler::label_parsers::label_declaration;
use crate::assembler::opcode_parsers::*;
use crate::assembler::operand_parsers::operand;
use crate::assembler::register_parsers::register;
use crate::assembler::Token;

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
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut results = vec![];
        if let Some(Token::Op { code }) = self.opcode {
            results.push(code as u8);
        } else {
            println!("Non-opcode found in opcode field");
            std::process::exit(1);
        }

        for operand in vec![&self.operand1, &self.operand2, &self.operand3] {
            if let Some(token) = operand {
                AssemblerInstruction::extract_operand(token, &mut results)
            }
        }

        results
    }

    fn extract_operand(t: &Token, results: &mut Vec<u8>) {
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
            _ => {
                println!("Opcode found in operand field");
                std::process::exit(1);
            }
        };
    }
}

named!(pub instruction_one<CompleteStr, AssemblerInstruction>,
    do_parse!(
        o: opcode_load >>
        r: register >>
        i: integer >>
        (
            AssemblerInstruction{
                opcode: Some(o),
                operand1: Some(r),
                operand2: Some(i),
                operand3: None,
                directive: None,
                label: None,
            }
        )
    )
);

named!(pub instruction_two<CompleteStr, AssemblerInstruction>,
    do_parse!(
        o: opcode_load >>
        opt!(multispace) >>
        (
            AssemblerInstruction{
                opcode: Some(o),
                operand1: None,
                operand2: None,
                operand3: None,
                directive: None,
                label: None,
            }
        )
    )
);

named!(pub instruction_three<CompleteStr, AssemblerInstruction>,
    do_parse!(
        o: opcode_load >>
        r1: register >>
        r2: register >>
        r3: register >>
        (
            AssemblerInstruction{
                opcode: Some(o),
                operand1: Some(r1),
                operand2: Some(r2),
                operand3: Some(r3),
                directive: None,
                label: None,
            }
        )
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
            AssemblerInstruction{
                opcode: Some(o),
                operand1: o1,
                operand2: o2,
                operand3: o3,
                directive: None,
                label: l,
            }
        )
    )
);

named!(pub instruction<CompleteStr, AssemblerInstruction>,
   do_parse!(
       ins: alt!(
           instruction_three |
           instruction_one |
           instruction_two
       ) >> ( ins )
   )
);

named!(pub instruction_with_directive<CompleteStr, AssemblerInstruction>,
    do_parse!(
        ins: alt!(
            instruction |
            directive
        ) >>
        (
            ins
        )
    )
);

#[cfg(test)]
mod tests {

    use super::super::*;
    use super::*;

    #[test]
    fn test_parse_instruction_form_one() {
        let result = instruction_one(CompleteStr("load $0 #100\n"));
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
        let result = instruction_two(CompleteStr("HLT"));
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
        let result = instruction_three(CompleteStr("add $1 $2 $3"));
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
}
