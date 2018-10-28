use rom::ROM;

const ROM_BANK_0_OFFSET: u16 = 0x0000;
const ROM_BANK_0_LAST: u16 = 0x3FFF ;
const ROM_BANK_1N_OFFSET: u16 = 0x4000;
const ROM_BANK_1N_LAST: u16 = 0x7FFF;
const VRAM_OFFSET: u16 = 0x8000;
const VRAM_LAST: u16 = 0x9FFF;
const RAM_OFFSET: u16 = 0xA000;
const RAM_LAST: u16 = 0xBFFF;
const WRAM_0_OFFSET: u16 = 0xC000;
const WRAM_0_LAST: u16 = 0xCFFF;
const WRAM_1N_OFFSET: u16 = 0xD000;
const WRAM_1N_LAST: u16 = 0xDFFF;
const ECHO_RAM_OFFSET: u16 = 0xE000;
const ECHO_RAM_LAST: u16 = 0xFDFF;
const SPRITE_ATTRIB_TABLE_OFFSET: u16 = 0xFE00;
const SPRITE_ATTRIB_TABLE_LAST: u16 = 0xFE9F;
const UNUSABLE_OFFSET: u16 = 0xFEA0;
const UNUSABLE_LAST: u16 = 0xFEFF;
const IO_REGISTERS_OFFSET: u16 = 0xFF00;
const IO_REGISTERS_LAST: u16 = 0xFF7F;
const HRAM_OFFSET: u16 = 0xFF80;
const HRAM_LAST: u16 = 0xFFFE;
const INTERRUPTS_ENABLE_REGISTER_OFFSET: u16 = 0xFFFF;

pub struct Memory {
    rom: ROM,
    vram: [u8; 0x2000],
    ram: [u8; 0x2000],
    wram_0: [u8; 0x1000],
    wram_1n: Vec<[u8; 0x1000]>,
    echo_ram: [u8; 0x1E00],
    sprite_attrib_table: [u8; 0x00A0],
    io_registers: [u8; 0x0080],
    hram: [u8; 0x007F],
    interrupts_enable_register: [u8; 0x0001],

    selected_rom_bank: usize,
    selected_wram_bank: usize,
}

impl Memory {
    pub fn new(rom: ROM) -> Memory {
        let mut wram_1n = Vec::new();
        wram_1n.push([0; 0x1000]);
        Memory {
            rom,
            vram: [0; 0x2000],
            ram: [0; 0x2000],
            wram_0: [0; 0x1000],
            wram_1n: wram_1n,
            echo_ram: [0; 0x1E00],
            sprite_attrib_table: [0; 0x00A0],
            io_registers: [0; 0x0080],
            hram: [0; 0x007F],
            interrupts_enable_register: [0; 0x0001],
            selected_rom_bank: 0,
            selected_wram_bank: 0,
        }
    }

    fn map_memory_area_mut(&mut self, address:u16) -> (&mut [u8], u16){
        match address{
            ROM_BANK_0_OFFSET ... ROM_BANK_0_LAST => unreachable!(),
            ROM_BANK_1N_OFFSET ... ROM_BANK_1N_LAST => unreachable!(),
            VRAM_OFFSET ... VRAM_LAST => (&mut self.vram, VRAM_OFFSET),
            RAM_OFFSET ... RAM_LAST => (&mut self.ram, RAM_OFFSET),
            WRAM_0_OFFSET ... WRAM_0_LAST => (&mut self.wram_0, WRAM_0_OFFSET),
            WRAM_1N_OFFSET ... WRAM_1N_LAST => (self.wram_1n.get_mut(self.selected_wram_bank).unwrap(), WRAM_1N_OFFSET),
            ECHO_RAM_OFFSET ... ECHO_RAM_LAST => (&mut self.echo_ram, ECHO_RAM_OFFSET),
            SPRITE_ATTRIB_TABLE_OFFSET ... SPRITE_ATTRIB_TABLE_LAST => (&mut self.sprite_attrib_table, SPRITE_ATTRIB_TABLE_OFFSET),
            UNUSABLE_OFFSET ... UNUSABLE_LAST => unreachable!(),
            IO_REGISTERS_OFFSET ... IO_REGISTERS_LAST => (&mut self.io_registers, IO_REGISTERS_OFFSET),
            HRAM_OFFSET ... HRAM_LAST => {
                (&mut self.hram, HRAM_OFFSET)
            },
            INTERRUPTS_ENABLE_REGISTER_OFFSET => (&mut self.interrupts_enable_register, INTERRUPTS_ENABLE_REGISTER_OFFSET),
            _ => unreachable!()
        }
    }

    fn map_memory_area(&self, address:u16) -> (&[u8], u16){
        match address{
            ROM_BANK_0_OFFSET ... ROM_BANK_0_LAST => (&self.rom.bank(0), ROM_BANK_0_OFFSET),
            ROM_BANK_1N_OFFSET ... ROM_BANK_1N_LAST => (&self.rom.bank(self.selected_rom_bank), ROM_BANK_1N_OFFSET),
            VRAM_OFFSET ... VRAM_LAST => (&self.vram, VRAM_OFFSET),
            RAM_OFFSET ... RAM_LAST => (&self.ram, RAM_OFFSET),
            WRAM_0_OFFSET ... WRAM_0_LAST => (&self.wram_0, WRAM_0_OFFSET),
            WRAM_1N_OFFSET ... WRAM_1N_LAST => (self.wram_1n.get(self.selected_wram_bank).unwrap(), WRAM_1N_OFFSET),
            ECHO_RAM_OFFSET ... ECHO_RAM_LAST => (&self.echo_ram, ECHO_RAM_OFFSET),
            SPRITE_ATTRIB_TABLE_OFFSET ... SPRITE_ATTRIB_TABLE_LAST => (&self.sprite_attrib_table, SPRITE_ATTRIB_TABLE_OFFSET),
            UNUSABLE_OFFSET ... UNUSABLE_LAST => unreachable!(),
            IO_REGISTERS_OFFSET ... IO_REGISTERS_LAST => (&self.io_registers, IO_REGISTERS_OFFSET),
            HRAM_OFFSET ... HRAM_LAST => {
                (&self.hram, HRAM_OFFSET)
            },
            INTERRUPTS_ENABLE_REGISTER_OFFSET => (&self.interrupts_enable_register, INTERRUPTS_ENABLE_REGISTER_OFFSET),
            _ => unreachable!()
        }
    }

    pub fn read(&self, address: u16) -> u8{
        let (memory_area, offset) = self.map_memory_area(address);
        *memory_area.get((address-offset) as usize).unwrap()
    }

    pub fn write(&mut self, address: u16, value: u8){
        let (memory_area, offset) = self.map_memory_area_mut(address);
        memory_area[(address-offset) as usize] = value;
    }

    pub fn following_u8(&self, address: u16) -> u8 {
        let (memory_area, offset) = self.map_memory_area(address);
        *memory_area.get((address-offset) as usize  + 1).unwrap()
    }

    pub fn following_u16(&self, address: u16) -> u16 {
        let (memory_area, offset) = self.map_memory_area(address);
        ((*memory_area.get((address-offset) as usize + 2).unwrap() as u16) << 8)  + *memory_area.get((address-offset) as usize + 1).unwrap() as u16
    }

}