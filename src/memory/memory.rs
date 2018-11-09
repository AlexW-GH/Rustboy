use std::fs::File;
use std::io::Read;
use memory::cartridge::ROM;


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

pub struct ReadOnly{memory: MemoryInternal}
pub struct WriteOnly{memory: MemoryInternal}
pub struct ReadWrite{memory: MemoryInternal}

pub trait MapsMemory{
    fn read(&self, address: u16) -> Result<u8, ()>;
    fn write(&mut self, address: u16, value: u8) -> Result<(), ()>;
    fn is_in_range(&self, address: u16) -> bool;
}

pub enum Memory{
    ReadOnly{memory: ReadOnly},
    WriteOnly{memory: WriteOnly},
    ReadWrite{memory: ReadWrite}
}

impl Memory{
    pub fn new_read_only(values: &[u8], from: u16, to: u16) -> Memory{
        let memory = ReadOnly{memory: MemoryInternal::new(values, from, to)};
        Memory::ReadOnly{memory}
    }

    pub fn new_write_only(values: &[u8], from: u16, to: u16) -> Memory{
        let memory = WriteOnly{memory: MemoryInternal::new(values, from, to)};
        Memory::WriteOnly{memory}
    }

    pub fn new_read_write(values: &[u8], from: u16, to: u16) -> Memory{
        let memory = ReadWrite{memory: MemoryInternal::new(values, from, to)};
        Memory::ReadWrite{memory}
    }
}

impl MapsMemory for Memory{
    fn read(&self, address: u16) -> Result<u8, ()>{
        match self{
            Memory::ReadOnly{memory} => Ok(memory.read(address)),
            Memory::ReadWrite{memory} => Ok(memory.read(address)),
            Memory::WriteOnly{memory} => Err(()),
        }
    }

    fn write(&mut self, address: u16, value: u8) -> Result<(), ()>{
        match self{
            Memory::WriteOnly{memory} => {
                memory.write(address, value);
                Ok(())
            },
            Memory::ReadWrite{memory} => {
                memory.write(address, value);
                Ok(())
            },
            Memory::ReadOnly{memory} => Err(())
        }
    }

    fn is_in_range(&self, address: u16) -> bool{
        let memory = match self{
            Memory::ReadOnly{memory} => &memory.memory,
            Memory::WriteOnly{memory} => &memory.memory,
            Memory::ReadWrite{memory} => &memory.memory
        };
        memory.from <= address && address <= memory.to
    }
}

impl ReadOnly{
    pub fn read(&self, address: u16) -> u8{
        assert!(self.memory.from <= address && address <= self.memory.to);
        let offset = self.memory.from;
        self.memory.read(address - offset)
    }
}

impl WriteOnly{
    pub fn write(&mut self, address: u16, value: u8){
        assert!(self.memory.from <= address && address <= self.memory.to);
        let offset = self.memory.from;
        self.memory.write(address - offset, value)
    }
}

impl ReadWrite{
    pub fn read(&self, address: u16) -> u8{
        assert!(self.memory.from <= address && address <= self.memory.to);
        let offset = self.memory.from;
        self.memory.read(address - offset)
    }

    pub fn write(&mut self, address: u16, value: u8){
        assert!(self.memory.from <= address && address <= self.memory.to);
        let offset = self.memory.from;
        self.memory.write(address - offset, value)
    }
}

struct MemoryInternal{
    from: u16,
    to: u16,
    memory: Vec<u8>
}

impl MemoryInternal{
    fn new(values: &[u8], from: u16, to: u16) -> MemoryInternal{
        assert!(from <= to);
        let mut memory = vec![0; ((to-from)+1) as usize];
        for (i, value) in values.iter().enumerate(){
            memory[i] = *value;
        }
        MemoryInternal{from, to, memory}
    }

    fn read(&self, address: u16) -> u8{
        let mem = self.memory[address as usize];
        mem
    }

    fn write(&mut self, address: u16, value: u8){
        self.memory[address as usize] = value;
    }
}