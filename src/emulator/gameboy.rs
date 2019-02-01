use crate::{
    gpu::lcd::LCDFetcher,
    mem::cartridge::Cartridge,
    processor::{
        cpu::CPU,
        interrupt_controller::InterruptController,
    },
};
use std::{
    cell::RefCell,
    rc::Rc,
};

pub struct Gameboy {
    cpu: CPU,
}

impl Gameboy {
    pub fn new(
        lcd_fetcher: Rc<RefCell<LCDFetcher>>,
        cartridge: Cartridge,
        boot_rom: Option<Vec<u8>>,
    ) -> Gameboy {
        let interrupt = InterruptController::new();
        let cpu = CPU::new(interrupt, cartridge, lcd_fetcher, boot_rom);
        Gameboy { cpu }
    }

    pub fn step(&mut self, steps: usize) {
        for _ in 0..steps {
            self.cpu.step();
        }
    }

    pub fn render_step(&mut self) {
        use crate::gpu::ppu::TICKS_PER_CYCLE;
        self.step(TICKS_PER_CYCLE)
    }
}
