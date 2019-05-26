use nom::alpha1;
use nom::types::CompleteStr;

use crate::assembler::instruction_parsers::AssemblerInstruction;
use crate::assembler::label_parsers::label_declaration;
use crate::assembler::operand_parsers::operand;
use crate::assembler::Token;

named!(directive_declaration<CompleteStr, Token>,
  do_parse!(
      tag!(".") >>
      name: alpha1 >>
      (
        Token::Directive{name: name.to_string()}
      )
  )
);

named!(pub directive_combined<CompleteStr, AssemblerInstruction>,
    ws!(
        do_parse!(
            l: opt!(label_declaration) >>
            name: directive_declaration >>
            o1: opt!(operand) >>
            o2: opt!(operand) >>
            o3: opt!(operand) >>
            (
                AssemblerInstruction{
                    opcode: None,
                    directive: Some(name),
                    label: l,
                    operand1: o1,
                    operand2: o2,
                    operand3: o3,
                }
            )
        )
    )
);

/// Will try to parse out any of the Directive forms
named!(pub directive<CompleteStr, AssemblerInstruction>,
    do_parse!(
        ins: alt!(
            directive_combined
        ) >>
        (
            ins
        )
    )
);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_directive_declaration() {
        let result = directive_declaration(CompleteStr(".directive"));
        assert_eq!(
            result,
            Ok((
                CompleteStr(""),
                // AssemblerInstruction {
                //     opcode: None,
                //     operand1: None,
                //     operand2: None,
                //     operand3: None,
                //directive: Some(
                Token::Directive {
                    name: "directive".to_string()
                } //),
                  //    label: None,
                  //}
            ))
        );
    }

    #[test]
    fn test_parse_directive_combined() {
        let result = directive_combined(CompleteStr("label: .directive $1 $2 $3"));
        assert_eq!(
            result,
            Ok((
                CompleteStr(""),
                AssemblerInstruction {
                    opcode: None,
                    operand1: Some(Token::Register { reg_num: 1 }),
                    operand2: Some(Token::Register { reg_num: 2 }),
                    operand3: Some(Token::Register { reg_num: 3 }),
                    directive: Some(Token::Directive {
                        name: "directive".to_string()
                    }),
                    label: Some(Token::LabelDeclaration {
                        name: "label".to_string()
                    }),
                }
            ))
        );
        let result = directive_combined(CompleteStr(".directive $1"));
        assert_eq!(
            result,
            Ok((
                CompleteStr(""),
                AssemblerInstruction {
                    opcode: None,
                    operand1: Some(Token::Register { reg_num: 1 }),
                    operand2: None,
                    operand3: None,
                    directive: Some(Token::Directive {
                        name: "directive".to_string()
                    }),
                    label: None,
                }
            ))
        );
    }

    #[test]
    fn test_parse_directive() {
        let result = directive(CompleteStr(".directive #1"));
        assert_eq!(
            result,
            Ok((
                CompleteStr(""),
                AssemblerInstruction {
                    opcode: None,
                    operand1: Some(Token::IntegerOperand { value: 1 }),
                    operand2: None,
                    operand3: None,
                    directive: Some(Token::Directive {
                        name: "directive".to_string()
                    }),
                    label: None,
                }
            ))
        );
    }

    #[test]
    fn test_string_directive() {
        let result = directive_combined(CompleteStr("test: .asciiz 'Hello'"));
        assert_eq!(result.is_ok(), true);
        let (_, directive) = result.unwrap();

        // Yes, this is the what the result should be
        let correct_instruction = AssemblerInstruction {
            opcode: None,
            label: Some(Token::LabelDeclaration {
                name: "test".to_string(),
            }),
            directive: Some(Token::Directive {
                name: "asciiz".to_string(),
            }),
            operand1: Some(Token::IrString {
                name: "Hello".to_string(),
            }),
            operand2: None,
            operand3: None,
        };

        assert_eq!(directive, correct_instruction);
    }
}
