/// Only low 12 bits are used
pub type MemoryAddress = u16;

pub type Constant = u8;

/// Only low 4 bits are used
pub type RegisterIdentifier = u8;

/// Only low 12 bits are used
pub type AddressRegister = u16;

pub fn split_opcode_bytes(opcode_bytes: (u8, u8)) -> (u8, u8, u8, u8) {
    (
        ((opcode_bytes.0 & 0xF0) >> 4) as u8,
        (opcode_bytes.0 & 0x0F) as u8,
        ((opcode_bytes.1 & 0xF0) >> 4) as u8,
        (opcode_bytes.1 & 0x0F) as u8
    )
}

macro_rules! addr_from_op {
    ($value: ident) => {
        u16::from_be_bytes([$value.0, $value.1]) & 0x0FFF
    };
}

#[derive(Debug)]
pub enum OpCode {
    CallMachineCode(MemoryAddress),
    DisplayClear,
    Return,
    Jump(MemoryAddress),
    CallSubroutine(MemoryAddress),
    CompareEqConst(RegisterIdentifier, Constant),
    CompareNotEqConst(RegisterIdentifier, Constant),
    CompareEq(RegisterIdentifier, RegisterIdentifier),
    CompareNotEq(RegisterIdentifier, RegisterIdentifier),
    SetRegisterConst(RegisterIdentifier, Constant),
    AddRegisterConst(RegisterIdentifier, Constant),
    SetRegister(RegisterIdentifier, RegisterIdentifier),
    Or(RegisterIdentifier, RegisterIdentifier),
    And(RegisterIdentifier, RegisterIdentifier),
    Xor(RegisterIdentifier, RegisterIdentifier),
    Add(RegisterIdentifier, RegisterIdentifier),
    Sub(RegisterIdentifier, RegisterIdentifier),
    ShiftRight(RegisterIdentifier),
    ShiftLeft(RegisterIdentifier),
    SubFrom(RegisterIdentifier, RegisterIdentifier),
    SetAddressRegisterConst(MemoryAddress),
    JumpOffset(MemoryAddress),
    Random(RegisterIdentifier, Constant),
    Draw(RegisterIdentifier, RegisterIdentifier, Constant),
    KeyPressed(RegisterIdentifier),
    KeyNotPressed(RegisterIdentifier),
    AwaitKeyPress(RegisterIdentifier),
    GetDelayTimer(RegisterIdentifier),
    SetDelayTimer(RegisterIdentifier),
    SetSoundTimer(RegisterIdentifier),
    AddAddressRegister(RegisterIdentifier),
    SpriteAddressRegister(RegisterIdentifier),
    StoreBCD(RegisterIdentifier),
    DumpRegisters(RegisterIdentifier),
    LoadRegisters(RegisterIdentifier)
}

impl TryFrom<(u8, u8)> for OpCode {
    type Error = ();

    fn try_from(value: (u8, u8)) -> Result<Self, Self::Error> {
        let opcode = match split_opcode_bytes(value) {
            (0, 0, 0xE, 0) => OpCode::DisplayClear,
            (0, 0, 0xE, 0xE) => OpCode::Return,
            (0, ..) => OpCode::CallMachineCode(addr_from_op!(value)),
            (1, ..) => OpCode::Jump(addr_from_op!(value)),
            (2, ..) => OpCode::CallSubroutine(addr_from_op!(value)),
            (3, reg, ..) => OpCode::CompareEqConst(reg, value.1),
            (4, reg, ..) => OpCode::CompareNotEqConst(reg, value.1),
            (5, reg_x, reg_y, 0) => OpCode::CompareEq(reg_x, reg_y),
            (6, reg, ..) => OpCode::SetRegisterConst(reg, value.1),
            (7, reg, ..) => OpCode::AddRegisterConst(reg, value.1),
            (8, reg_x, reg_y, 0) => OpCode::SetRegister(reg_x, reg_y),
            (8, reg_x, reg_y, 1) => OpCode::Or(reg_x, reg_y),
            (8, reg_x, reg_y, 2) => OpCode::And(reg_x, reg_y),
            (8, reg_x, reg_y, 3) => OpCode::Xor(reg_x, reg_y),
            (8, reg_x, reg_y, 4) => OpCode::Add(reg_x, reg_y),
            (8, reg_x, reg_y, 5) => OpCode::Sub(reg_x, reg_y),
            (8, reg, _, 6) => OpCode::ShiftRight(reg),
            (8, reg_x, reg_y, 7) => OpCode::SubFrom(reg_x, reg_y),
            (8, reg, _, 0xE) => OpCode::ShiftLeft(reg),
            (9, reg_x, reg_y, 0) => OpCode::CompareNotEq(reg_x, reg_y),
            (0xA, ..) => OpCode::SetAddressRegisterConst(addr_from_op!(value)),
            (0xB, ..) => OpCode::JumpOffset(addr_from_op!(value)),
            (0xC, reg, ..) => OpCode::Random(reg, value.0),
            (0xD, reg_x, reg_y, height) => OpCode::Draw(reg_x, reg_y, height),
            (0xE, reg, 9, 0xE) => OpCode::KeyPressed(reg),
            (0xE, reg, 0xA, 1) => OpCode::KeyNotPressed(reg),
            (0xF, reg, 0, 7) => OpCode::GetDelayTimer(reg),
            (0xF, reg, 0, 0xA) => OpCode::AwaitKeyPress(reg),
            (0xF, reg, 1, 5) => OpCode::SetDelayTimer(reg),
            (0xF, reg, 1, 8) => OpCode::SetSoundTimer(reg),
            (0xF, reg, 1, 0xE) => OpCode::AddAddressRegister(reg),
            (0xF, reg, 2, 9) => OpCode::SpriteAddressRegister(reg),
            (0xF, reg, 3, 3) => OpCode::StoreBCD(reg),
            (0xF, reg, 5, 5) => OpCode::DumpRegisters(reg),
            (0xF, reg, 6, 5) => OpCode::LoadRegisters(reg),
            _ => return Err(())
        };

        Ok(opcode)
    }
}
