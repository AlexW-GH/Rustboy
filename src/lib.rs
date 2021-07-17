#[macro_use]
extern crate log;

mod debug;
mod emulator;
mod gpu;
mod mem;
mod processor;
mod util;

pub use emulator::gameboy::{Emulator, Gameboy};
pub use mem::cartridge::Cartridge;
