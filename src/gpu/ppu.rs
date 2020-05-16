use super::lcd::{
    LCDFetcher,
    LCD,
};
use crate::{
    mem::memory::{
        MapsMemory,
        Memory,
    },
    processor::interrupt_controller::InterruptController,
};
use std::{
    cell::RefCell,
    rc::Rc,
};

// LCD Control Register
const LCDC_REGISTER: u16 = 0xFF40;

// LCD Position and Scrolling
const SCY_REGISTER: u16 = 0xFF42;
const SCX_REGISTER: u16 = 0xFF43;
const LY_REGISTER: u16 = 0xFF44;

const OAM_SEARCH_TICKS: usize = 20 * 4;
const PIXEL_TRANSFER_AND_HBLANK_TICKS: usize = 94 * 4;

const LINES_TO_DRAW: usize = 144;
const LINES_VBLANK: usize = 10;

const TICKS_PER_LINE: usize = OAM_SEARCH_TICKS + PIXEL_TRANSFER_AND_HBLANK_TICKS;
const LINES_PER_CYCLE: usize = LINES_TO_DRAW + LINES_VBLANK;
pub const TICKS_PER_CYCLE: usize = LINES_PER_CYCLE * TICKS_PER_LINE;

const TILES_IN_LINES: u16 = 0x14;

pub enum PPUMode {
    HBLANK    = 0,
    VBLANK    = 1,
    OAMSEARCH = 2,
    TRANSFER  = 3,
}

pub struct PixelProcessingUnit {
    memory: Memory,
    oam:    Memory,

    lcd:        LCD,
    pixel_fifo: PixelFifo,
    fetcher:    Fetcher,

    current_tick:  usize,
    current_pixel: u8,
    current_line:  u8,
    mode:          PPUMode,
}

impl PixelProcessingUnit {
    pub fn new(lcd_fetcher: Rc<RefCell<LCDFetcher>>) -> PixelProcessingUnit {
        let memory = Memory::new_read_write(&[0u8; 0], 0x8000, 0x9FFF);
        let oam = Memory::new_read_write(&[0u8; 0], 0xFE00, 0xFE9F);

        let lcd = LCD::new(lcd_fetcher);
        let pixel_fifo = PixelFifo::new();
        let fetcher = Fetcher::new();
        let current_tick = 0;
        let current_pixel = 0;
        let current_line = 0;
        PixelProcessingUnit {
            memory,
            oam,
            lcd,
            pixel_fifo,
            fetcher,
            current_tick,
            current_pixel,
            mode: PPUMode::OAMSEARCH,
            current_line,
        }
    }

    pub fn step(&mut self, io_registers: &mut Memory, _interrupt: &mut InterruptController) {
        match self.mode {
            PPUMode::HBLANK => self.h_blank(io_registers),
            PPUMode::VBLANK => self.v_blank(io_registers),
            PPUMode::OAMSEARCH => self.oam_search(io_registers),
            PPUMode::TRANSFER => self.pixel_transfer(io_registers),
        }

        if self.current_tick <= TICKS_PER_CYCLE {
            self.current_tick += 1;
        } else {
            self.current_tick = 0;
            self.pixel_fifo.reset();
            self.fetcher.reset();
        }
    }

    pub fn v_blank(&mut self, io_registers: &mut Memory) {
        if (self.current_tick + 1) % TICKS_PER_LINE == 0 {
            if (self.current_tick + 1) % TICKS_PER_CYCLE == 0 {
                self.mode = PPUMode::OAMSEARCH;
                self.current_line = 0;
            } else {
                self.current_line += 1;
            }

            self.current_pixel = 0;
            io_registers.write(LY_REGISTER, self.current_line).unwrap();
        }
    }

    pub fn h_blank(&mut self, io_registers: &mut Memory) {
        if (self.current_tick + 1) % TICKS_PER_LINE == 0 {
            if self.current_line as usize == LINES_TO_DRAW {
                self.mode = PPUMode::VBLANK;
                self.current_line = 0;
                self.lcd.display();
            } else {
                self.mode = PPUMode::OAMSEARCH;
                self.current_line += 1;
            }

            self.current_pixel = 0;
            io_registers.write(LY_REGISTER, self.current_line).unwrap();
        }
    }

    pub fn oam_search(&mut self, _io_registers: &mut Memory) {
        if (((self.current_tick + 1) % TICKS_PER_LINE) % OAM_SEARCH_TICKS) == 0 {
            self.mode = PPUMode::TRANSFER;
        }
        // Todo!
        // println!("oam search @ tick: {}", self.current_tick);
    }

    pub fn pixel_transfer(&mut self, io_registers: &mut Memory) {
        if (self.current_line as usize) < LINES_TO_DRAW {
            if self.current_pixel < 160 {
                if self.current_tick % 2 == 1 {
                    self.fetcher.fetch_tile(&mut self.pixel_fifo, &self.memory, io_registers);
                }
                self.pixel_fifo.write_pixel(
                    &mut self.lcd,
                    &mut self.current_pixel,
                    self.current_line,
                );
            } else if self.current_pixel >= 160 {
                self.mode = PPUMode::HBLANK;
                self.current_pixel = 0;
            }
        } else if self.current_line as usize == LINES_TO_DRAW {
            self.mode = PPUMode::VBLANK;
            self.lcd.display();
        }
    }
}

impl MapsMemory for PixelProcessingUnit {
    fn read(&self, address: u16) -> Result<u8, ()> {
        if self.memory.is_in_range(address) {
            self.memory.read(address)
        } else if self.oam.is_in_range(address) {
            self.oam.read(address)
        } else {
            Err(())
        }
    }

    fn write(&mut self, address: u16, value: u8) -> Result<(), ()> {
        if self.memory.is_in_range(address) {
            self.memory.write(address, value)
        } else if self.oam.is_in_range(address) {
            self.oam.write(address, value)
        } else {
            Err(())
        }
    }

    fn is_in_range(&self, address: u16) -> bool {
        let vram = self.memory.is_in_range(address);
        let oam = self.oam.is_in_range(address);
        vram | oam
    }
}

struct PixelFifo {
    current_size: usize,
    color_queue:  u32,
}

impl PixelFifo {
    pub fn new() -> PixelFifo {
        PixelFifo { current_size: 0, color_queue: 0 }
    }

    pub fn write_pixel(&mut self, lcd: &mut LCD, pixel_in_line: &mut u8, line: u8) {
        if self.current_size >= 8 {
            let color = self.pop();
            lcd.set_pixel(u32::from(*pixel_in_line), u32::from(line), color);
            *pixel_in_line += 1;
        }
    }

    fn pop(&mut self) -> u8 {
        let color = ((self.color_queue >> 30) & 0b11) as u8;
        self.color_queue <<= 2;
        self.current_size -= 1;
        color
    }

    pub fn push(&mut self, pixels: u16) {
        assert!(self.current_size < 8);
        self.color_queue |= u32::from(pixels) << ((16 - self.current_size) - 8);
        self.current_size += 8;
    }

    pub fn is_free(&self) -> bool {
        self.current_size < 8
    }

    fn reset(&mut self) {
        self.current_size = 0;
    }
}

struct Fetcher {
    current_tile_address: u16,
    current_map_line: u8,
    current_step: FetcherStep,
    current_tile_number: u16,
    data0: u8,
    data1: u8,
}

enum FetcherStep {
    ReadTile,
    ReadData0,
    ReadData1,
    WriteData,
}

impl FetcherStep {
    fn next(&self) -> FetcherStep {
        match self {
            FetcherStep::ReadTile => FetcherStep::ReadData0,
            FetcherStep::ReadData0 => FetcherStep::ReadData1,
            FetcherStep::ReadData1 => FetcherStep::WriteData,
            FetcherStep::WriteData => FetcherStep::ReadTile,
        }
    }
}

impl Fetcher {
    pub fn new() -> Fetcher {
        Fetcher {
            current_step: FetcherStep::ReadTile,
            current_tile_number: 0,
            current_tile_address: 0,
            data0: 0,
            data1: 0,
            current_map_line: 0,
        }
    }

    pub fn fetch_tile(
        &mut self,
        pixel_fifo: &mut PixelFifo,
        vram: &Memory,
        io_registers: &mut Memory,
    ) {
        match self.current_step {
            FetcherStep::ReadTile => self.read_tile(vram, io_registers),
            FetcherStep::ReadData0 => self.read_data0(vram, io_registers),
            FetcherStep::ReadData1 => self.read_data1(pixel_fifo, vram, io_registers),
            FetcherStep::WriteData => self.write_data(pixel_fifo),
        }
    }

    fn read_tile(&mut self, vram: &Memory, io_registers: &mut Memory) {
        let lcd_control_register = io_registers.read(LCDC_REGISTER).unwrap();
        let bg_map_address = if (lcd_control_register >> 3) & 1 == 0 { 0x9800 } else { 0x9C00 };
        let _scx = u16::from(io_registers.read(SCX_REGISTER).unwrap());
        let scy = io_registers.read(SCY_REGISTER).unwrap();
        let tile_map_address = bg_map_address
            + self.current_tile_address
            + u16::from((self.current_map_line.wrapping_add(scy) as u8) / 8) * 0x20;
        // println!("line: {:#04x} | scy: {} | bg_map_address: {:#06x} | tile: {:#06x} | translated line: {:#04x} | tile_map_address: {:#06x}", self.current_map_line, scy, bg_map_address, self.current_tile_address, ((self.current_map_line.wrapping_add(scy) / 8) as u16), tile_map_address);
        self.current_tile_number = u16::from(vram.read(tile_map_address).unwrap());
        self.current_tile_address += 1;
        self.current_step = self.current_step.next();
    }

    fn read_data0(&mut self, vram: &Memory, io_registers: &mut Memory) {
        let lcd_control_register = io_registers.read(LCDC_REGISTER).unwrap();
        let bg_tiles_address = if (lcd_control_register >> 4) & 1 == 0 { 0x9000 } else { 0x8000 };
        if bg_tiles_address == 0x8000 {
            self.data0 = vram
                .read(
                    bg_tiles_address
                        + u16::from((self.current_map_line % 8) * 0x2)
                        + (self.current_tile_number * 0x10),
                )
                .unwrap();
            if self.current_map_line % 8 == 0 {
                // println!("data0 ({:#06x}), {:#010b}", bg_tiles_address +
                // ((self.current_map_line % 8) * 0x2) as u16 + (self.current_tile_number *
                // 0x10), self.data0);
            }
        } else if bg_tiles_address == 0x9000 {
            // Todo: Recheck later
            let mapped_tile = self.current_tile_number as i8;
            self.data0 = vram
                .read(
                    (i32::from(bg_tiles_address)
                        + ((i32::from(self.current_map_line) % 8) * 0x2)
                        + (i32::from(mapped_tile) * 0x10) as i32) as u16,
                )
                .unwrap();
        } else {
            unimplemented!()
        }
        self.current_step = self.current_step.next();
    }

    fn read_data1(&mut self, pixel_fifo: &mut PixelFifo, vram: &Memory, io_registers: &mut Memory) {
        let lcd_control_register = io_registers.read(LCDC_REGISTER).unwrap();
        let bg_tiles_address =
            if (lcd_control_register >> 4) & 1 == 0 { 0x9000 } else { 0x8000 } + 1;
        if bg_tiles_address == 0x8001 {
            self.data1 = vram
                .read(
                    bg_tiles_address
                        + u16::from((self.current_map_line % 8) * 0x2)
                        + (self.current_tile_number * 0x10),
                )
                .unwrap();
            if self.current_map_line % 8 == 0 {
                // println!("data1 ({:#06x}), {:#010b}", bg_tiles_address +
                // ((self.current_map_line % 8) * 0x2) as u16 + (self.current_tile_number *
                // 0x10), self.data1);
            }
        } else if bg_tiles_address == 0x9001 {
            // Todo: Recheck later
            let _mapped_tile = i32::from(self.current_tile_number);
            self.data1 = vram
                .read(
                    (i32::from(bg_tiles_address)
                        + ((i32::from(self.current_map_line) % 8) * 0x2)
                        + i32::from(self.current_tile_number * 0x10)) as u16,
                )
                .unwrap();
        } else {
            unimplemented!()
        }
        self.current_step = self.current_step.next();
        self.write_data(pixel_fifo);
    }

    fn write_data(&mut self, pixel_fifo: &mut PixelFifo) {
        if pixel_fifo.is_free() {
            pixel_fifo.push(self.combine_pixels());
            self.current_step = self.current_step.next();
            if self.current_tile_address % TILES_IN_LINES == 0 && self.current_tile_address != 0 {
                self.current_tile_address = 0;
                self.current_map_line = (self.current_map_line + 1) % LINES_TO_DRAW as u8;
            };
        }
    }

    fn combine_pixels(&mut self) -> u16 {
        let mut result: u16 = 0;
        result |= u16::from(((self.data1 >> 7) & 1) << 1 | ((self.data0 >> 7) & 1)) << 14;
        result |= u16::from(((self.data1 >> 6) & 1) << 1 | ((self.data0 >> 6) & 1)) << 12;
        result |= u16::from(((self.data1 >> 5) & 1) << 1 | ((self.data0 >> 5) & 1)) << 10;
        result |= u16::from(((self.data1 >> 4) & 1) << 1 | ((self.data0 >> 4) & 1)) << 8;
        result |= u16::from(((self.data1 >> 3) & 1) << 1 | ((self.data0 >> 3) & 1)) << 6;
        result |= u16::from(((self.data1 >> 2) & 1) << 1 | ((self.data0 >> 2) & 1)) << 4;
        result |= u16::from(((self.data1 >> 1) & 1) << 1 | ((self.data0 >> 1) & 1)) << 2;
        result |= u16::from(((self.data1) & 1) << 1 | ((self.data0) & 1));
        // println!("Combined Pixels: {:#018b}", result);
        result
    }

    fn reset(&mut self) {
        self.current_tile_address = 0;
        self.current_map_line = 0;
        self.current_step = FetcherStep::ReadTile;
    }
}
