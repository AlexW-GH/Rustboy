#[derive(Debug)]
pub struct ReadOnly {
    memory: MemoryInternal,
}
#[derive(Debug)]
pub struct ReadWrite {
    memory: MemoryInternal,
}

pub trait MapsMemory {
    fn read(&self, address: u16) -> Result<u8, ()>;
    fn write(&mut self, address: u16, value: u8) -> Result<(), ()>;
    fn is_in_range(&self, address: u16) -> bool;
}

#[derive(Debug)]
pub enum Memory {
    ReadOnly { memory: ReadOnly },
    ReadWrite { memory: ReadWrite },
}

impl Memory {
    pub fn new_read_only(values: &[u8], from: u16, to: u16) -> Memory {
        let memory = ReadOnly {
            memory: MemoryInternal::new(values, from, to),
        };
        Memory::ReadOnly { memory }
    }

    pub fn new_read_write(values: &[u8], from: u16, to: u16) -> Memory {
        let memory = ReadWrite {
            memory: MemoryInternal::new(values, from, to),
        };
        Memory::ReadWrite { memory }
    }
}

impl MapsMemory for Memory {
    fn read(&self, address: u16) -> Result<u8, ()> {
        match self {
            Memory::ReadOnly { memory } => Ok(memory.read(address)),
            Memory::ReadWrite { memory } => Ok(memory.read(address)),
        }
    }

    fn write(&mut self, address: u16, value: u8) -> Result<(), ()> {
        match self {
            Memory::ReadWrite { memory } => {
                memory.write(address, value);
                Ok(())
            }
            Memory::ReadOnly { .. } => Err(()),
        }
    }

    fn is_in_range(&self, address: u16) -> bool {
        let memory = match self {
            Memory::ReadOnly { memory } => &memory.memory,
            Memory::ReadWrite { memory } => &memory.memory,
        };
        memory.from <= address && address <= memory.to
    }
}

impl ReadOnly {
    pub fn read(&self, address: u16) -> u8 {
        assert!(self.memory.from <= address && address <= self.memory.to);
        let offset = self.memory.from;
        self.memory.read(address - offset)
    }
}

impl ReadWrite {
    pub fn read(&self, address: u16) -> u8 {
        assert!(self.memory.from <= address && address <= self.memory.to);
        let offset = self.memory.from;
        self.memory.read(address - offset)
    }

    pub fn write(&mut self, address: u16, value: u8) {
        assert!(self.memory.from <= address && address <= self.memory.to);
        let offset = self.memory.from;
        self.memory.write(address - offset, value)
    }
}

#[derive(Debug)]
struct MemoryInternal {
    from: u16,
    to: u16,
    memory: Vec<u8>,
}

impl MemoryInternal {
    fn new(values: &[u8], from: u16, to: u16) -> MemoryInternal {
        assert!(from <= to);
        let mut memory = vec![0; ((to - from) + 1) as usize];
        for (i, value) in values.iter().enumerate() {
            memory[i] = *value;
        }
        MemoryInternal { from, to, memory }
    }

    fn read(&self, address: u16) -> u8 {
        self.memory[address as usize]
    }

    fn write(&mut self, address: u16, value: u8) {
        self.memory[address as usize] = value;
    }
}
