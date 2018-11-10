use std::sync::Arc;
use std::sync::RwLock;
use gpu::lcd::LCD;
use processor::interrupt_controller::InterruptController;
use processor::cpu::CPU;
use memory::cartridge::Cartridge;
use gpu::ppu::PixelProcessingUnit;

pub struct Gameboy {
    cpu: CPU,
}

impl Gameboy {
    pub fn new(lcd: Arc<RwLock<LCD>>, cartridge: Cartridge, ppu: PixelProcessingUnit, boot: bool) -> Gameboy{
        let interrupt = InterruptController::new();
        let cpu = CPU::new(interrupt, cartridge, ppu, boot);
        Gameboy{ cpu }
    }

    pub fn run(&mut self){
        self.cpu.run();
    }
}
