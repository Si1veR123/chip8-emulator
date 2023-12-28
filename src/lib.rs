pub mod opcode;
pub mod rand;
pub mod display;

mod emulator;
pub use emulator::{Emulator, EmulatorBuilder, EmulatorError};
