use super::{opcode::{OpCode, AddressRegister}, rand::Rand, display::ChipDisplay};
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug)]
pub enum EmulatorError {
    MemoryOutOfBounds,
    InvalidOpCode,
    InvalidReturn
}

impl std::fmt::Display for EmulatorError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl std::error::Error for EmulatorError {}

pub struct Emulator<D> {
    registers: [u8; 16],
    memory_address_register: AddressRegister,
    memory: [u8; 4096],
    delay_timer: u8,
    sound_timer: u8,
    program_counter: usize,
    random_generator: Rand,
    stack: Vec<usize>,
    pub display: D
}

impl<D> Emulator<D> {
    pub fn new(display: D) -> Self {
        EmulatorBuilder::new().build(display)
    }

    pub fn write_at_pc(&mut self, bytes: &[u8]) -> bool {
        self.write_memory(bytes, self.program_counter)
    }

    pub fn write_memory(&mut self, bytes: &[u8], at: usize) -> bool {
        if at + bytes.len() > self.memory.len() {
            false
        } else {
            self.memory.as_mut_slice()[at..at+bytes.len()].copy_from_slice(bytes);
            true
        }
    }
}

impl<D: ChipDisplay> Emulator<D> {
    pub fn next_opcode(&mut self) -> Result<(), EmulatorError> {
        let bytes = (
            *self.memory.get(self.program_counter).ok_or(EmulatorError::MemoryOutOfBounds)?,
            *self.memory.get(self.program_counter + 1).ok_or(EmulatorError::MemoryOutOfBounds)?
        );

        self.program_counter += 2;

        let opcode = bytes.try_into().map_err(|_| EmulatorError::InvalidOpCode)?;
        
        match opcode {
            OpCode::CallMachineCode(_) => todo!(),
            OpCode::DisplayClear => todo!(),
            OpCode::Return => {
                let return_address = self.stack.pop().ok_or(EmulatorError::InvalidReturn)?;
                self.program_counter = return_address;
            },
            OpCode::Jump(address) => self.program_counter = address as usize,
            OpCode::CallSubroutine(address) => {
                self.stack.push(self.program_counter);
                self.program_counter = address as usize;
            },
            OpCode::CompareEqConst(reg, c) => self.program_counter += (self.registers[reg as usize] == c) as usize * 2,
            OpCode::CompareNotEqConst(reg, c) => self.program_counter += (self.registers[reg as usize] != c) as usize * 2,
            OpCode::CompareEq(reg_x, reg_y) => self.program_counter += (self.registers[reg_x as usize] == self.registers[reg_y as usize]) as usize * 2,
            OpCode::CompareNotEq(reg_x, reg_y) => self.program_counter += (self.registers[reg_x as usize] != self.registers[reg_y as usize]) as usize * 2,
            OpCode::SetRegisterConst(reg, c) => self.registers[reg as usize] = c,
            OpCode::AddRegisterConst(reg, c) => self.registers[reg as usize] += c,
            OpCode::SetRegister(reg_x, reg_y) => self.registers[reg_x as usize] = self.registers[reg_y as usize],
            OpCode::Or(reg_x, reg_y) => self.registers[reg_x as usize] |= self.registers[reg_y as usize],
            OpCode::And(reg_x, reg_y) => self.registers[reg_x as usize] &= self.registers[reg_y as usize],
            OpCode::Xor(reg_x, reg_y) => self.registers[reg_x as usize] ^= self.registers[reg_y as usize],
            OpCode::Add(reg_x, reg_y) => {
                let add_result = self.registers[reg_x as usize].overflowing_add(self.registers[reg_y as usize]);
                self.registers[0xF] = add_result.1 as u8;
                self.registers[reg_x as usize] = add_result.0;
            },
            OpCode::Sub(reg_x, reg_y) => {
                let sub_result = self.registers[reg_x as usize].overflowing_sub(self.registers[reg_y as usize]);
                self.registers[0xF] = sub_result.1 as u8;
                self.registers[reg_x as usize] = sub_result.0;
            },
            OpCode::ShiftRight(reg) => {
                self.registers[0xF] = self.registers[reg as usize] & 1;
                self.registers[reg as usize] >>= 1;
            },
            OpCode::ShiftLeft(reg) => {
                self.registers[0xF] = (self.registers[reg as usize] & 0x80) >> 7;
                self.registers[reg as usize] <<= 1;
            },
            OpCode::SubFrom(reg_x, reg_y) => {
                let y = self.registers[reg_y as usize];
                let x_val = self.registers.get_mut(reg_x as usize).unwrap();
                *x_val = y - *x_val;
            },
            OpCode::SetAddressRegisterConst(c) => self.memory_address_register = c,
            OpCode::JumpOffset(address) => self.program_counter = address as usize + self.registers[0] as usize,
            OpCode::Random(reg, c) => {
                let rand_number = self.random_generator.rand_range(0, 255) as u8;
                self.registers[reg as usize] = rand_number & c;
            },
            OpCode::Draw(reg_x, reg_y, height) => {
                let memory_index = self.memory_address_register as usize;
                let sprite_size = height as usize * 8;
                let x = self.registers[reg_x as usize];
                let y = self.registers[reg_y as usize];
                let flip = self.display.draw_sprite(x, y, &self.memory[memory_index..memory_index + sprite_size]);
                self.registers[0xF] = flip as u8;
            },
            OpCode::KeyPressed(_) => todo!(),
            OpCode::KeyNotPressed(_) => todo!(),
            OpCode::AwaitKeyPress(_) => todo!(),
            OpCode::GetDelayTimer(reg) => self.registers[reg as usize] = self.delay_timer,
            OpCode::SetDelayTimer(reg) => self.delay_timer = self.registers[reg as usize],
            OpCode::SetSoundTimer(reg) => self.sound_timer = self.registers[reg as usize],
            OpCode::AddAddressRegister(reg) => self.memory_address_register += self.registers[reg as usize] as u16,
            OpCode::SpriteAddressRegister(_) => todo!(),
            OpCode::StoreBCD(reg) => {
                let value = self.registers[reg as usize];
                let hundreds = value / 100;
                let tens = (value % 100) / 10;
                let ones = value % 10;

                let success = self.write_memory(&[hundreds, tens, ones], self.memory_address_register as usize);
                if !success { return Err(EmulatorError::MemoryOutOfBounds) }
            },
            OpCode::DumpRegisters(reg) => {
                let memory_index = self.memory_address_register as usize;
                let register_count = reg as usize + 1;

                let memory_to_write = self.memory.get_mut(memory_index..memory_index + register_count)
                    .ok_or(EmulatorError::MemoryOutOfBounds)?;
                memory_to_write.copy_from_slice(&self.registers[0..register_count]);
            },
            OpCode::LoadRegisters(reg) => {
                let memory_index = self.memory_address_register as usize;
                let register_count = reg as usize + 1;

                let registers = &mut self.registers[0..register_count];
                let memory_registers = self.memory.get(memory_index..memory_index + register_count)
                    .ok_or(EmulatorError::MemoryOutOfBounds)?;
                registers.copy_from_slice(memory_registers);
            },
        }

        Ok(())
    }
}

pub struct EmulatorBuilder {
    mem_init: Option<u8>,
    program_counter_init: Option<usize>,
    rand_seed: Option<u32>
}

impl EmulatorBuilder {
    pub fn new() -> Self {
        Self { mem_init: None, program_counter_init: None, rand_seed: None }
    }

    pub fn initial_memory(mut self, byte: u8) -> Self {
        self.mem_init = Some(byte);
        self
    }

    pub fn initial_program_counter(mut self, pc: usize) -> Self {
        self.program_counter_init = Some(pc);
        self
    }

    pub fn seed_random(mut self, seed: u32) -> Self {
        self.rand_seed = Some(seed);
        self
    }

    pub fn build_with_program<D>(self, display: D, program: &[u8]) -> Option<Emulator<D>> {
        let mut emulator = self.build(display);
        match emulator.write_at_pc(program) {
            true => Some(emulator),
            false => None
        }
    } 

    pub fn build<D>(self, display: D) -> Emulator<D> {
        let seed = self.rand_seed.unwrap_or_else(|| SystemTime::now().duration_since(UNIX_EPOCH).unwrap().subsec_nanos());
        Emulator {
            registers: [0; 16],
            memory_address_register: 0,
            memory: [self.mem_init.unwrap_or(0); 4096],
            delay_timer: 0,
            sound_timer: 0,
            program_counter: self.program_counter_init.unwrap_or(512),
            random_generator: Rand::new(seed),
            stack: Vec::with_capacity(12),
            display
        }
    }
}