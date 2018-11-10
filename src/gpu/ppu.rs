use memory::memory::Memory;
use memory::memory::MapsMemory;

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

pub struct PixelProcessingUnit{
    memory: Memory
}

impl PixelProcessingUnit {
    pub fn new() -> PixelProcessingUnit{
        let memory = Memory::new_read_write(&[0u8; 0], 0x8000, 0x9FFF);
        PixelProcessingUnit{memory}
    }
}

impl MapsMemory for PixelProcessingUnit{
    fn read(&self, address: u16) -> Result<u8, ()> {
        self.memory.read(address)
    }

    fn write(&mut self, address: u16, value: u8) -> Result<(), ()> {
        self.memory.write(address, value)
    }

    fn is_in_range(&self, address: u16) -> bool{
        self.memory.is_in_range(address)
    }
}