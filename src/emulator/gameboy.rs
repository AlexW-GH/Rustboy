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
use crate::debug::vram_fetcher::VramDebugger;
use crate::debug::vram_fetcher::VRAMFetcher;
use image::{
    ImageBuffer,
    Rgba,
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

impl VramDebugger for Gameboy{
    fn render_all_background_tiles(&self, fetcher: VRAMFetcher) -> ImageBuffer<Rgba<u8>, Vec<u8>> {
        fetcher.render_all_background_tiles(&self.cpu)
    }

    fn render_background_tilemap(&self, fetcher: VRAMFetcher) -> ImageBuffer<Rgba<u8>, Vec<u8>> {
        fetcher.render_background_tilemap(&self.cpu)
    }
}
