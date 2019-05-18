use nom::types::CompleteStr;

use crate::assembler::integer_parsers::integer;
use crate::assembler::label_parsers::label_usage;
use crate::assembler::register_parsers::register;
use crate::assembler::Token;

named!(pub operand<CompleteStr, Token>,
    alt!(
        register |
        integer |
        label_usage
    )
);

mod tests {
    #[allow(unused_imports)]
    use super::*;

    #[test]
    fn test_operand() {
        let result = operand(CompleteStr("#10"));
        assert_eq!(result.is_ok(), true);
        let (_, value) = result.unwrap();
        assert_eq!(value, Token::IntegerOperand { value: 10 });

        // Test an invalid one (missing the #)
        let result = operand(CompleteStr("$10"));
        let (_, value) = result.unwrap();
        assert_eq!(value, Token::Register { reg_num: 10 });
    }
}
