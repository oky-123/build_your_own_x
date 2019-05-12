use super::instruction::*;

#[derive(Debug)]
pub struct VM {
    registers: [i32; 32],
    pc: usize,
    program: Vec<u8>,
    remainder: u32,
}

impl VM {
    pub fn new() -> VM {
        VM {
            registers: [0; 32],
            program: vec![],
            pc: 0,
            remainder: 0,
        }
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

    pub fn run(&mut self) {
        let mut is_done = false;
        while !is_done {
            is_done = self.execute_instruction();
            println!("{}", is_done);
        }
    }

    pub fn run_once(&mut self) {
        self.execute_instruction();
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
            Opcode::DIV => {
                let register1 = self.registers[self.next_8_bits() as usize];
                let register2 = self.registers[self.next_8_bits() as usize];
                self.registers[self.next_8_bits() as usize] = register1 / register2;
                self.remainder = (register1 % register2) as u32;
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
        test_vm.run();
        assert_eq!(test_vm.pc, 1);
    }

    #[test]
    fn test_opcode_igl() {
        let mut test_vm = VM::new();
        let test_bytes = vec![200, 0, 0, 0];
        test_vm.program = test_bytes;
        test_vm.run();
        assert_eq!(test_vm.pc, 1);
    }

    #[test]
    fn test_load_and_add_opcode() {
        let mut test_vm = VM::new();
        test_vm.program = vec![1, 0, 1, 244, 2, 0, 1, 1, 0]; // 1, 244 = [0, 0, 0, 0, 0, 0, 0, 1], [1, 1, 1, 1, 1, 0, 1, 0],
                                                             // 500 - 244 = 256
        test_vm.run();
        assert_eq!(test_vm.registers[0], 500);
        assert_eq!(test_vm.registers[1], 500);
        assert_eq!(test_vm.pc, 9);
    }

    #[test]
    fn test_init_registers() {
        let mut test_vm = VM::new();
        test_vm.init_registers([10; 32]);

        assert_eq!(test_vm.registers[0], 10);
        assert_eq!(test_vm.registers[31], 10);
    }

    #[test]
    fn test_div_opcode() {
        let mut test_vm = VM::new();
        let mut array = [0; 32];
        for i in 0..32 {
            array[i] = i as i32;
        }
        test_vm.init_registers(array);
        test_vm.program = vec![3, 3, 2, 3]; // 3 / 2 = 1 remainder 1
        assert_eq!(test_vm.registers[3], 3);
        assert_eq!(test_vm.registers[2], 2);
        test_vm.run();

        assert_eq!(test_vm.registers[3], 1);
        assert_eq!(test_vm.remainder, 1)
    }
}