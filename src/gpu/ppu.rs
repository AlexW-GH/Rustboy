use memory::memory::Memory;
use memory::memory::MapsMemory;
use util;
use gpu::lcd::LCD;
use gpu::lcd::LCDFetcher;
use std::sync::Mutex;
use std::sync::Arc;

// LCD Control Register
const LCDC_REGISTER: u16 = 0xFF40;

// LCD Status Register
const STAT_REGISTER: u16 = 0xFF41;

// LCD Position and Scrolling
const SCY_REGISTER: u16 = 0xFF42;
const SCX_REGISTER: u16 = 0xFF43;
const LY_REGISTER: u16 = 0xFF44;
const LYC_REGISTER: u16 = 0xFF45;
const WY_REGISTER: u16 = 0xFF4A;
const WX_REGISTER: u16 = 0xFF4B;

// LCD Monochrome Palettes
const BGP_REGISTER: u16 = 0xFF47;
const OBP0_REGISTER: u16 = 0xFF48;
const OBP1_REGISTER: u16 = 0xFF49;


const OAM_SEARCH_TICKS: usize = 20 * 4;
const PIXEL_TRANSFER_AND_HBLANK_TICKS: usize = 94 * 4;

const LINES_TO_DRAW: usize = 144;
const LINES_VBLANK: usize = 10;

const TICKS_PER_LINE: usize = OAM_SEARCH_TICKS + PIXEL_TRANSFER_AND_HBLANK_TICKS;
const LINES_PER_CYCLE: usize = LINES_TO_DRAW + LINES_VBLANK;
const TICKS_PER_CYCLE: usize = LINES_PER_CYCLE * TICKS_PER_LINE;

pub struct PixelProcessingUnit{
    memory: Memory,
    oam: Memory,

    lcd: LCD,
    pixel_fifo: PixelFifo,
    fetcher: Fetcher,

    current_tick: usize,
    current_pixel: usize,
    current_tile: usize,
    valid_objects: [usize; 10],
    valid_objects_count: usize,
}

impl PixelProcessingUnit {
    pub fn new(lcd_fetcher: Arc<Mutex<LCDFetcher>>) -> PixelProcessingUnit{
        let mut memory = Memory::new_read_write(&[0u8; 0], 0x8000, 0x9FFF);
        let oam = Memory::new_read_write(&[0u8; 0], 0xFE00, 0xFE9F);

        let lcd = LCD::new(lcd_fetcher);
        let pixel_fifo = PixelFifo::new();
        let fetcher = Fetcher::new();
        let current_tick = 0;
        let current_pixel = 0;
        let current_tile = 0;
        let valid_objects = [0; 10];
        let valid_objects_count = 0;
        PixelProcessingUnit{memory, oam, lcd, pixel_fifo, fetcher, current_tick, current_pixel, current_tile, valid_objects, valid_objects_count}
    }

    /*  FF40
      Bit 7 - LCD Display Enable             (0=Off, 1=On)
      Bit 6 - Window Tile Map Display Select (0=9800-9BFF, 1=9C00-9FFF)
      Bit 5 - Window Display Enable          (0=Off, 1=On)
      Bit 4 - BG & Window Tile Data Select   (0=8800-97FF, 1=8000-8FFF)
      Bit 3 - BG Tile Map Display Select     (0=9800-9BFF, 1=9C00-9FFF)
      Bit 2 - OBJ (Sprite) Size              (0=8x8, 1=8x16)
      Bit 1 - OBJ (Sprite) Display Enable    (0=Off, 1=On)
      Bit 0 - BG Display (for CGB see below) (0=Off, 1=On)
    */

    pub fn step(&mut self, io_registers: &mut Memory){
        if self.current_tick % TICKS_PER_LINE == 0 {
            self.current_pixel = 0;
        }
        match self.current_tick % TICKS_PER_LINE {
            0 .. OAM_SEARCH_TICKS => self.oam_search(io_registers),
            OAM_SEARCH_TICKS ..TICKS_PER_LINE => self.pixel_transfer(io_registers),
            _ => ()
        }

        if self.current_tick < TICKS_PER_CYCLE {
            self.current_tick += 1;
        } else {
            self.current_tick = 0;
        }
    }

    pub fn oam_search(&mut self, io_registers: &mut Memory){
        //Todo!
    }

    pub fn pixel_transfer(&mut self, io_registers: &mut Memory){
        if self.current_pixel < 160 {
            let line = self.current_tick / TICKS_PER_LINE;
            if line < 144 {
                if self.current_pixel == 160 {
                    //Todo: HBLANK!
                }
                let lcd_control_register = io_registers.read(LCDC_REGISTER).unwrap();
                self.current_pixel += self.pixel_fifo.write_pixel(&mut self.lcd, self.current_pixel as u32, line as u32);

                /* FIXME */ let current_tile = 0;
                self.fetcher.fetch_tile(current_tile, lcd_control_register, &mut self.pixel_fifo, &self.memory );
            } else if line == 144 {
                //Todo: VBLANK!
                self.lcd.display();
            }

        }

    }
}

impl MapsMemory for PixelProcessingUnit{
    fn read(&self, address: u16) -> Result<u8, ()> {
        if self.memory.is_in_range(address){
            self.memory.read(address)
        } else if self.oam.is_in_range(address) {
            self.oam.read(address)
        } else {
            Err(())
        }
    }

    fn write(&mut self, address: u16, value: u8) -> Result<(), ()> {
        if self.memory.is_in_range(address){
            self.memory.write(address, value)
        } else if self.oam.is_in_range(address) {
            self.oam.write(address, value)
        } else {
            Err(())
        }
    }

    fn is_in_range(&self, address: u16) -> bool{
        let vram = self.memory.is_in_range(address);
        let oam = self.memory.is_in_range(address);
        vram | oam
    }
}

struct PixelFifo {
    current_size: usize,
    color_queue: u32,
    palette_queue: u32,
}

impl PixelFifo{
    pub fn new() -> PixelFifo{
        PixelFifo { current_size: 0, color_queue: 0, palette_queue: 0}
    }

    pub fn write_pixel(&mut self, lcd: &mut LCD, pixel_in_line: u32, line: u32) -> usize{
        if self.current_size >= 7 {
            let color = ((self.color_queue >> 30) & 0b11) as u8;
            lcd.set_pixel(pixel_in_line,line, color);
            self.color_queue = self.color_queue << 2;
            self.current_size -= 1;
            return 1;
        }
        return 0;
    }

    pub fn push(&mut self, pixels: u16){
        assert!(self.current_size < 8);
        self.color_queue |= (pixels as u32) << ((16 - self.current_size) - 8);
        self.current_size += 8;
    }

    pub fn is_free(&self) -> bool{
        self.current_size < 8
    }
}

struct Fetcher {
    current_step: FetcherStep,
    current_tile: u8,
    fetched_color: u16,
    fetched_palette: u16,
}

enum FetcherStep{
    ReadTile,
    ReadData0,
    ReadData1,
    WriteData,
}

impl FetcherStep{
    fn next(&self) -> FetcherStep {
        match self {
            FetcherStep::ReadTile => FetcherStep::ReadData0,
            FetcherStep::ReadData0 => FetcherStep::ReadData1,
            FetcherStep::ReadData1 => FetcherStep::WriteData,
            FetcherStep::WriteData => FetcherStep::ReadTile,
        }
    }
}

impl Fetcher{
    pub fn new() -> Fetcher {
        Fetcher {current_step: FetcherStep::ReadTile, current_tile: 0, fetched_color: 0, fetched_palette: 0}
    }

    pub fn fetch_tile(&mut self, tile: u16, lcd_control_register: u8, pixel_fifo: &mut PixelFifo, vram: &Memory) -> u8{
        match self.current_step{
            FetcherStep::ReadTile => {
                let bg_map_address = if (lcd_control_register >> 3) & 1 == 0 { 0x9800 } else { 0x9C00};
                self.current_tile = vram.read(bg_map_address + tile * 0x8).unwrap();
                self.current_step = self.current_step.next();
                return 0;
            },
            FetcherStep::ReadData0 => {
                let bg_tiles_address = if (lcd_control_register >> 4) & 1 == 0 { 0x8000 } else { 0x9000};
                if bg_tiles_address == 0x8000 {
                    self.fetched_color = (vram.read(0x8000 + (self.current_tile as u16 * 0x10)).unwrap() as u16) << 8;
                } else if bg_tiles_address == 0x9000{
                    // Todo: Recheck later
                    let mapped_tile = self.current_tile as i32;
                    self.fetched_color = ((vram.read((0x9000 + (mapped_tile* 0x10)) as u16)).unwrap() as u16) << 8;
                } else {unimplemented!()}
                self.current_step = self.current_step.next();
                return 0;
            }
            FetcherStep::ReadData1 => {
                let bg_tiles_address = if (lcd_control_register >> 4) & 1 == 0 { 0x8000 } else { 0x9000};
                if bg_tiles_address == 0x8000 {
                    self.fetched_color |= vram.read(0x8001 + (self.current_tile as u16 * 0x10)).unwrap() as u16;
                } else if bg_tiles_address == 0x9000{
                    // Todo: Recheck later
                    let mapped_tile = self.current_tile as i32;
                    self.fetched_color |= (vram.read((0x9001 + (mapped_tile* 0x10)) as u16)).unwrap() as u16;
                } else {unimplemented!()}
                self.current_step = self.current_step.next();
                self.fetch_tile(tile, lcd_control_register, pixel_fifo, vram);
                return 0;
            }
            FetcherStep::WriteData => {
                if pixel_fifo.is_free() {
                    pixel_fifo.push(self.fetched_color);
                    self.current_step = self.current_step.next();
                    return 1;
                }
                return 0;
            },
        }
    }
}