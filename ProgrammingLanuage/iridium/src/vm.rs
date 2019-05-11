use super::instruction::*;

#[derive(Debug)]
pub struct VM {
    registers: [i32; 32],
    pc: usize,
    program: Vec<u8>,
}

impl VM {
    pub fn new() -> VM {
        VM {
            registers: [0; 32],
            program: vec![],
            pc: 0,
        }
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
        loop {
            if self.pc >= self.program.len() {
                break;
            }
            match self.decode_opcode() {
                Opcode::HLT => {
                    println!("HLT encountered");
                    return;
                }
                Opcode::LOAD => {
                    let register = self.next_8_bits() as usize; // We cast to usize so we can use it as an index into the array
                    let number = self.next_16_bits() as u16;
                    self.registers[register] = number as i32; // Our registers are i32s, so we need to cast it. We'll cover that later.
                    continue; // Start another iteration of the loop. The next 8 bits waiting to be read should be an opcode.
                }
                _ => {
                    println!("Unrecognized opcode found! Terminating!");
                    return;
                }
            }
        }
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
    fn test_load_opcode() {
        let mut test_vm = VM::new();
        test_vm.program = vec![1, 0, 1, 244]; // Remember, this is how we represent 500 using two u8s in little endian format
                                              // [0, 0, 0, 0, 0, 0, 0, 1] [1, 1, 1, 1, 1, 0, 1, 0],
                                              // 500 - 244 = 256
        test_vm.run();
        assert_eq!(test_vm.registers[0], 500);
    }
}
