use chip8_emulator::{EmulatorBuilder, display::MatrixDisplay};

const PROGRAM: &[u8] = &[0x70, 0xFF, 0x73, 0xFF, 0xAF, 0xA0, 0xFF, 0x55, 0xD1, 0x21];

fn main() {
    let mut emulator = EmulatorBuilder::new()
        .build_with_program(MatrixDisplay::new(), PROGRAM).unwrap();

    emulator.next_opcode().unwrap();
    emulator.next_opcode().unwrap();
    emulator.next_opcode().unwrap();
    emulator.next_opcode().unwrap();
    emulator.next_opcode().unwrap();

    println!("{:?}", emulator.display.screen);
}
