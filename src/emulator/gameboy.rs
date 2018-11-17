use std::sync::Arc;
use std::sync::RwLock;
use gpu::lcd::LCD;
use processor::interrupt_controller::InterruptController;
use processor::cpu::CPU;
use memory::cartridge::Cartridge;
use gpu::ppu::PixelProcessingUnit;
use std::time::Instant;
use std::sync::Mutex;
use gpu::lcd::LCDFetcher;

const NANO_CYCLE_TIME: i64 = 238;

pub struct Gameboy {
    cpu: CPU,
}

impl Gameboy {
    pub fn new(lcd_fetcher: Arc<Mutex<LCDFetcher>>, cartridge: Cartridge, boot_rom: Option<Vec<u8>>) -> Gameboy{
        let interrupt = InterruptController::new();
        let cpu = CPU::new(interrupt, cartridge, lcd_fetcher, boot_rom);
        Gameboy{ cpu }
    }

    pub fn run(&mut self){
        let mut clock = ClockGenerator::new();
        clock.reset();
        loop{
            clock.wait_next_tick();
            self.cpu.step();
        }
    }
}

struct ClockGenerator{
    time_spent: i64,
    last_measure: Instant
}

impl ClockGenerator{
    pub fn new() -> ClockGenerator{
        let time_spent = 0;
        let last_measure = Instant::now();
        ClockGenerator{time_spent, last_measure}
    }

    pub fn reset(&mut self){
        self.time_spent = 0;
        self.last_measure = Instant::now();
    }

    pub fn wait_next_tick(&mut self){
        loop {
            self.time_spent += self.last_measure.elapsed().as_nanos() as i64;
            if self.time_spent >= NANO_CYCLE_TIME{
                self.time_spent -= NANO_CYCLE_TIME;
                self.last_measure = Instant::now();
                break
            }
        }
    }
}