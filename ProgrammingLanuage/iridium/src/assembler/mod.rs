use crate::instruction::Opcode;

#[derive(Debug, PartialEq)]
pub enum Token {
    Op { code: Opcode },
}
