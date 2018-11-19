use memory::memory::Memory;
use memory::memory::MapsMemory;
use util;
use gpu::lcd::LCD;
use gpu::lcd::LCDFetcher;
use std::sync::Mutex;
use std::sync::Arc;
use processor::interrupt_controller::InterruptController;

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


const OAM_SEARCH_TICKS: usize = 20 * 2;
const PIXEL_TRANSFER_AND_HBLANK_TICKS: usize = 94 * 2;

const LINES_TO_DRAW: usize = 144;
const LINES_VBLANK: usize = 10;

const TICKS_PER_LINE: usize = OAM_SEARCH_TICKS + PIXEL_TRANSFER_AND_HBLANK_TICKS;
const LINES_PER_CYCLE: usize = LINES_TO_DRAW + LINES_VBLANK;
const TICKS_PER_CYCLE: usize = LINES_PER_CYCLE * TICKS_PER_LINE;

const TILES_IN_LINES: u16 = 0x14;
const TILE_LINES: u16 = 0x12;
const MAX_TILES: u16 = TILE_LINES * TILES_IN_LINES;

pub struct PixelProcessingUnit{
    memory: Memory,
    oam: Memory,

    lcd: LCD,
    pixel_fifo: PixelFifo,
    fetcher: Fetcher,

    current_tick: usize,
    current_pixel: usize,
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
        let valid_objects = [0; 10];
        let valid_objects_count = 0;
        PixelProcessingUnit{memory, oam, lcd, pixel_fifo, fetcher, current_tick, current_pixel, valid_objects, valid_objects_count}
    }

    pub fn step(&mut self, io_registers: &mut Memory, interrupt: &mut InterruptController){
        if self.current_tick % TICKS_PER_LINE as usize == 0 {
            self.current_pixel = 0;
        }
        match self.current_tick % TICKS_PER_LINE as usize {
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

        let mut line =  io_registers.read(LY_REGISTER).unwrap();
        if (line as usize) < LINES_TO_DRAW {
            if self.current_pixel == 160 {
                //Todo: HBLANK!
                self.current_pixel += 1;
            } else if self.current_pixel < 160{
                if self.current_tick % 2 == 1 {
                    self.fetcher.fetch_tile(&mut self.pixel_fifo, &self.memory, io_registers);
                }

                self.current_pixel += self.pixel_fifo.write_pixel(&mut self.lcd, self.current_pixel as u32, line as u32);
            }
        } else {
            if line as usize == LINES_TO_DRAW {
                //Todo: VBLANK!
                self.lcd.display();
            }
            if line as usize >= LINES_PER_CYCLE {
                io_registers.write(LY_REGISTER, 0).unwrap();
            } else {
                io_registers.write(LY_REGISTER, line+1).unwrap();
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
        if self.current_size >= 8 {
            let color = self.pop();
            lcd.set_pixel(pixel_in_line,line, color);
            return 1;
        }
        return 0;
    }

    fn pop(&mut self) -> u8{
        let color = ((self.color_queue >> 30) & 0b11) as u8;
        self.color_queue = self.color_queue << 2;
        self.current_size -= 1;
        color
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
    current_tile_number: u16,
    next_tile: u16,
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
        Fetcher {current_step: FetcherStep::ReadTile, current_tile_number: 0, next_tile: 0, fetched_color: 0, fetched_palette: 0}
    }

    pub fn fetch_tile(&mut self, pixel_fifo: &mut PixelFifo, vram: &Memory, io_registers: &mut Memory) {
        match self.current_step{
            FetcherStep::ReadTile => self.read_tile(vram, io_registers),
            FetcherStep::ReadData0 => self.read_data0(vram, io_registers),
            FetcherStep::ReadData1 => self.read_tile1(pixel_fifo, vram, io_registers),
            FetcherStep::WriteData => self.write_data(pixel_fifo),
        }
    }

    fn read_tile(&mut self, vram: &Memory, io_registers: &mut Memory) {
        let lcd_control_register = io_registers.read(LCDC_REGISTER).unwrap();
        let bg_map_address = if (lcd_control_register >> 3) & 1 == 0 { 0x9800 } else { 0x9C00 };
        let mut line =  io_registers.read(LY_REGISTER).unwrap();
        let mut tile = if self.next_tile % TILES_IN_LINES == 0 && self.next_tile != 0 {
            self.next_tile = self.next_tile - TILES_IN_LINES;
            line += 1;
            io_registers.write(LY_REGISTER, line).unwrap();
        };
        if self.next_tile >= MAX_TILES{
            self.next_tile = 0;
        }
        let scx = io_registers.read((SCX_REGISTER)).unwrap() as u16;
        let scy = io_registers.read((SCY_REGISTER)).unwrap();
        let tile_map_address = bg_map_address + self.next_tile + (((line % LINES_TO_DRAW as u8).wrapping_add(scy) / 8) as u16) * 0x20;
        //println!("scy: {:#06x} | bg_map_address: {:#06x} | next_tile: {:#06x} | line: {} | tile_map_address: {:#06x}", scy, bg_map_address, self.next_tile, ((line.wrapping_add(scy) / 8) as u16), tile_map_address);
        self.current_tile_number = vram.read(tile_map_address).unwrap() as u16;
        self.next_tile = (self.next_tile +1);
        self.current_step = self.current_step.next();
    }

    //TODO: completly broken, fix pls
    fn read_data0(&mut self, vram: &Memory, io_registers: &mut Memory) {
        let lcd_control_register = io_registers.read(LCDC_REGISTER).unwrap();
        let mut line =  io_registers.read(LY_REGISTER).unwrap() as u16;
        let bg_tiles_address = if (lcd_control_register >> 4) & 1 == 1 { 0x8000 } else { 0x9000 };
        if bg_tiles_address == 0x8000 {
            self.fetched_color = (vram.read(0x8000 + ((line % 8) * 0x2) + (self.current_tile_number * 0x10) as u16).unwrap() as u16) << 8;
        } else if bg_tiles_address == 0x9000 {
            // Todo: Recheck later
            let mapped_tile = self.current_tile_number as i32;
            //println!("read data0 @ {:#06x}", (0x9000 + (mapped_tile * 0x10)) as u16);
            self.fetched_color = ((vram.read((0x9000 + (mapped_tile * 0x10)) as u16)).unwrap() as u16) << 8;
        } else { unimplemented!() }
        self.current_step = self.current_step.next();
    }

    fn read_tile1(&mut self, pixel_fifo: &mut PixelFifo, vram: &Memory, io_registers: &mut Memory) {
        let lcd_control_register = io_registers.read(LCDC_REGISTER).unwrap();
        let mut line =  io_registers.read(LY_REGISTER).unwrap() as u16;
        let bg_tiles_address = if (lcd_control_register >> 4) & 1 == 1 { 0x8000 } else { 0x9000 };
        if bg_tiles_address == 0x8000 {
            self.fetched_color = (vram.read(0x8001 + ((line % 8) * 0x2) + (self.current_tile_number as u16 * 0x10)).unwrap() as u16) << 8;
        } else if bg_tiles_address == 0x9000 {
            // Todo: Recheck later
            let mapped_tile = self.current_tile_number as i32;
            self.fetched_color |= (vram.read((0x9001 + (mapped_tile * 0x10)) as u16)).unwrap() as u16;
        } else { unimplemented!() }
        self.current_step = self.current_step.next();
        self.write_data(pixel_fifo);
    }

    fn write_data(&mut self, pixel_fifo: &mut PixelFifo) {
        if pixel_fifo.is_free() {
            //println!("push fifo");
            pixel_fifo.push(self.fetched_color);
            self.current_step = self.current_step.next();
        }
    }
}