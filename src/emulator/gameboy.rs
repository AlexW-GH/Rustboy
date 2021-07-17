use crate::debug::vram_fetcher::VRAMFetcher;
use crate::debug::vram_fetcher::VramDebugger;
use crate::{
    gpu::screen::ScreenFetcher,
    mem::cartridge::Cartridge,
    processor::{cpu::Cpu, interrupt_controller::InterruptController},
};
use image::{ImageBuffer, Rgba};
use std::{cell::RefCell, rc::Rc};

pub trait Emulator {
    fn step(&mut self, steps: usize);
    fn render_step(&mut self);
    fn load_cartridge(&mut self, cartridge: Cartridge);
}

pub struct Gameboy {
    cpu: Option<Cpu>,
    lcd_fetcher: Rc<RefCell<ScreenFetcher>>,
    boot_rom: Option<Vec<u8>>,
}

impl Gameboy {
    pub fn new(boot_rom: Option<Vec<u8>>) -> Self {
        let lcd_fetcher = Rc::new(RefCell::new(ScreenFetcher::new()));
        Gameboy {
            cpu: None,
            lcd_fetcher,
            boot_rom,
        }
    }

    pub fn game_title(&self) -> &str {
        if let Some(cpu) = &self.cpu {
            cpu.game_title()
        } else {
            ""
        }
    }

    pub fn screen(&self) -> ImageBuffer<Rgba<u8>, Vec<u8>> {
        RefCell::borrow(&self.lcd_fetcher).image().clone()
    }
}

impl Emulator for Gameboy {
    fn step(&mut self, steps: usize) {
        if let Some(cpu) = &mut self.cpu {
            for _ in 0..steps {
                cpu.step();
            }
        }
    }

    fn render_step(&mut self) {
        use crate::gpu::ppu::TICKS_PER_CYCLE;
        self.step(TICKS_PER_CYCLE)
    }

    fn load_cartridge(&mut self, cartridge: Cartridge) {
        let interrupt = InterruptController::new();
        self.cpu = Some(Cpu::new(
            interrupt,
            cartridge,
            self.lcd_fetcher.clone(),
            self.boot_rom.clone(),
        ));
    }
}

impl VramDebugger for Gameboy {
    fn render_all_background_tiles(&self, fetcher: VRAMFetcher) -> ImageBuffer<Rgba<u8>, Vec<u8>> {
        if let Some(cpu) = &self.cpu {
            return fetcher.render_all_background_tiles(cpu);
        }
        ImageBuffer::new(0, 0)
    }

    fn render_background_tilemap(&self, fetcher: VRAMFetcher) -> ImageBuffer<Rgba<u8>, Vec<u8>> {
        if let Some(cpu) = &self.cpu {
            return fetcher.render_background_tilemap(cpu);
        }
        ImageBuffer::new(0, 0)
    }
}
