use std::fs::File;
use std::io::Read;

use rom::ROM;
use registers::Registers;

const BOOT_OFFSET: u16 = 0x0000;
const BOOT_LAST: u16 = 0x00FF;
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

const BOOT_ADDRESS: u16 = 0xFF50;

pub struct Memory {
    rom: ROM,
    boot: Vec<u8>,
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
    boot_sequence: bool
}

impl Memory {
    pub fn new(rom: ROM, boot_sequence: bool) -> Memory {
        let boot = Self::read_boot_data(boot_sequence);
        let mut wram_1n = Vec::new();
        wram_1n.push([0; 0x1000]);
        wram_1n.push([0; 0x1000]);
        let memory =Memory {
            rom,
            boot,
            vram: [0; 0x2000],
            ram: [0; 0x2000],
            wram_0: [0; 0x1000],
            wram_1n,
            echo_ram: [0; 0x1E00],
            sprite_attrib_table: [0; 0x00A0],
            io_registers: [0; 0x0080],
            hram: [0; 0x007F],
            interrupts_enable_register: [0; 0x0001],
            selected_rom_bank: 1,
            selected_wram_bank: 1,
            boot_sequence,
        };
        Self::init_memory(memory, boot_sequence)
    }

    fn init_memory(mut memory: Memory, boot_sequence: bool) -> Memory{
        if !boot_sequence {
            memory.write(0xFF05, 0x00);
            memory.write(0xFF06, 0x00);
            memory.write(0xFF07, 0x00);
            memory.write(0xFF10, 0x80);
            memory.write(0xFF11, 0xBF);
            memory.write(0xFF12, 0xF3);
            memory.write(0xFF14, 0xBF);
            memory.write(0xFF16, 0x3F);
            memory.write(0xFF17, 0x00);
            memory.write(0xFF19, 0xBF);
            memory.write(0xFF1A, 0x7F);
            memory.write(0xFF1B, 0xFF);
            memory.write(0xFF1C, 0x9F);
            memory.write(0xFF1E, 0xBF);
            memory.write(0xFF20, 0xFF);
            memory.write(0xFF21, 0x00);
            memory.write(0xFF22, 0x00);
            memory.write(0xFF23, 0xBF);
            memory.write(0xFF24, 0x77);
            memory.write(0xFF25, 0xF3);
            memory.write(0xFF26, 0xF1);
            memory.write(0xFF40, 0x91);
            memory.write(0xFF42, 0x00);
            memory.write(0xFF43, 0x00);
            memory.write(0xFF45, 0x00);
            memory.write(0xFF47, 0xFC);
            memory.write(0xFF48, 0xFF);
            memory.write(0xFF49, 0xFF);
            memory.write(0xFF4A, 0x00);
            memory.write(0xFF4B, 0x00);
            memory.write(0xFFFF, 0x00);
        }
        memory
    }

    fn map_memory_area_mut(&mut self, address:u16) -> (&mut [u8], u16){
        match address{
            ROM_BANK_0_OFFSET ... ROM_BANK_0_LAST =>(self.rom.bank_mut(0), ROM_BANK_0_OFFSET),
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
            BOOT_OFFSET ... BOOT_LAST if self.boot_sequence == true => (&self.boot, BOOT_OFFSET),
            ROM_BANK_0_OFFSET ... ROM_BANK_0_LAST => (self.rom.bank(0), ROM_BANK_0_OFFSET),
            ROM_BANK_1N_OFFSET ... ROM_BANK_1N_LAST => (self.rom.bank(self.selected_rom_bank), ROM_BANK_1N_OFFSET),
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

    pub fn push_u16_stack(&mut self, value: u16, sp: u16){
        self.write(sp-1, ((value>>8) & 0xFF) as u8 );
        self.write(sp-2, (value & 0xFF) as u8 );
    }

    pub fn pop_u16_stack(&self, sp: u16) -> u16{
        let val_lo = self.read(sp);
        let val_hi = self.read(sp + 1);
        ((val_hi as u16) << 8) + val_lo as u16
    }

    pub fn read(&self, address: u16) -> u8{
        let (memory_area, offset) = self.map_memory_area(address);
        *memory_area.get((address-offset) as usize).unwrap()
    }

    pub fn write(&mut self, address: u16, value: u8){
        match address{
            BOOT_ADDRESS => if value != 0 { self.boot_sequence = false} else { self.boot_sequence = true }
            0xFF02 => if value == 0x81 {
                let (memory_area, offset) = self.map_memory_area(address);
                print!("{}", *memory_area.get((0xFF01-offset) as usize).unwrap() as char)
            }
            _ => {
                let (memory_area, offset) = self.map_memory_area_mut(address);
                memory_area[(address-offset) as usize] = value;
            }
        }

    }

    pub fn following_u8(&self, address: u16) -> u8 {
        let (memory_area, offset) = self.map_memory_area(address);
        *memory_area.get((address-offset) as usize  + 1).unwrap()
    }

    pub fn following_u16(&self, address: u16) -> u16 {
        let (memory_area, offset) = self.map_memory_area(address);
        ((*memory_area.get((address-offset) as usize + 2).unwrap() as u16) << 8)  + *memory_area.get((address-offset) as usize + 1).unwrap() as u16
    }

    fn read_boot_data(boot_sequence: bool) -> Vec<u8>{
        let mut game = Vec::new();
        if boot_sequence {
            let mut file = File::open("assets/boot.gb").expect("Boot Sequence not found");
            file.read_to_end(&mut game).expect("something went wrong reading the file");
        }
        game
    }

}