use std::sync::Arc;
use std::sync::RwLock;
use memory::Memory;

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

struct PixelProcessingUnit{
    memory: Arc<RwLock<Memory>>
}

impl PixelProcessingUnit {}