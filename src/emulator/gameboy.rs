use crate::processor::interrupt_controller::InterruptController;
use crate::processor::cpu::CPU;
use crate::mem::cartridge::Cartridge;
use crate::gpu::lcd::LCDFetcher;
use std::rc::Rc;
use std::cell::RefCell;

pub struct Gameboy {
    cpu: CPU,
}

impl Gameboy {
    pub fn new(lcd_fetcher: Rc<RefCell<LCDFetcher>>, cartridge: Cartridge, boot_rom: Option<Vec<u8>>) -> Gameboy{
        let interrupt = InterruptController::new();
        let cpu = CPU::new(interrupt, cartridge, lcd_fetcher, boot_rom);
        Gameboy{ cpu }
    }

    pub fn step(&mut self, steps: usize){
        for _ in 0 .. steps {
            self.cpu.step();
        }
    }

    pub fn render_step(&mut self) {
        use crate::gpu::ppu::TICKS_PER_CYCLE;
        self.step(TICKS_PER_CYCLE)
    }
}