use nom::digit;
use nom::types::CompleteStr;

use crate::assembler::Token;

named!(pub integer<CompleteStr, Token>,
    ws!(
        do_parse!(
            tag!("#") >>
            reg_num: digit >>
            (
                Token::IntegerOperand{value: reg_num.parse::<i32>().unwrap()}
            )
        )
    )
);

mod tests {
    #[allow(unused_imports)]
    use super::*;

    #[test]
    fn test_parse_integer() {
        // Test a valid integer operand
        let result = integer(CompleteStr("#10"));
        assert_eq!(result.is_ok(), true);
        let (rest, value) = result.unwrap();
        assert_eq!(rest, CompleteStr(""));
        assert_eq!(value, Token::IntegerOperand { value: 10 });

        // Test an invalid one (missing the #)
        let result = integer(CompleteStr("10"));
        assert_eq!(result.is_ok(), false);
    }
}
