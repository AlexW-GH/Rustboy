use super::memory::{MapsMemory, Memory};

#[derive(Debug, Clone)]
struct CartridgeHeader {
    title: String,
    manufacturer: String,
    licensee_code: u16,
    old_licensee_code: u8,
    cartridge_type: CartridgeType,
    rom_size: RomSize,
    ram_size: RamSize,
    destination: String,
    version: u8,
    checksum: u8,
    global_checksum: u16,
}

impl CartridgeHeader {
    pub fn new(rom: &[u8]) -> CartridgeHeader {
        let title = Self::extract_title(rom);
        let manufacturer = String::new();
        let licensee_code = (u16::from(rom[0x144]) << 8) + u16::from(rom[0x145]);
        let cartridge_type = CartridgeType::new(rom[0x147]);
        let rom_size = RomSize::new(rom[0x148]);
        let ram_size = RamSize::new(rom[0x149]);
        let destination = String::new();
        let old_licensee_code = rom[0x14B];
        let version = rom[0x14C];
        let checksum = rom[0x14D];
        let global_checksum = (u16::from(rom[0x14E]) << 8) + u16::from(rom[0x14F]);
        let header = CartridgeHeader {
            title,
            manufacturer,
            licensee_code,
            old_licensee_code,
            cartridge_type,
            rom_size,
            ram_size,
            destination,
            version,
            checksum,
            global_checksum,
        };
        info!("Load Rom: {:?}", header);

        header
    }

    fn extract_title(rom: &[u8]) -> String {
        let mut title = Vec::new();
        for &character in rom.iter().take(0x144).skip(0x134) {
            if character == 0 {
                break;
            }
            title.push(character)
        }
        String::from_utf8(title).unwrap_or_else(|_| "".to_string())
    }
}

#[derive(Copy, Clone, Debug)]
enum CartridgeType {
    MBCNone {
        ram: bool,
        battery: bool,
    },
    MBC1 {
        ram: bool,
        battery: bool,
    },
    MBC2 {
        battery: bool,
    },
    MMM01 {
        ram: bool,
        battery: bool,
    },
    MBC3 {
        timer: bool,
        ram: bool,
        battery: bool,
    },
}

impl CartridgeType {
    pub fn new(code: u8) -> CartridgeType {
        match code {
            0x00 => CartridgeType::MBCNone {
                ram: false,
                battery: false,
            },
            0x01 => CartridgeType::MBC1 {
                ram: false,
                battery: false,
            },
            0x02 => CartridgeType::MBC1 {
                ram: true,
                battery: false,
            },
            0x03 => CartridgeType::MBC1 {
                ram: true,
                battery: true,
            },
            0x05 => CartridgeType::MBC2 { battery: false },
            0x06 => CartridgeType::MBC2 { battery: true },
            0x0C => CartridgeType::MMM01 {
                ram: true,
                battery: false,
            },
            0x0D => CartridgeType::MMM01 {
                ram: true,
                battery: true,
            },
            0x0F => CartridgeType::MBC3 {
                timer: true,
                ram: false,
                battery: true,
            },
            0x10 => CartridgeType::MBC3 {
                timer: true,
                ram: true,
                battery: true,
            },
            0x11 => CartridgeType::MBC3 {
                timer: false,
                ram: false,
                battery: false,
            },
            0x12 => CartridgeType::MBC3 {
                timer: false,
                ram: true,
                battery: false,
            },
            0x13 => CartridgeType::MBC3 {
                timer: false,
                ram: true,
                battery: true,
            },
            _ => panic!("Cartridge type unsupported"),
        }
    }
}

#[derive(Copy, Clone, Debug)]
enum RomSize {
    KB32,
    KB64,
    KB128,
    KB256,
    KB512,
    KB1024,
    KB2048,
    KB4096,
    KB8192,
}

impl RomSize {
    pub fn new(code: u8) -> RomSize {
        match code {
            0x00 => RomSize::KB32,
            0x01 => RomSize::KB64,
            0x02 => RomSize::KB128,
            0x03 => RomSize::KB256,
            0x04 => RomSize::KB512,
            0x05 => RomSize::KB1024,
            0x06 => RomSize::KB2048,
            0x07 => RomSize::KB4096,
            0x08 => RomSize::KB8192,
            _ => panic!("Rom size not supported"),
        }
    }
}

#[derive(Copy, Clone, Debug)]
enum RamSize {
    None,
    KB2,
    KB8,
    KB32,
    KB64,
    KB128,
}

impl RamSize {
    pub fn new(code: u8) -> RamSize {
        match code {
            0x00 => RamSize::None,
            0x01 => RamSize::KB2,
            0x02 => RamSize::KB8,
            0x03 => RamSize::KB32,
            0x04 => RamSize::KB128,
            0x05 => RamSize::KB64,
            _ => panic!("Ram size not supported"),
        }
    }
}

struct MemoryBankController {}

impl MemoryBankController {
    pub fn create_rom_memory(rom: &Rom) -> Box<dyn MapsMemory + Send> {
        let rom_size = rom.header.rom_size;
        let ram_size = rom.header.ram_size;
        let rom_ref = &rom;
        match rom_ref.header.cartridge_type {
            CartridgeType::MBCNone { ram, .. } => MbcNone::new(&rom, ram),
            CartridgeType::MBC1 { ram, .. } => Mbc1::new(&rom, ram, rom_size, ram_size),
            _ => unimplemented!(),
        }
    }
}

struct MbcNone {
    memory: Vec<Memory>,
}

impl MbcNone {
    fn new(rom: &Rom, ram: bool) -> Box<MbcNone> {
        let mut memory = Vec::new();
        let end = if rom.data.len() >= 0x7FFF {
            0x7FFF
        } else {
            rom.data.len()
        };
        memory.push(Memory::new_read_only(
            &rom.data[0x0000..=end],
            0x0000,
            0x7FFF,
        ));
        if ram {
            memory.push(Memory::new_read_write(&[0u8; 0], 0xA000, 0xBFFF));
        }
        Box::new(MbcNone { memory })
    }
}

impl MapsMemory for MbcNone {
    fn read(&self, address: u16) -> Result<u8, ()> {
        self.memory
            .iter()
            .find(|mem| mem.is_in_range(address))
            .map(|mem| mem.read(address))
            .unwrap()
    }

    fn write(&mut self, _address: u16, _value: u8) -> Result<(), ()> {
        Ok(())
    }

    fn is_in_range(&self, address: u16) -> bool {
        self.memory.iter().any(|mem| mem.is_in_range(address))
    }
}

struct Mbc1 {
    rom: Vec<Memory>,
    ram: Vec<Memory>,
    ram_enable: u8,
    rom_bank_number: u8,
    ram_bank_number: u8,
    mode_select: u8,
}

impl Mbc1 {
    pub fn new(rom: &Rom, ram: bool, rom_size: RomSize, ram_size: RamSize) -> Box<Mbc1> {
        let mut memory_rom = Vec::new();
        let mut memory_ram = Vec::new();
        let rom_banks = match rom_size {
            RomSize::KB32 => 1,
            RomSize::KB64 => 4,
            RomSize::KB128 => 8,
            RomSize::KB256 => 16,
            RomSize::KB512 => 32,
            RomSize::KB1024 => 63,
            RomSize::KB2048 => 125,
            _ => unreachable!(),
        };
        let end = if rom.data.len() >= 0x3FFF {
            0x3FFF
        } else {
            rom.data.len() - 1
        };
        let bank0 = Memory::new_read_only(&rom.data[0x0000..=end], 0x0000, 0x3FFF);
        memory_rom.push(bank0);
        for i in 1..=rom_banks {
            let start = if rom.data.len() >= 0x4000 * i {
                0x4000 * i
            } else {
                rom.data.len()
            };
            let end = if rom.data.len() >= start + 0x3FFF {
                start + 0x3FFF
            } else {
                rom.data.len() - 1
            };
            let banki = Memory::new_read_only(&rom.data[start..=end], 0x4000, 0x7FFF);
            memory_rom.push(banki);
        }
        let (ram_banks, bank_size) = if ram {
            match ram_size {
                RamSize::None => (0, 0),
                RamSize::KB2 => (1, 0x800),
                RamSize::KB8 => (1, 0x2000),
                RamSize::KB32 => (4, 0x2000),
                _ => unreachable!(),
            }
        } else {
            (0, 0)
        };
        for _ in 0..ram_banks {
            memory_ram.push(Memory::new_read_write(
                &[0u8; 0],
                0xA000,
                0xA000 + bank_size,
            ));
        }
        Box::new(Mbc1 {
            rom: memory_rom,
            ram: memory_ram,
            ram_enable: 0,
            rom_bank_number: 1,
            ram_bank_number: 0,
            mode_select: 0,
        })
    }
}

impl MapsMemory for Mbc1 {
    fn read(&self, address: u16) -> Result<u8, ()> {
        match address {
            0x0000..=0x3FFF => self.rom[0].read(address),
            0x4000..=0x7FFF => {
                let mut bank = self.rom_bank_number;
                if self.mode_select == 0 {
                    bank |= self.ram_bank_number << 5;
                }
                self.rom[bank as usize].read(address)
            }
            0xA000..=0xBFFF => {
                if (self.ram_enable & 0x0A) == 0x0A {
                    let bank = if self.mode_select == 1 {
                        self.ram_bank_number
                    } else {
                        0
                    };
                    self.ram[bank as usize].read(address)
                } else {
                    Err(())
                }
            }
            _ => unreachable!(),
        }
    }

    fn write(&mut self, address: u16, value: u8) -> Result<(), ()> {
        match address {
            0x0000..=0x1FFF => {
                self.ram_enable = value;
                Ok(())
            }
            0x2000..=0x3FFF => {
                self.rom_bank_number = value & 0b11111;
                if self.rom_bank_number == 0 {
                    self.rom_bank_number = 1
                };
                Ok(())
            }
            0x4000..=0x5FFF => {
                self.ram_bank_number = value & 0b11;
                Ok(())
            }
            0x6000..=0x7FFF => {
                self.mode_select = value & 0b1;
                Ok(())
            }
            _ => {
                let bank = self.ram_bank_number;
                self.ram[bank as usize].write(address, value)
            }
        }
    }

    fn is_in_range(&self, address: u16) -> bool {
        let rom = self.rom.iter().any(|mem| mem.is_in_range(address));
        let ram = self.ram.iter().any(|mem| mem.is_in_range(address));
        ram || rom
    }
}

pub struct Cartridge {
    mbc: Box<dyn MapsMemory + Send>,
    header: CartridgeHeader,
}

impl Cartridge {
    pub fn new(game: Vec<u8>) -> Cartridge {
        let mut data = Vec::new();
        for val in game {
            data.push(val);
        }
        let header = CartridgeHeader::new(&data);
        let mbc = MemoryBankController::create_rom_memory(&Rom {
            data,
            header: header.clone(),
        });
        Cartridge { mbc, header }
    }

    pub fn title(&self) -> &str {
        &self.header.title
    }
}

impl MapsMemory for Cartridge {
    fn read(&self, address: u16) -> Result<u8, ()> {
        self.mbc.read(address)
    }

    fn write(&mut self, address: u16, value: u8) -> Result<(), ()> {
        self.mbc.write(address, value)
    }

    fn is_in_range(&self, address: u16) -> bool {
        self.mbc.is_in_range(address)
    }
}

struct Rom {
    data: Vec<u8>,
    header: CartridgeHeader,
}
