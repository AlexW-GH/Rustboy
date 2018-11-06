use memory::memory::Memory;

const ROM_BANK_SIZE: usize = 0x4000;

pub struct CartridgeHeader{
    title: String,
    manufacturer: String,
    licensee_code: u8,
    old_licensee_code: u8,
    cartridge_type: CardridgeType,
    rom_size: RomSize,
    ram_size: RamSize,
    destination: String,
    version: u8,
    checksum: [u8; 24],
    global_checksum: u16
}

pub enum CardridgeType{
    MBCNone{ram: bool, battery: bool},
    MBC1{ram: bool, battery: bool},
    MBC2{battery: bool},
    MMM01{ram: bool, battery: bool},
    MBC3{timer: bool, ram: bool, battery: bool},
    MBC5{ram: bool, battery: bool, rumble: bool},
    MBC6,
    MBC7,
    PocketCamera,
    BandaiTama5,
    HuC3,
    HuC1
}

pub enum RomSize{
    KB32,
    KB64,
    KB128,
    KB256,
    KB512,
    MB1,
    MB2,
    MB4,
    MB8,
    MB1p1,
    MB1p2,
    MB1p5
}

pub enum RamSize{
    None,
    KB2,
    KB8,
    KB32,
    KB64,
    KB128,
}

struct MemoryBankController{}

impl MemoryBankController{
    pub fn create(header: CartridgeHeader) -> Box<MBC>{
        let rom_size = header.rom_size;
        let ram_size = header.ram_size;
        match header.cartridge_type{
            CardridgeType::MBCNone{ram, battery} => {
                Box::new(MBCNone::new(ram))
            },
            CardridgeType::MBC1{ram, battery} => {
                Box::new(MBC1::new(ram, rom_size, ram_size))
            },
            _ => unimplemented!()
        }
    }
}

trait MBC{
    fn read(&self, address: u16) -> Result<u8, ()>;
    fn write(&mut self, address: u16, value: u8) -> Result<(), ()>;
}

struct MBCNone {
    memory: Vec<Memory>
}

impl MBCNone{
    fn new(ram: bool) -> MBCNone {
        let mut memory = Vec::new();
        memory.push(Memory::new_read_only(0x0000, 0x7FFF));
        if ram {
            memory.push(Memory::new_read_write(0xA000, 0xBFFF));
        }
        MBCNone { memory }
    }
}

impl MBC for MBCNone {
    fn read(&self, address: u16) -> Result<u8, ()>{
        self.memory.iter()
            .find(|mem| mem.is_in_range(address))
            .map( |mem| mem.read(address))
            .unwrap()
    }

    fn write(&mut self, address: u16, value: u8) -> Result<(), ()>{
        self.memory.iter_mut()
            .find(|mem| mem.is_in_range(address))
            .map( |mem| mem.write(address, value))
            .unwrap()
    }
}

pub struct MBC1 {
    rom: Vec<Memory>,
    ram: Vec<Memory>,
    ram_enable: u8,
    rom_bank_number: u8,
    ram_bank_number: u8,
    mode_select: u8
}

impl MBC1{
    pub fn new(ram: bool, rom_size: RomSize, ram_size: RamSize) -> MBC1 {
        let mut memory_rom = Vec::new();
        let mut memory_ram = Vec::new();
        let rom_banks = match rom_size {
            RomSize::KB32 => 1,
            RomSize::KB64 => 4,
            RomSize::KB128 => 8,
            RomSize::KB256 => 16,
            RomSize::KB512 => 32,
            RomSize::MB1 => 63,
            RomSize::MB2 => 125,
            _ => unreachable!()
        };
        memory_rom.push(Memory::new_read_only(0x0000, 0x3FFF));
        for i in 1 .. rom_banks{
            memory_rom.push(Memory::new_read_only(0x4000, 0x7FFF));
        }
        let (ram_banks, bank_size) = if ram {
            match ram_size{
                RamSize::None => (0, 0),
                RamSize::KB2 => (1, 0x800),
                RamSize::KB8 => (1, 0x2000),
                RamSize::KB32 => (4, 0x2000),
                _ => unreachable!()
            }
        } else { (0,0) };
        for i in 0 .. ram_banks{
            memory_ram.push(Memory::new_read_write(0xA000, 0xA000+bank_size));
        }
        MBC1 { rom: memory_rom, ram: memory_ram, ram_enable: 0, rom_bank_number: 0, ram_bank_number: 0, mode_select: 0 }
    }
}

impl MBC for MBC1 {
    fn read(&self, address: u16) -> Result<u8, ()>{
        match address {
            0x0000 ... 0x3FFF => self.rom[0].read(address),
            0x4000 ... 0x7FFF => {
                let mut bank = self.rom_bank_number | 1;
                if self.mode_select == 0{
                    bank = (self.ram_enable << 5) | bank;
                }
                self.rom[bank as usize].read(address)
            },
            0xA000 ... 0xBFFF => {
                if (self.ram_enable & 0x0A) == 0x0A{
                    let bank = if self.mode_select == 1 {self.ram_bank_number} else {0};
                    self.ram[bank as usize].read(address)
                } else {
                    Err(())
                }

            },
            _ => unreachable!()
        }
    }

    fn write(&mut self, address: u16, value: u8) -> Result<(), ()>{
        match address{
            0x0000 ... 0x1FFF => {
                self.ram_enable = value;
                return Ok(());
            },
            0x2000 ... 0x3FFF => {
                self.rom_bank_number = value & 0b11111;
                return Ok(());
            },
            0x4000 ... 0x5FFF => {
                self.ram_bank_number = value & 0b11;
                return Ok(());
            },
            0x6000 ... 0x7FFF => {
                self.mode_select = value & 0b1;
                return Ok(());
            },
            _ => unreachable!()
        }
        let bank = self.ram_bank_number;
        self.ram[bank as usize].write(address, value)
    }
}

pub struct Cartridge{
    rom: ROM,
    mbc: MBCNone
}

pub struct ROM {
    data: Vec<u8>
}

impl ROM {
    pub fn new(game: Vec<u8>) -> ROM{
        let mut data = vec![0; ROM_BANK_SIZE*2];
        for (i, val) in game.iter().enumerate(){
            if i<ROM_BANK_SIZE*2 {
                data[i] = *val;
            } else {
                data.push(*val);
            }
        }
        ROM { data }
    }

    pub fn bank(&self, index: usize) -> &[u8]{

        let start_slice = if self.data.len() >= index*ROM_BANK_SIZE {
            index*ROM_BANK_SIZE
        } else {
            panic!("Memory Range {:#06x} - {:#06x} out of bounds",index*ROM_BANK_SIZE, (index+1)*ROM_BANK_SIZE )
        };
        let end_slice = if self.data.len() >= (index+1)*ROM_BANK_SIZE {
            (index+1)*ROM_BANK_SIZE
        } else {
            self.data.len()
        };
        &self.data[start_slice .. end_slice]
    }

    pub fn bank_mut(&mut self, index: usize) -> &mut [u8]{
        let start_slice = if self.data.len() >= index*ROM_BANK_SIZE {
            index*ROM_BANK_SIZE
        } else {
            panic!("Memory Range {:#06x} - {:#06x} out of bounds",index*ROM_BANK_SIZE, (index+1)*ROM_BANK_SIZE )
        };
        let end_slice = if self.data.len() >= (index+1)*ROM_BANK_SIZE {
            index+1*ROM_BANK_SIZE
        } else {
            self.data.len()
        };
        self.data[start_slice .. end_slice].as_mut()
    }
}