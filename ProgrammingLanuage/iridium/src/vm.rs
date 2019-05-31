use super::instruction::*;
use crate::assembler::{PIE_HEADER_LENGTH, PIE_HEADER_PREFIX};
use std::num::ParseIntError;

#[derive(Debug, Clone)]
pub struct VM {
    pub registers: [i32; 32],
    pub pc: usize,        // pointer-sized: u64
    pub program: Vec<u8>, // u8 <= 256
    remainder: u32,
    equal_flag: bool,
    heap: Vec<u8>,
    pub ro_data: Vec<u8>,
}

pub fn prepend_header(mut b: Vec<u8>) -> Vec<u8> {
    let mut prepension = vec![];
    for byte in PIE_HEADER_PREFIX.into_iter() {
        prepension.push(byte.clone());
    }
    while prepension.len() < PIE_HEADER_LENGTH {
        prepension.push(0);
    }
    prepension.append(&mut b);
    prepension
}

impl VM {
    pub fn new() -> VM {
        VM {
            registers: [0; 32],
            program: vec![],
            pc: 0,
            remainder: 0,
            equal_flag: false,
            heap: vec![],
            ro_data: vec![],
        }
    }

    pub fn verify_header(&self) -> bool {
        if self.program[0..4] != PIE_HEADER_PREFIX {
            return false;
        }
        true
    }

    pub fn init_registers(&mut self, vec: [i32; 32]) {
        self.registers = vec;
    }

    fn next_8_bits(&mut self) -> u8 {
        let result = self.program[self.pc];
        self.pc += 1;
        return result;
    }

    fn next_16_bits(&mut self) -> u16 {
        let result = ((self.program[self.pc] as u16) << 8) | self.program[self.pc + 1] as u16;
        self.pc += 2;
        return result;
    }

    fn decode_opcode(&mut self) -> Opcode {
        let opcode = Opcode::from(self.program[self.pc]);
        self.pc += 1;
        return opcode;
    }

    pub fn run(&mut self) -> u32 {
        if !self.verify_header() {
            println!("Header was incorrect");
            1
        } else {
            self.pc = 64;
            let mut is_done = false;
            while !is_done {
                is_done = self.execute_instruction();
            }
            0
        }
    }

    pub fn run_once(&mut self) {
        self.execute_instruction();
    }

    pub fn add_hexes(&mut self, i: &str) {
        let result = self.parse_hex(i);
        match result {
            Err(e) => {
                println!("{}", e);
            }
            Ok(bytes) => {
                self.add_bytes(bytes);
            }
        }
    }

    fn parse_hex(&mut self, i: &str) -> Result<Vec<u8>, ParseIntError> {
        let split = i.split(" ").collect::<Vec<&str>>();

        let mut results: Vec<u8> = vec![];
        for hex_string in split {
            let byte = u8::from_str_radix(&hex_string, 16);
            match byte {
                Ok(result) => {
                    results.push(result);
                }
                Err(e) => {
                    return Err(e);
                }
            }
        }
        Ok(results)
    }

    pub fn add_bytes(&mut self, mut b: Vec<u8>) {
        self.program.append(&mut b);
    }

    fn execute_instruction(&mut self) -> bool {
        if self.pc >= self.program.len() {
            return true;
        }
        match self.decode_opcode() {
            Opcode::LOAD => {
                let register = self.next_8_bits() as usize;
                let number = self.next_16_bits() as u32;
                self.registers[register] = number as i32;
            }
            Opcode::ADD => {
                let register1 = self.registers[self.next_8_bits() as usize];
                let register2 = self.registers[self.next_8_bits() as usize];
                self.registers[self.next_8_bits() as usize] = register1 + register2;
            }
            Opcode::SUB => {
                let register1 = self.registers[self.next_8_bits() as usize];
                let register2 = self.registers[self.next_8_bits() as usize];
                self.registers[self.next_8_bits() as usize] = register1 - register2;
            }
            Opcode::MUL => {
                let register1 = self.registers[self.next_8_bits() as usize];
                let register2 = self.registers[self.next_8_bits() as usize];
                self.registers[self.next_8_bits() as usize] = register1 * register2;
            }
            Opcode::DIV => {
                let register1 = self.registers[self.next_8_bits() as usize];
                let register2 = self.registers[self.next_8_bits() as usize];
                self.registers[self.next_8_bits() as usize] = register1 / register2;
                self.remainder = (register1 % register2) as u32;
            }
            Opcode::JMP => {
                let target = self.registers[self.next_8_bits() as usize];
                self.pc = target as usize;
            }
            Opcode::JMPF => {
                let value = self.registers[self.next_8_bits() as usize];
                self.pc += value as usize;
            }
            Opcode::JMPB => {
                let value = self.registers[self.next_8_bits() as usize];
                self.pc -= value as usize;
            }
            Opcode::EQ => {
                let register1 = self.registers[self.next_8_bits() as usize];
                let register2 = self.registers[self.next_8_bits() as usize];
                if register1 == register2 {
                    self.equal_flag = true;
                } else {
                    self.equal_flag = false;
                }
                self.next_8_bits();
            }
            Opcode::NEQ => {
                let register1 = self.registers[self.next_8_bits() as usize];
                let register2 = self.registers[self.next_8_bits() as usize];
                if register1 == register2 {
                    self.equal_flag = false;
                } else {
                    self.equal_flag = true;
                }
                self.next_8_bits();
            }
            Opcode::GT => {
                let register1 = self.registers[self.next_8_bits() as usize];
                let register2 = self.registers[self.next_8_bits() as usize];
                if register1 > register2 {
                    self.equal_flag = true;
                } else {
                    self.equal_flag = false;
                }
                self.next_8_bits();
            }
            Opcode::LT => {
                let register1 = self.registers[self.next_8_bits() as usize];
                let register2 = self.registers[self.next_8_bits() as usize];
                if register1 < register2 {
                    self.equal_flag = true;
                } else {
                    self.equal_flag = false;
                }
                self.next_8_bits();
            }
            Opcode::GTQ => {
                let register1 = self.registers[self.next_8_bits() as usize];
                let register2 = self.registers[self.next_8_bits() as usize];
                if register1 >= register2 {
                    self.equal_flag = true;
                } else {
                    self.equal_flag = false;
                }
                self.next_8_bits();
            }
            Opcode::LTQ => {
                let register1 = self.registers[self.next_8_bits() as usize];
                let register2 = self.registers[self.next_8_bits() as usize];
                if register1 <= register2 {
                    self.equal_flag = true;
                } else {
                    self.equal_flag = false;
                }
                self.next_8_bits();
            }
            Opcode::JEQ => {
                let value = self.registers[self.next_8_bits() as usize];
                if self.equal_flag {
                    self.pc = value as usize;
                }
            }
            Opcode::ALOC => {
                let bytes = self.registers[self.next_8_bits() as usize];
                let new_end = self.heap.len() as i32 + bytes;
                self.heap.resize(new_end as usize, 0);
            }
            Opcode::INC => {
                let register = self.next_8_bits() as usize;
                self.registers[register] += 1;
            }
            Opcode::DEC => {
                let register = self.next_8_bits() as usize;
                self.registers[register] -= 1;
            }
            Opcode::PRTS => {
                let starting_offset = self.next_16_bits() as usize;
                let mut ending_offset = starting_offset;
                let slice = self.ro_data.as_slice();
                while slice[ending_offset] != 0 {
                    ending_offset += 1;
                }
                let result = std::str::from_utf8(&slice[starting_offset..ending_offset]);
                match result {
                    Ok(s) => {
                        print!("{}\n", s);
                    }
                    Err(e) => println!("Error decoding string for prts instruction: {:#?}", e),
                };
            }
            Opcode::HLT => {
                println!("HLT encountered");
                return true;
            }
            Opcode::IGL => {
                println!("IGL encountered");
                return true;
            }
        }
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_vm() {
        let test_vm = VM::new();
        assert_eq!(test_vm.registers[0], 0)
    }

    #[test]
    fn test_opcode_hlt() {
        let mut test_vm = VM::new();
        let test_bytes = vec![0, 0, 0, 0];
        test_vm.program = test_bytes;
        test_vm.program = prepend_header(test_vm.program);
        test_vm.run();
        assert_eq!(test_vm.pc, 1 + PIE_HEADER_LENGTH);
    }

    #[test]
    fn test_opcode_igl() {
        let mut test_vm = VM::new();
        let test_bytes = vec![200, 0, 0, 0];
        test_vm.program = test_bytes;
        test_vm.program = prepend_header(test_vm.program);
        test_vm.run();
        assert_eq!(test_vm.pc, 1 + PIE_HEADER_LENGTH);
    }

    #[test]
    fn test_init_registers() {
        let mut test_vm = VM::new();
        test_vm.init_registers([10; 32]);

        assert_eq!(test_vm.registers[0], 10);
        assert_eq!(test_vm.registers[31], 10);
    }

    #[test]
    fn test_sub_opcode() {
        let mut test_vm = VM::new();
        let mut array = [0; 32];
        for i in 0..32 {
            array[i] = i as i32;
        }
        test_vm.init_registers(array);
        test_vm.program = vec![3, 3, 1, 4]; // 3 - 1 = 2
        test_vm.program = prepend_header(test_vm.program);
        test_vm.run();

        assert_eq!(test_vm.registers[4], 2);
    }

    #[test]
    fn test_mul_opcode() {
        let mut test_vm = VM::new();
        let mut array = [0; 32];
        for i in 0..32 {
            array[i] = i as i32;
        }
        test_vm.init_registers(array);
        test_vm.program = vec![4, 3, 4, 5]; // 3 * 4 = 12
        test_vm.program = prepend_header(test_vm.program);
        test_vm.run();

        assert_eq!(test_vm.registers[5], 12);
    }

    #[test]
    fn test_div_opcode() {
        let mut test_vm = VM::new();
        let mut array = [0; 32];
        for i in 0..32 {
            array[i] = i as i32;
        }
        test_vm.init_registers(array);
        test_vm.program = vec![5, 3, 2, 3]; // 3 / 2 = 1 remainder 1
        test_vm.program = prepend_header(test_vm.program);
        assert_eq!(test_vm.registers[3], 3);
        assert_eq!(test_vm.registers[2], 2);
        test_vm.run();

        assert_eq!(test_vm.registers[3], 1);
        assert_eq!(test_vm.remainder, 1)
    }

    #[test]
    fn test_jmp_opcode() {
        let mut test_vm = VM::new();
        test_vm.registers[0] = 1;
        test_vm.program = vec![6, 0, 0, 0];
        test_vm.run_once();

        assert_eq!(test_vm.pc, 1);
    }

    #[test]
    fn test_jmpf_opcode() {
        let mut test_vm = VM::new();
        test_vm.registers[0] = 2;
        test_vm.program = vec![7, 0, 0, 0];
        test_vm.run_once();

        assert_eq!(test_vm.pc, 4);
    }

    #[test]
    fn test_jmpb_opcode() {
        let mut test_vm = VM::new();
        test_vm.registers[0] = 2;
        test_vm.program = vec![8, 0, 0, 0];
        test_vm.run_once();

        assert_eq!(test_vm.pc, 0);
    }

    #[test]
    fn test_eq_opcode() {
        let mut test_vm = VM::new();
        test_vm.program = vec![9, 0, 0, 0];
        test_vm.run_once();

        assert_eq!(test_vm.equal_flag, true);
        assert_eq!(test_vm.pc, 4);
    }

    #[test]
    fn test_neq_opcode() {
        let mut test_vm = VM::new();
        test_vm.program = vec![10, 0, 0, 0];
        test_vm.run_once();

        assert_eq!(test_vm.equal_flag, false);
        assert_eq!(test_vm.pc, 4);
    }

    #[test]
    fn test_gt_opcode() {
        let mut test_vm = VM::new();
        test_vm.program = vec![11, 0, 0, 0];
        test_vm.run_once();

        assert_eq!(test_vm.equal_flag, false);
        assert_eq!(test_vm.pc, 4);
    }

    #[test]
    fn test_lt_opcode() {
        let mut test_vm = VM::new();
        test_vm.program = vec![12, 0, 0, 0];
        test_vm.run_once();

        assert_eq!(test_vm.equal_flag, false);
        assert_eq!(test_vm.pc, 4);
    }

    #[test]
    fn test_gtq_opcode() {
        let mut test_vm = VM::new();
        test_vm.program = vec![13, 0, 0, 0];
        test_vm.run_once();

        assert_eq!(test_vm.equal_flag, true);
        assert_eq!(test_vm.pc, 4);
    }

    #[test]
    fn test_ltq_opcode() {
        let mut test_vm = VM::new();
        test_vm.program = vec![14, 0, 0, 0];
        test_vm.run_once();

        assert_eq!(test_vm.equal_flag, true);
        assert_eq!(test_vm.pc, 4);
    }

    #[test]
    fn test_jeq_opcode() {
        let mut test_vm = VM::new();
        test_vm.program = vec![9, 0, 0, 0, 15, 0];
        test_vm.run_once();
        test_vm.run_once();

        assert_eq!(test_vm.equal_flag, true);
        assert_eq!(test_vm.pc, 0);
    }

    #[test]
    fn test_inc_opcode() {
        let mut test_vm = VM::new();
        test_vm.registers[0] = 1024;
        test_vm.program = vec![17, 0];
        test_vm.run_once();
        assert_eq!(test_vm.registers[0], 1025);
    }

    #[test]
    fn test_dec_opcode() {
        let mut test_vm = VM::new();
        test_vm.registers[0] = 1024;
        test_vm.program = vec![18, 0];
        test_vm.run_once();
        assert_eq!(test_vm.registers[0], 1023);
    }

    #[test]
    fn test_aloc_opcode() {
        let mut test_vm = VM::new();
        test_vm.registers[0] = 1024;
        test_vm.program = vec![16, 0, 0, 0];
        test_vm.run_once();
        assert_eq!(test_vm.heap.len(), 1024);
    }
}
