use crate::processor::registers::Registers;
use crate::processor::interrupt_controller::InterruptController;
use crate::processor::opcodes;
use crate::memory::memory::{Memory, MapsMemory};
use crate::memory::cartridge::Cartridge;
use crate::gpu::ppu::PixelProcessingUnit;
use crate::gpu::lcd::LCDFetcher;
use std::sync::Mutex;
use std::sync::Arc;
use std::rc::Rc;
use std::cell::RefCell;

pub struct CPU{
    pub registers: Registers,
    pub interrupt: InterruptController,

    memory: Vec<Memory>,
    io_registers: Memory,
    boot_rom: Option<Memory>,
    ppu: PixelProcessingUnit,
    cartridge: Cartridge,

    cpu_wait_cycles: i64
}

impl CPU {
    pub fn new(interrupt: InterruptController, cartridge: Cartridge, lcd_fetcher: Rc<RefCell<LCDFetcher>>, boot_rom: Option<Vec<u8>>) -> CPU {

        let boot_rom = match boot_rom {
            Some(opcodes) => Some(Memory::new_read_only(&opcodes, 0x0000, opcodes.len() as u16)),
            None => None
        };
        let boot_sequence = boot_rom.is_some();
        let memory = Self::init_memory();
        let io_registers = Memory::new_read_write(&[0u8; 0], 0xFF00, 0xFF7F);
        let ppu = PixelProcessingUnit::new(lcd_fetcher);
        let cpu_wait_cycles = 0;
        let mut cpu = CPU { registers: Registers::new(boot_sequence), interrupt, memory, io_registers, boot_rom, ppu, cartridge, cpu_wait_cycles };
        cpu.init_boot_state(boot_sequence);
        cpu
    }

    pub fn step (&mut self){
        {
            let io_registers = &mut self.io_registers;
            let interrupt = &mut self.interrupt;
            self.ppu.step(io_registers, interrupt);
        }
        if self.cpu_wait_cycles <= 0 {
            let pc = self.registers.pc();
            let opcode = self.read(pc).unwrap();
            //println!("pc: {:#06x} | opcode: {:#04x}", pc, opcode);
            self.cpu_wait_cycles += opcodes::execute(opcode, pc, self) as i64;
        }
        self.cpu_wait_cycles -= 1;
    }

    pub fn init_memory() -> Vec<Memory> {
        let mut memory = Vec::new();
        memory.push(Memory::new_read_write(&[0u8; 0], 0xC000, 0xDFFF));
        memory.push(Memory::new_read_write(&[0u8; 0], 0xE000, 0xFDFF));
        memory.push(Memory::new_read_write(&[0u8; 0], 0xFF80, 0xFFFE));
        memory.push(Memory::new_read_write(&[0u8; 0], 0xFFFF, 0xFFFF));
        memory
    }

    fn init_boot_state(&mut self, boot_sequence: bool){
        use crate::util::memory_op::write_memory;
        if !boot_sequence {
            write_memory(self, 0xFF05, 0x00);
            write_memory(self, 0xFF06, 0x00);
            write_memory(self, 0xFF07, 0x00);
            write_memory(self, 0xFF10, 0x80);
            write_memory(self, 0xFF11, 0xBF);
            write_memory(self, 0xFF12, 0xF3);
            write_memory(self, 0xFF14, 0xBF);
            write_memory(self, 0xFF16, 0x3F);
            write_memory(self, 0xFF17, 0x00);
            write_memory(self, 0xFF19, 0xBF);
            write_memory(self, 0xFF1A, 0x7F);
            write_memory(self, 0xFF1B, 0xFF);
            write_memory(self, 0xFF1C, 0x9F);
            write_memory(self, 0xFF1E, 0xBF);
            write_memory(self, 0xFF20, 0xFF);
            write_memory(self, 0xFF21, 0x00);
            write_memory(self, 0xFF22, 0x00);
            write_memory(self, 0xFF23, 0xBF);
            write_memory(self, 0xFF24, 0x77);
            write_memory(self, 0xFF25, 0xF3);
            write_memory(self, 0xFF26, 0xF1);
            write_memory(self, 0xFF40, 0x91);
            write_memory(self, 0xFF42, 0x00);
            write_memory(self, 0xFF43, 0x00);
            write_memory(self, 0xFF45, 0x00);
            write_memory(self, 0xFF47, 0xFC);
            write_memory(self, 0xFF48, 0xFF);
            write_memory(self, 0xFF49, 0xFF);
            write_memory(self, 0xFF4A, 0x00);
            write_memory(self, 0xFF4B, 0x00);
            write_memory(self, 0xFFFF, 0x00);
        }
    }
}

impl MapsMemory for CPU {
    fn read(&self, address: u16) -> Result<u8, ()> {
        if let Some(boot) = &self.boot_rom{
            if boot.is_in_range(address){
                return boot.read(address);
            }
        }
        let read = self.memory.iter()
            .find(|mem| mem.is_in_range(address))
            .map( |mem| mem.read(address))
            .unwrap_or_else(|| Err(()));
        if read.is_err() {
            if self.ppu.is_in_range(address) {
                self.ppu.read(address)
            } else if self.cartridge.is_in_range(address){
                self.cartridge.read(address)
            } else if self.io_registers.is_in_range(address) {
                self.io_registers.read(address)
            } else {
                if address >= 0xFEA0 && address <= 0xFEFF{
                    Ok(0xFF)
                } else {
                    Err(())
                }
            }
        } else{
            read
        }
    }

    fn write(&mut self, address: u16, value: u8) -> Result<(), ()>{
        let write = self.memory.iter_mut()
            .find(|mem| mem.is_in_range(address))
            .map( |mem| mem.write(address, value))
            .unwrap_or_else(|| Err(()));
        if write.is_ok() {
            if (address >= 0xE000) && (address <= 0xFDFF){
                self.write(address - 0x2000, value).unwrap();
            }
            write
        } else {
            if self.ppu.is_in_range(address) {
                self.ppu.write(address, value)
            } else if self.cartridge.is_in_range(address){
                self.cartridge.write(address, value)
            } else if self.io_registers.is_in_range(address) {
                if address == 0xFF50 {
                    self.boot_rom = None;
                }
                self.io_registers.write(address, value)
            } else {
                if address >= 0xFEA0 && address <= 0xFEFF{
                    Ok(())
                } else {
                    Err(())
                }
            }
        }
    }

    fn is_in_range(&self, address: u16) -> bool{
        let mut read = self.memory.iter()
            .find(|mem| mem.is_in_range(address))
            .is_some();
        read |= self.cartridge.is_in_range(address);
        read |= self.ppu.is_in_range(address);
        read
    }
}

#[cfg(test)]
mod tests {
    use simplelog;
    use std::sync::Arc;
    use log::LevelFilter;
    use simplelog::Config;
    use simplelog::TestLogger;
    use crate::processor::cpu::CPU;
    use crate::processor::interrupt_controller::InterruptController;
    use crate::memory::cartridge::Cartridge;
    use crate::util::memory_op::*;
    use crate::gpu::lcd::LCDFetcher;
    use std::sync::Mutex;
    use std::rc::Rc;
    use std::cell::RefCell;

    fn create_cpu(rom: Vec<u8>) -> CPU {
        let logger = TestLogger::init(LevelFilter::Debug, Config::default());
        if logger.is_ok() {
            logger.unwrap();
        }
        let interrupt = InterruptController::new();
        let rom = add_header(rom);
        let cartridge = Cartridge::new(rom);
        let lcd_fetcher = Rc::new(RefCell::new(LCDFetcher::new()));
        let mut cpu = CPU::new(interrupt, cartridge, lcd_fetcher, None);
        cpu.registers.set_pc(0);
        cpu.registers.set_f(0x0);
        cpu
    }

    fn add_header(rom: Vec<u8>) -> Vec<u8> {
        let mut with_header = vec![0u8; 0x150];
        with_header[0x147] = 0x01;
        with_header[0x148] = 0x00;
        with_header[0x149] = 0x00;
        for (i, byte) in rom.iter().enumerate() {
            with_header[i] = *byte;
        }
        with_header
    }

    fn run_steps_without_wait_cycles(steps: usize, cpu: &mut CPU) {
        for _ in 0..steps {
            cpu.step();
            cpu.cpu_wait_cycles = 0;
        }
    }

    #[test]
    fn ld_a_b() {
        let rom = vec![
            0b00_111_110,
            0b00_000_001,   // LD A, 1
            0b00_000_110,
            0b00_000_010,   // LD B, 2
            0b01_111_000,    // LD A, B
        ];
        let mut cpu = create_cpu(rom);
        run_steps_without_wait_cycles(3, &mut cpu);
        let registers = &cpu.registers;
        assert_eq!(registers.a(), 0b00000010);
        assert_eq!(registers.pc(), 5);
    }

    #[test]
    fn ld_b_d() {
        let rom = vec![
            0b00_000_110,
            0b00_000_001,   // LD B, 1
            0b00_010_110,
            0b00_000_010,   // LD D, 2
            0b01_000_010,   // LD B, D
        ];
        let mut cpu = create_cpu(rom);
        run_steps_without_wait_cycles(3, &mut cpu);
        let registers = &cpu.registers;
        assert_eq!(registers.b(), 0b00000010);
        assert_eq!(registers.pc(), 5);
    }

    #[test]
    fn ld_b_24() {
        let rom = vec![
            0b00_000_110,
            0b00_011_000,   // LD B, 24
        ];
        let mut cpu = create_cpu(rom);
        run_steps_without_wait_cycles(1, &mut cpu);
        let registers = &cpu.registers;
        assert_eq!(registers.b(), 0b00011000);
        assert_eq!(registers.pc(), 2);
    }

    #[test]
    fn ld_h_mem_hl() {
        let rom = vec![
            0b00_101_110,
            0x00,         // LD L, 0x00
            0b00_100_110,
            0x80,         // LD H, 0x00
            0b00_110_110,
            0x5C,         // LD HL, 24
            0b01_100_110  // LD H, (HL)
        ];
        let mut cpu = create_cpu(rom);
        run_steps_without_wait_cycles(4, &mut cpu);
        let registers = &cpu.registers;
        assert_eq!(registers.h(), 0x5C);
        assert_eq!(registers.pc(), 7);
    }

    #[test]
    fn ld_mem_hl_a() {
        let rom = vec![
            0b00_111_110,
            0x3C,         // LD A, 0x3C
            0b00_100_001,
            0xC5,
            0x8A,         // LD HL, 0x8AC5
            0b01_110_111  // LD (HL), A
        ];
        let mut cpu = create_cpu(rom);
        run_steps_without_wait_cycles(3, &mut cpu);
        let registers = &cpu.registers;
        assert_eq!(read_memory(&cpu, 0x8AC5), 0x3C);
        assert_eq!(registers.pc(), 6);
    }

    #[test]
    fn ld_mem_hl_24() {
        let rom = vec![
            0b00_100_001,
            0xC5,
            0x8A,         // LD HL, 0x8AC5
            0b_00_110_110,
            13              // LD (HL), 13
        ];
        let mut cpu = create_cpu(rom);
        run_steps_without_wait_cycles(2, &mut cpu);
        let registers = &cpu.registers;
        assert_eq!(read_memory(&cpu, 0x8AC5), 13);
        assert_eq!(registers.pc(), 5);
    }

    #[test]
    fn ld_a_mem_bc() {
        let rom = vec![
            0b00_100_001,
            0xC5,
            0x8A,          // LD HL, 0x8AC5
            0b_00_110_110,
            0x2F,            // LD (HL), 0x2F
            0b00_000_001,
            0xC5,
            0x8A,          // LD BC, 0x8AC5
            0b00_001_010   // LD A, (BC)

        ];
        let mut cpu = create_cpu(rom);
        run_steps_without_wait_cycles(4, &mut cpu);
        let registers = &cpu.registers;
        assert_eq!(registers.a(), 0x2F);
        assert_eq!(registers.pc(), 9);
    }

    #[test]
    fn ld_a_mem_de() {
        let rom = vec![
            0b00_100_001,
            0xC5,
            0x8A,          // LD HL, 0x8AC5
            0b_00_110_110,
            0x5F,            // LD (HL), 0x5F
            0b00_010_001,
            0xC5,
            0x8A,          // LD DE, 0x8AC5
            0b00_011_010   // LD A, (DE)

        ];
        let mut cpu = create_cpu(rom);
        run_steps_without_wait_cycles(4, &mut cpu);
        let registers = &cpu.registers;
        assert_eq!(registers.a(), 0x5F);
        assert_eq!(registers.pc(), 9);
    }

    #[test]
    fn ld_a_mem_c() {
        let rom = vec![
            0b00_100_001,
            0x95,
            0xFF,          // LD HL, 0xFF95
            0b_00_110_110,
            0x21,          // LD (HL), 0x21
            0b00_001_110,
            0x95,          // LD C, 0x95
            0b11_110_010   // LD A, (C)

        ];
        let mut cpu = create_cpu(rom);
        run_steps_without_wait_cycles(4, &mut cpu);
        let registers = &cpu.registers;
        assert_eq!(registers.a(), 0x21);
        assert_eq!(registers.pc(), 8);
    }

    #[test]
    fn ld_mem_c_a() {
        let rom = vec![
            0b00_111_110,
            0x3C,         // LD A, 0x3C
            0b00_001_110,
            0x9F,         // LD C, 0x9F
            0b11_100_010  // LD (C), A
        ];
        let mut cpu = create_cpu(rom);
        run_steps_without_wait_cycles(3, &mut cpu);
        let registers = &cpu.registers;
        assert_eq!(read_memory(&cpu, 0xFF9F), 0x3C);
        assert_eq!(registers.pc(), 5);
    }

    #[test]
    fn ld_a_mem_34() {
        let rom = vec![
            0b00_111_110,
            0x3C,         // LD A, 0x3C
            0b00_001_110,
            0x9F,         // LD C, 0x9F
            0b11_100_010, // LD (C), A
            0b00_111_110,
            0,            // LD A, 0
            0b11_110_000,
            0x9F        // LD A, (n)
        ];
        let mut cpu = create_cpu(rom);
        run_steps_without_wait_cycles(5, &mut cpu);
        let registers = &cpu.registers;
        assert_eq!(registers.a(), 0x3C);
        assert_eq!(registers.pc(), 9);
    }

    #[test]
    fn ld_mem_n_a() {
        let rom = vec![
            0b00_111_110,
            0x3C,         // LD A, 0x3C
            0b11_100_000,
            0x34          // LD (n), A
        ];
        let mut cpu = create_cpu(rom);
        run_steps_without_wait_cycles(2, &mut cpu);
        let registers = &cpu.registers;
        assert_eq!(read_memory(&cpu, 0xFF34), 0x3C);
        assert_eq!(registers.pc(), 4);
    }

    #[test]
    fn ld_a_mem_nn() {
        let rom = vec![
            0b00_100_001,
            0x44,
            0xFF,          // LD HL, 0xFF44
            0b_00_110_110,
            0x2F,          // LD (HL), 0x2F
            0b_11_111_010,
            0x44,
            0xFF,          // LD A, (nn)
        ];
        let mut cpu = create_cpu(rom);
        run_steps_without_wait_cycles(3, &mut cpu);
        let registers = &cpu.registers;
        assert_eq!(registers.a(), 0x2F);
        assert_eq!(registers.pc(), 8);
    }

    #[test]
    fn ld_mem_nn_a() {
        let rom = vec![
            0b00_111_110,
            0x3A,          // LD A, 0x3A
            0b11_101_010,
            0x44,
            0xFF,          // LD (nn), A
        ];
        let mut cpu = create_cpu(rom);
        run_steps_without_wait_cycles(2, &mut cpu);
        let registers = &cpu.registers;
        assert_eq!(read_memory(&cpu, 0xFF44), 0x3A);
        assert_eq!(registers.pc(), 5);
    }

    #[test]
    fn ld_a_mem_hli() {
        let rom = vec![
            0b00_100_001,
            0x08,
            0x80,          // LD HL, 0x8008
            0b_00_110_110,
            0x77,          // LD (HL), 0x77
            0b00_101_010   // LD A, (HLI)
        ];
        let mut cpu = create_cpu(rom);
        run_steps_without_wait_cycles(3, &mut cpu);
        let registers = &cpu.registers;
        assert_eq!(registers.a(), 0x77);
        assert_eq!(registers.hl(), 0x8009);
        assert_eq!(registers.pc(), 6);
    }

    #[test]
    fn ld_a_mem_hld() {
        let rom = vec![
            0b00_100_001,
            0x08,
            0x80,          // LD HL, 0x8008
            0b_00_110_110,
            0x77,          // LD (HL), 0x77
            0b00_111_010   // LD A, (HLD)
        ];
        let mut cpu = create_cpu(rom);
        run_steps_without_wait_cycles(3, &mut cpu);
        let registers = &cpu.registers;
        assert_eq!(registers.a(), 0x77);
        assert_eq!(registers.hl(), 0x8007);
        assert_eq!(registers.pc(), 6);
    }

    #[test]
    fn ld_mem_bc_a() {
        let rom = vec![
            0b00_000_001,
            0x00,
            0x80,         // LD BC, 0x8000
            0b00_111_110,
            0xAB,         // LD A, 0x3A
            0b00_000_010  // LD (BC), A
        ];
        let mut cpu = create_cpu(rom);
        run_steps_without_wait_cycles(3, &mut cpu);
        let registers = &cpu.registers;
        assert_eq!(read_memory(&cpu, 0x8000), 0xAB);
        assert_eq!(registers.pc(), 6);
    }

    #[test]
    fn ld_mem_de_a() {
        let rom = vec![
            0b00_010_001,
            0x00,
            0x80,         // LD DE, 0x8000
            0b00_111_110,
            0xAD,         // LD A, 0x3A
            0b00_010_010  // LD (DE), A
        ];
        let mut cpu = create_cpu(rom);
        run_steps_without_wait_cycles(3, &mut cpu);
        let registers = &cpu.registers;
        assert_eq!(read_memory(&cpu, 0x8000), 0xAD);
        assert_eq!(registers.pc(), 6);
    }

    #[test]
    fn ld_mem_hli_a() {
        let rom = vec![
            0b00_111_110,
            0x96,          // LD A, 0x96
            0b00_100_001,
            0x08,
            0x80,          // LD HL, 0x8008
            0b00_100_010   // LD (HLI), A

        ];
        let mut cpu = create_cpu(rom);
        run_steps_without_wait_cycles(3, &mut cpu);
        let registers = &cpu.registers;
        assert_eq!(registers.hl(), 0x8009);
        assert_eq!(read_memory(&cpu, 0x8008), 0x96);
        assert_eq!(registers.pc(), 6);
    }

    #[test]
    fn ld_mem_hld_a() {
        let rom = vec![
            0b00_111_110,
            0x97,          // LD A, 0x96
            0b00_100_001,
            0x08,
            0x80,          // LD HL, 0x8008
            0b00_110_010   // LD (HLD), A

        ];
        let mut cpu = create_cpu(rom);
        run_steps_without_wait_cycles(3, &mut cpu);
        let registers = &cpu.registers;
        assert_eq!(registers.hl(), 0x8007);
        assert_eq!(read_memory(&cpu, 0x8008), 0x97);
        assert_eq!(registers.pc(), 6);
    }

    #[test]
    fn ld_hl_0x3a5b() {
        let rom = vec![
            0b00_100_001,
            0x5B,
            0x3A          // LD HL, 0x3A5B

        ];
        let mut cpu = create_cpu(rom);
        run_steps_without_wait_cycles(1, &mut cpu);
        let registers = &cpu.registers;
        assert_eq!(registers.hl(), 0x3A5B);
        assert_eq!(registers.pc(), 3);
    }

    #[test]
    fn ld_sp_hl() {
        let rom = vec![
            0b00_100_001,
            0x5B,
            0x3A,         // LD HL, 0x3A5B
            0b11_111_001  // LD SP, HL
        ];
        let mut cpu = create_cpu(rom);
        run_steps_without_wait_cycles(2, &mut cpu);
        let registers = &cpu.registers;
        assert_eq!(registers.sp(), 0x3A5B);
        assert_eq!(registers.pc(), 4);
    }

    #[test]
    fn push_hl() {
        let rom = vec![
            0b00_100_001,
            0x5B,
            0x3A,         // LD HL, 0x3A5B
            0b11_100_101, // PUSH HL
        ];
        let mut cpu = create_cpu(rom);
        let old_sp = {cpu.registers.sp()};
        run_steps_without_wait_cycles(2, &mut cpu);
        let registers = &cpu.registers;
        let sp = registers.sp();
        assert_eq!(old_sp, 0xFFFE);
        assert_eq!(read_memory(&cpu, old_sp-1), 0x3A);
        assert_eq!(read_memory(&cpu, old_sp-2), 0x5B);
        assert_eq!(sp, old_sp-2);
        assert_eq!(registers.pc(), 4);
    }

    #[test]
    fn pop_bc() {
        let rom = vec![
            0b00_100_001,
            0x5B,
            0x3A,         // LD HL, 0x3A5B
            0b11_100_101, // PUSH HL

            0b11_000_001  // POP BC
        ];
        let mut cpu = create_cpu(rom);
        run_steps_without_wait_cycles(2, &mut cpu);
        let old_sp = cpu.registers.sp();
        run_steps_without_wait_cycles(1, &mut cpu);
        let registers = &cpu.registers;
        let sp = registers.sp();
        assert_eq!(sp, old_sp+2);
        assert_eq!(registers.b(), 0x3A);
        assert_eq!(registers.c(), 0x5B);
        assert_eq!(registers.pc(), 5);
    }

    #[test]
    fn ldhl_sp_sub_5() {
        let rom = vec![
            0b00_110_001,
            0xF5,
            0xFF,           // LD SP, 0xFFF0
            0b11_111_000,
            0b11111011      // LDHL SP, -5
        ];
        let mut cpu = create_cpu(rom);
        run_steps_without_wait_cycles(2, &mut cpu);
        let registers = &cpu.registers;
        assert_eq!(registers.hl(), 0xFFF0);
        assert_eq!(registers.pc(), 5);
    }

    #[test]
    fn ldhl_sp_add_5() {
        let rom = vec![
            0b00_110_001,
            0xF8,
            0xFF,           // LD SP, 0xFFF8
            0b11_111_000,
            2               // LDHL SP, 2
        ];
        let mut cpu = create_cpu(rom);
        run_steps_without_wait_cycles(2, &mut cpu);
        let registers = &cpu.registers;
        assert_eq!(registers.flag_z(), 0);
        assert_eq!(registers.flag_h(), 0);
        assert_eq!(registers.flag_n(), 0);
        assert_eq!(registers.flag_cy(), 0);
        assert_eq!(registers.hl(), 0xFFFA);
        assert_eq!(registers.pc(), 5);
    }

    #[test]
    fn ld_mem_0xfff8_sp() {
        let rom = vec![
            0b00_110_001,
            0xF8,
            0xFF,           // LD SP, 0xFFF8
            0b00_001_000,
            0x00,
            0x80,           // LD (nn), SP
        ];
        let mut cpu = create_cpu(rom);
        run_steps_without_wait_cycles(2, &mut cpu);
        let registers = &cpu.registers;
        assert_eq!(read_memory(&cpu, 0x8000), 0xf8);
        assert_eq!(read_memory(&cpu, 0x8001), 0xff);
        assert_eq!(registers.pc(), 6);
    }

    #[test]
    fn add_a_b() {
        let rom = vec![
            0b00_111_110,
            0x3A,           // LD A, 0x3A
            0b00_000_110,
            0xC6,           // LD B, 0xC6
            0b10_000_000    // ADD A, B
        ];
        let mut cpu = create_cpu(rom);
        run_steps_without_wait_cycles(3, &mut cpu);
        let registers = &cpu.registers;
        assert_eq!(registers.a(), 0x0);
        assert_eq!(registers.flag_z(), 1);
        assert_eq!(registers.flag_h(), 1);
        assert_eq!(registers.flag_n(), 0);
        assert_eq!(registers.flag_cy(), 1);
        assert_eq!(registers.pc(), 5);
    }

    #[test]
    fn add_a_0xff() {
        let rom = vec![
            0b00_111_110,
            0x3C,           // LD A, 0x3C
            0b11_000_110,
            0xFF            // ADD A, 0xFF
        ];
        let mut cpu = create_cpu(rom);
        run_steps_without_wait_cycles(2, &mut cpu);
        let registers = &cpu.registers;
        assert_eq!(registers.a(), 0x3B);
        assert_eq!(registers.flag_z(), 0);
        assert_eq!(registers.flag_h(), 1);
        assert_eq!(registers.flag_n(), 0);
        assert_eq!(registers.flag_cy(), 1);
        assert_eq!(registers.pc(), 4);
    }

    #[test]
    fn add_a_mem_hl() {
        let rom = vec![
            0b00_111_110,
            0x3C,           // LD A, 0x3C
            0b00_100_001,
            0x00,
            0x80,          // LD HL, 0x8000
            0b_00_110_110,
            0x12,          // LD (HL), 0x12
            0b10_000_110   // ADD A, (HL)
        ];
        let mut cpu = create_cpu(rom);
        run_steps_without_wait_cycles(4, &mut cpu);
        let registers = &cpu.registers;
        assert_eq!(registers.a(), 0x4E);
        assert_eq!(registers.flag_z(), 0);
        assert_eq!(registers.flag_h(), 0);
        assert_eq!(registers.flag_n(), 0);
        assert_eq!(registers.flag_cy(), 0);
        assert_eq!(registers.pc(), 8);
    }

    #[test]
    fn adc_a_e() {
        let rom = vec![
            0b00_111_110,
            0x3A,           // LD A, 0x3A
            0b11_000_110,
            0xC6,           // ADD A, 0xC6
            0b00_111_110,
            0xE1,           // LD A, 0xE1
            0b00_011_110,
            0x0F,           // LD E, 0x0F
            0b10_001_011    // ADD A, E
        ];
        let mut cpu = create_cpu(rom);
        run_steps_without_wait_cycles(5, &mut cpu);
        let registers = &cpu.registers;
        assert_eq!(registers.a(), 0xF1);
        assert_eq!(registers.flag_z(), 0);
        assert_eq!(registers.flag_h(), 1);
        assert_eq!(registers.flag_n(), 0);
        assert_eq!(registers.flag_cy(), 0);
        assert_eq!(registers.pc(), 9);
    }

    #[test]
    fn adc_a_0x3b() {
        let rom = vec![
            0b00_111_110,
            0x3A,           // LD A, 0x3A
            0b11_000_110,
            0xC6,           // ADD A, 0xC6
            0b00_111_110,
            0xE1,           // LD A, 0xE1
            0b11_001_110,
            0x3B            // ADD A, 0x3B
        ];
        let mut cpu = create_cpu(rom);
        run_steps_without_wait_cycles(4, &mut cpu);
        let registers = &cpu.registers;
        assert_eq!(registers.a(), 0x1D);
        assert_eq!(registers.flag_z(), 0);
        assert_eq!(registers.flag_h(), 0);
        assert_eq!(registers.flag_n(), 0);
        assert_eq!(registers.flag_cy(), 1);
        assert_eq!(registers.pc(), 8);
    }

    #[test]
    fn adc_a_mem_hl() {
        let rom = vec![
            0b00_111_110,
            0x3A,           // LD A, 0x3A
            0b11_000_110,
            0xC6,           // ADD A, 0xC6
            0b00_111_110,
            0xE1,           // LD A, 0xE1
            0b00_100_001,
            0x00,
            0x80,          // LD HL, 0x8000
            0b_00_110_110,
            0x1E,          // LD (HL), 0x1E
            0b10_001_110   // ADD A, (HL)
        ];
        let mut cpu = create_cpu(rom);
        run_steps_without_wait_cycles(6, &mut cpu);
        let registers = &cpu.registers;
        assert_eq!(registers.a(), 0x0);
        assert_eq!(registers.flag_z(), 1);
        assert_eq!(registers.flag_h(), 1);
        assert_eq!(registers.flag_n(), 0);
        assert_eq!(registers.flag_cy(), 1);
        assert_eq!(registers.pc(), 12);
    }

    #[test]
    fn sub_a_h() {
        let rom = vec![
            0b00_111_110,
            0x3E,           // LD A, 0x3E
            0b00_100_110,
            0x3E,           // LD H, 0x3E
            0b10_010_100    // ADD A, H
        ];
        let mut cpu = create_cpu(rom);
        run_steps_without_wait_cycles(3, &mut cpu);
        let registers = &cpu.registers;
        assert_eq!(registers.a(), 0x0);
        assert_eq!(registers.flag_z(), 1);
        assert_eq!(registers.flag_h(), 0);
        assert_eq!(registers.flag_n(), 1);
        assert_eq!(registers.flag_cy(), 0);
        assert_eq!(registers.pc(), 5);
    }

    #[test]
    fn sub_a_0x0f() {
        let rom = vec![
            0b00_111_110,
            0x3E,           // LD A, 0x3E
            0b11_010_110,
            0x0F            // ADD A, 0x0F
        ];
        let mut cpu = create_cpu(rom);
        run_steps_without_wait_cycles(2, &mut cpu);
        let registers = &cpu.registers;
        assert_eq!(registers.a(), 0x2F);
        assert_eq!(registers.flag_z(), 0);
        assert_eq!(registers.flag_h(), 1);
        assert_eq!(registers.flag_n(), 1);
        assert_eq!(registers.flag_cy(), 0);
        assert_eq!(registers.pc(), 4);
    }

    #[test]
    fn sub_a_mem_hl() {
        let rom = vec![
            0b00_111_110,
            0x3E,           // LD A, 0x3E
            0b00_100_001,
            0x00,
            0x80,          // LD HL, 0x8000
            0b_00_110_110,
            0x40,          // LD (HL), 0x40
            0b10_010_110   // SUB A, (HL)
        ];
        let mut cpu = create_cpu(rom);
        run_steps_without_wait_cycles(4, &mut cpu);
        let registers = &cpu.registers;
        assert_eq!(registers.a(), 0xFE);
        assert_eq!(registers.flag_z(), 0);
        assert_eq!(registers.flag_h(), 0);
        assert_eq!(registers.flag_n(), 1);
        assert_eq!(registers.flag_cy(), 1);
        assert_eq!(registers.pc(), 8);
    }

    #[test]
    fn sbc_a_h() {
        let rom =vec![
            0b00_111_110,
            0x3A,           // LD A, 0x3A
            0b11_000_110,
            0xC6,           // ADD A, 0xC6
            0b00_111_110,
            0x3B,           // LD A, 0x3B
            0b00_100_110,
            0x2A,           // LD H, 0x2A
            0b10_011_100    // SBC A, E
        ];
        let mut cpu = create_cpu(rom);
        run_steps_without_wait_cycles(5, &mut cpu);
        let registers = &cpu.registers;
        assert_eq!(registers.a(), 0x10);
        assert_eq!(registers.flag_z(), 0);
        assert_eq!(registers.flag_h(), 0);
        assert_eq!(registers.flag_n(), 1);
        assert_eq!(registers.flag_cy(), 0);
        assert_eq!(registers.pc(), 9);
    }

    #[test]
    fn sbc_a_0x3a() {
        let rom = vec![
            0b00_111_110,
            0x3A,           // LD A, 0x3A
            0b11_000_110,
            0xC6,           // ADD A, 0xC6
            0b00_111_110,
            0x3B,           // LD A, 0x3B
            0b11_011_110,
            0x3A            // SBC A, 0x3A
        ];
        let mut cpu = create_cpu(rom);
        run_steps_without_wait_cycles(4, &mut cpu);
        let registers = &cpu.registers;
        assert_eq!(registers.a(), 0x00);
        assert_eq!(registers.flag_z(), 1);
        assert_eq!(registers.flag_h(), 0);
        assert_eq!(registers.flag_n(), 1);
        assert_eq!(registers.flag_cy(), 0);
        assert_eq!(registers.pc(), 8);
    }

    #[test]
    fn sbc_a_mem_hl() {
        let rom = vec![
            0b00_111_110,
            0x3A,           // LD A, 0x3A
            0b11_000_110,
            0xC6,           // ADD A, 0xC6
            0b00_111_110,
            0x3B,           // LD A, 0x3B
            0b00_100_001,
            0x00,
            0x80,          // LD HL, 0x8000
            0b_00_110_110,
            0x4F,          // LD (HL), 0x4F
            0b10_011_110   // SBC A, (HL)
        ];
        let mut cpu = create_cpu(rom);
        run_steps_without_wait_cycles(6, &mut cpu);
        let registers = &cpu.registers;
        assert_eq!(registers.a(), 0xEB);
        assert_eq!(registers.flag_z(), 0);
        assert_eq!(registers.flag_h(), 1);
        assert_eq!(registers.flag_n(), 1);
        assert_eq!(registers.flag_cy(), 1);
        assert_eq!(registers.pc(), 12);
    }

    #[test]
    fn and_a_l() {
        let rom = vec![
            0b00_111_110,
            0x5A,           // LD A, 0x5A
            0b00_101_110,
            0x3F,           // LD L, 0x3F
            0b10_100_101    // AND A, L
        ];
        let mut cpu = create_cpu(rom);
        run_steps_without_wait_cycles(3, &mut cpu);
        let registers = &cpu.registers;
        assert_eq!(registers.a(), 0x1A);
        assert_eq!(registers.flag_z(), 0);
        assert_eq!(registers.flag_h(), 1);
        assert_eq!(registers.flag_n(), 0);
        assert_eq!(registers.flag_cy(), 0);
        assert_eq!(registers.pc(), 5);
    }

    #[test]
    fn and_a_0x38() {
        let rom = vec![
            0b00_111_110,
            0x5A,           // LD A, 0x5A
            0b11_100_110,
            0x38            // AND A, 0x38
        ];
        let mut cpu = create_cpu(rom);
        run_steps_without_wait_cycles(2, &mut cpu);
        let registers = &cpu.registers;
        assert_eq!(registers.a(), 0x18);
        assert_eq!(registers.flag_z(), 0);
        assert_eq!(registers.flag_h(), 1);
        assert_eq!(registers.flag_n(), 0);
        assert_eq!(registers.flag_cy(), 0);
        assert_eq!(registers.pc(), 4);
    }

    #[test]
    fn and_a_mem_hl() {
        let rom = vec![
            0b00_111_110,
            0x5A,           // LD A, 0x5A
            0b00_100_001,
            0x00,
            0x80,          // LD HL, 0x8000
            0b00_110_110,
            0x00,          // LD (HL), 0x00
            0b10_100_110   // AND A, (HL)
        ];
        let mut cpu = create_cpu(rom);
        run_steps_without_wait_cycles(4, &mut cpu);
        let registers = &cpu.registers;
        assert_eq!(registers.a(), 0x00);
        assert_eq!(registers.flag_z(), 1);
        assert_eq!(registers.flag_h(), 1);
        assert_eq!(registers.flag_n(), 0);
        assert_eq!(registers.flag_cy(), 0);
        assert_eq!(registers.pc(), 8);
    }

    #[test]
    fn or_a_l() {
        let rom = vec![
            0b00_111_110,
            0x5A,           // LD A, 0x5A
            0b10_110_111    // OR A, A
        ];
        let mut cpu = create_cpu(rom);
        run_steps_without_wait_cycles(2, &mut cpu);
        let registers = &cpu.registers;
        assert_eq!(registers.a(), 0x5A);
        assert_eq!(registers.flag_z(), 0);
        assert_eq!(registers.flag_h(), 0);
        assert_eq!(registers.flag_n(), 0);
        assert_eq!(registers.flag_cy(), 0);
        assert_eq!(registers.pc(), 3);
    }

    #[test]
    fn or_a_0x03() {
        let rom = vec![
            0b00_111_110,
            0x5A,           // LD A, 0x5A
            0b11_110_110,
            0x03            // OR A, 0x03
        ];
        let mut cpu = create_cpu(rom);
        run_steps_without_wait_cycles(2, &mut cpu);
        let registers = &cpu.registers;
        assert_eq!(registers.a(), 0x5B);
        assert_eq!(registers.flag_z(), 0);
        assert_eq!(registers.flag_h(), 0);
        assert_eq!(registers.flag_n(), 0);
        assert_eq!(registers.flag_cy(), 0);
        assert_eq!(registers.pc(), 4);
    }

    #[test]
    fn or_a_mem_hl() {
        let rom = vec![
            0b00_111_110,
            0x5A,           // LD A, 0x5A
            0b00_100_001,
            0x00,
            0x80,          // LD HL, 0x8000
            0b00_110_110,
            0x00,          // LD (HL), 0x00
            0b10_110_110   // OR A, (HL)
        ];
        let mut cpu = create_cpu(rom);
        run_steps_without_wait_cycles(4, &mut cpu);
        let registers = &cpu.registers;
        assert_eq!(registers.a(), 0x5A);
        assert_eq!(registers.flag_z(), 0);
        assert_eq!(registers.flag_h(), 0);
        assert_eq!(registers.flag_n(), 0);
        assert_eq!(registers.flag_cy(), 0);
        assert_eq!(registers.pc(), 8);
    }

    #[test]
    fn xor_a_l() {
        let rom = vec![
            0b00_111_110,
            0xFF,           // LD A, 0xFF
            0b10_101_111    // XOR A, A
        ];
        let mut cpu = create_cpu(rom);
        run_steps_without_wait_cycles(2, &mut cpu);
        let registers = &cpu.registers;
        assert_eq!(registers.a(), 0x00);
        assert_eq!(registers.flag_z(), 1);
        assert_eq!(registers.flag_h(), 0);
        assert_eq!(registers.flag_n(), 0);
        assert_eq!(registers.flag_cy(), 0);
        assert_eq!(registers.pc(), 3);
    }

    #[test]
    fn xor_a_0x0f() {
        let rom = vec![
            0b00_111_110,
            0xFF,           // LD A, 0xFF
            0b11_101_110,
            0x0F            // XOR A, 0x0F
        ];
        let mut cpu = create_cpu(rom);
        run_steps_without_wait_cycles(2, &mut cpu);
        let registers = &cpu.registers;
        assert_eq!(registers.a(), 0xF0);
        assert_eq!(registers.flag_z(), 0);
        assert_eq!(registers.flag_h(), 0);
        assert_eq!(registers.flag_n(), 0);
        assert_eq!(registers.flag_cy(), 0);
        assert_eq!(registers.pc(), 4);
    }

    #[test]
    fn xor_a_mem_hl() {
        let rom = vec![
            0b00_111_110,
            0xFF,           // LD A, 0xFF
            0b00_100_001,
            0x00,
            0x80,          // LD HL, 0x8000
            0b00_110_110,
            0x8A,          // LD (HL), 0x8A
            0b10_101_110   // XOR A, (HL)
        ];
        let mut cpu = create_cpu(rom);
        run_steps_without_wait_cycles(4, &mut cpu);
        let registers = &cpu.registers;
        assert_eq!(registers.a(), 0x75);
        assert_eq!(registers.flag_z(), 0);
        assert_eq!(registers.flag_h(), 0);
        assert_eq!(registers.flag_n(), 0);
        assert_eq!(registers.flag_cy(), 0);
        assert_eq!(registers.pc(), 8);
    }

    #[test]
    fn cp_a_b() {
        let rom = vec![
            0b00_111_110,
            0x3C,           // LD A, 0x3C
            0b00_000_110,
            0x2F,           // LD B, 0x2F
            0b10_111_000    // CP A, B
        ];
        let mut cpu = create_cpu(rom);
        run_steps_without_wait_cycles(3, &mut cpu);
        let registers = &cpu.registers;
        assert_eq!(registers.a(), 0x3C);
        assert_eq!(registers.flag_z(), 0);
        assert_eq!(registers.flag_h(), 1);
        assert_eq!(registers.flag_n(), 1);
        assert_eq!(registers.flag_cy(), 0);
        assert_eq!(registers.pc(), 5);
    }

    #[test]
    fn cp_a_0x3c() {
        let rom = vec![
            0b00_111_110,
            0x3C,           // LD A, 0x3C
            0b11_111_110,
            0x3C            // CP A, 0x3C
        ];
        let mut cpu = create_cpu(rom);
        run_steps_without_wait_cycles(2, &mut cpu);
        let registers = &cpu.registers;
        assert_eq!(registers.a(), 0x3C);
        assert_eq!(registers.flag_z(), 1);
        assert_eq!(registers.flag_h(), 0);
        assert_eq!(registers.flag_n(), 1);
        assert_eq!(registers.flag_cy(), 0);
        assert_eq!(registers.pc(), 4);
    }

    #[test]
    fn cp_a_mem_hl() {
        let rom = vec![
            0b00_111_110,
            0x3C,           // LD A, 0x3C
            0b00_100_001,
            0x00,
            0x80,          // LD HL, 0x8000
            0b_00_110_110,
            0x40,          // LD (HL), 0x40
            0b10_111_110   // SUB A, (HL)
        ];
        let mut cpu = create_cpu(rom);
        run_steps_without_wait_cycles(4, &mut cpu);
        let registers = &cpu.registers;
        assert_eq!(registers.a(), 0x3C);
        assert_eq!(registers.flag_z(), 0);
        assert_eq!(registers.flag_h(), 0);
        assert_eq!(registers.flag_n(), 1);
        assert_eq!(registers.flag_cy(), 1);
        assert_eq!(registers.pc(), 8);
    }

    #[test]
    fn inc_r() {
        let rom = vec![
            0b00_111_110,
            0xFF,           // LD A, 0xFF
            0b00_111_100    // INC A
        ];
        let mut cpu = create_cpu(rom);
        let cy = cpu.registers.flag_cy();
        run_steps_without_wait_cycles(2, &mut cpu);
        let registers = &cpu.registers;
        assert_eq!(registers.a(), 0x00);
        assert_eq!(registers.flag_z(), 1);
        assert_eq!(registers.flag_h(), 1);
        assert_eq!(registers.flag_n(), 0);
        assert_eq!(registers.flag_cy(), cy);
        assert_eq!(registers.pc(), 3);
    }

    #[test]
    fn inc_mem_hl() {
        let rom = vec![
            0b00_100_001,
            0x00,
            0x80,          // LD HL, 0x8000
            0b00_110_110,
            0x50,          // LD (HL), 0x50
            0b00_110_100   // INC (HL)
        ];
        let mut cpu = create_cpu(rom);
        let cy = cpu.registers.flag_cy();
        run_steps_without_wait_cycles(3, &mut cpu);
        let registers = &cpu.registers;
        assert_eq!(read_memory(&cpu, 0x8000), 0x51);
        assert_eq!(registers.flag_z(), 0);
        assert_eq!(registers.flag_h(), 0);
        assert_eq!(registers.flag_n(), 0);
        assert_eq!(registers.flag_cy(), cy);
        assert_eq!(registers.pc(), 6);
    }

    #[test]
    fn dec_r() {
        let rom = vec![
            0b00_101_110,
            0x01,           // LD L, 0x01
            0b00_101_101    // DEC L
        ];
        let mut cpu = create_cpu(rom);
        let cy = cpu.registers.flag_cy();
        run_steps_without_wait_cycles(2, &mut cpu);
        let registers = &cpu.registers;
        assert_eq!(registers.l(), 0x00);
        assert_eq!(registers.flag_z(), 1);
        assert_eq!(registers.flag_h(), 0);
        assert_eq!(registers.flag_n(), 1);
        assert_eq!(registers.flag_cy(), cy);
        assert_eq!(registers.pc(), 3);
    }

    #[test]
    fn dec_mem_hl() {
        let rom = vec![
            0b00_100_001,
            0x00,
            0x80,          // LD HL, 0x8000
            0b00_110_110,
            0x00,          // LD (HL), 0x00
            0b00_110_101   // DEC (HL)
        ];
        let mut cpu = create_cpu(rom);
        let cy = cpu.registers.flag_cy();
        run_steps_without_wait_cycles(3, &mut cpu);
        let registers = &cpu.registers;
        assert_eq!(read_memory(&cpu, 0x8000), 0xFF);
        assert_eq!(registers.flag_z(), 0);
        assert_eq!(registers.flag_h(), 1);
        assert_eq!(registers.flag_n(), 1);
        assert_eq!(registers.flag_cy(), cy);
        assert_eq!(registers.pc(), 6);
    }

    #[test]
    fn add_hl_bc() {
        let rom = vec![
            0b00_100_001,
            0x23,
            0x8A,           // LD HL, 0x8A23
            0b00_000_001,
            0x05,
            0x06,           // LD BC, 0x0605
            0b00_001_001    // ADD HL, BC

        ];
        let mut cpu = create_cpu(rom);
        run_steps_without_wait_cycles(3, &mut cpu);
        let registers = &cpu.registers;
        assert_eq!(registers.hl(), 0x9028);
        assert_eq!(registers.flag_h(), 1);
        assert_eq!(registers.flag_n(), 0);
        assert_eq!(registers.flag_cy(), 0);
        assert_eq!(registers.pc(), 7);
    }

    #[test]
    fn add_hl_hl() {
        let rom = vec![
            0b00_100_001,
            0x23,
            0x8A,           // LD HL, 0x8A23
            0b00_101_001    // ADD HL, BC

        ];
        let mut cpu = create_cpu(rom);
        let z = cpu.registers.flag_z();
        run_steps_without_wait_cycles(2, &mut cpu);
        let registers = &cpu.registers;
        assert_eq!(registers.hl(), 0x1446);
        assert_eq!(registers.flag_z(), z);
        assert_eq!(registers.flag_h(), 1);
        assert_eq!(registers.flag_n(), 0);
        assert_eq!(registers.flag_cy(), 1);
        assert_eq!(registers.pc(), 4);
    }

    #[test]
    fn add_sp_2() {
        let rom = vec![
            0b00_110_001,
            0xF8,
            0xFF,           // LD SP, 0xFFF8
            0b11_101_000,
            2               // ADD SP, 2

        ];
        let mut cpu = create_cpu(rom);
        run_steps_without_wait_cycles(2, &mut cpu);
        let registers = &cpu.registers;
        assert_eq!(registers.sp(), 0xFFFA);
        assert_eq!(registers.flag_z(), 0);
        assert_eq!(registers.flag_h(), 0);
        assert_eq!(registers.flag_n(), 0);
        assert_eq!(registers.flag_cy(), 0);
        assert_eq!(registers.pc(), 5);
    }

    #[test]
    fn inc_de() {
        let rom = vec![
            0b00_010_001,
            0x5F,
            0x23,           // LD HL, 0x235F
            0b00_010_011    // INC DE

        ];
        let mut cpu = create_cpu(rom);
        let z = cpu.registers.flag_z();
        let h = cpu.registers.flag_h();
        let n = cpu.registers.flag_n();
        let cy = cpu.registers.flag_cy();
        run_steps_without_wait_cycles(2, &mut cpu);
        let registers = &cpu.registers;
        assert_eq!(registers.de(), 0x2360);
        assert_eq!(registers.flag_z(), z);
        assert_eq!(registers.flag_h(), h);
        assert_eq!(registers.flag_n(), n);
        assert_eq!(registers.flag_cy(), cy);
        assert_eq!(registers.pc(), 4);
    }

    #[test]
    fn dec_de() {
        let rom = vec![
            0b00_010_001,
            0x5F,
            0x23,           // LD HL, 0x235F
            0b00_011_011    // DEC DE

        ];
        let mut cpu = create_cpu(rom);
        let z = cpu.registers.flag_z();
        let h = cpu.registers.flag_h();
        let n = cpu.registers.flag_n();
        let cy = cpu.registers.flag_cy();
        run_steps_without_wait_cycles(2, &mut cpu);
        let registers = &cpu.registers;
        assert_eq!(registers.de(), 0x235E);
        assert_eq!(registers.flag_z(), z);
        assert_eq!(registers.flag_h(), h);
        assert_eq!(registers.flag_n(), n);
        assert_eq!(registers.flag_cy(), cy);
        assert_eq!(registers.pc(), 4);
    }

    #[test]
    fn rlca() {
        let rom = vec![
            0b00_111_110,
            0x85,           // LD A, 0x85
            0b00_000_111    // RLCA

        ];
        let mut cpu = create_cpu(rom);
        run_steps_without_wait_cycles(2, &mut cpu);
        let registers = &cpu.registers;
        assert_eq!(registers.a(), 0x0B);
        assert_eq!(registers.flag_z(), 0);
        assert_eq!(registers.flag_h(), 0);
        assert_eq!(registers.flag_n(), 0);
        assert_eq!(registers.flag_cy(), 1);
        assert_eq!(registers.pc(), 3);
    }

    #[test]
    fn rla() {
        let rom = vec![
            0b00_111_110,
            0xFF,           // LD A, 0xFF
            0b11_000_110,
            1,              // ADD A, 1
            0b00_111_110,
            0x95,           // LD A, 0x95
            0b00_010_111    // RLA

        ];
        let mut cpu = create_cpu(rom);
        run_steps_without_wait_cycles(4, &mut cpu);
        let registers = &cpu.registers;
        assert_eq!(registers.a(), 0x2B);
        assert_eq!(registers.flag_z(), 0);
        assert_eq!(registers.flag_h(), 0);
        assert_eq!(registers.flag_n(), 0);
        assert_eq!(registers.flag_cy(), 1);
        assert_eq!(registers.pc(), 7);
    }

    #[test]
    fn rrca() {
        let rom = vec![
            0b00_111_110,
            0x3b,           // LD A, 0x3b
            0b00_001_111    // RRCA

        ];
        let mut cpu = create_cpu(rom);
        run_steps_without_wait_cycles(2, &mut cpu);
        let registers = &cpu.registers;
        assert_eq!(registers.a(), 0x9D);
        assert_eq!(registers.flag_z(), 0);
        assert_eq!(registers.flag_h(), 0);
        assert_eq!(registers.flag_n(), 0);
        assert_eq!(registers.flag_cy(), 1);
        assert_eq!(registers.pc(), 3);
    }

    #[test]
    fn rra() {
        let rom = vec![
            0b00_111_110,
            0x81,           // LD A, 0x81
            0b00_011_111    // RRA

        ];
        let mut cpu = create_cpu(rom);
        run_steps_without_wait_cycles(2, &mut cpu);
        let registers = &cpu.registers;
        assert_eq!(registers.a(), 0x40);
        assert_eq!(registers.flag_z(), 0);
        assert_eq!(registers.flag_h(), 0);
        assert_eq!(registers.flag_n(), 0);
        assert_eq!(registers.flag_cy(), 1);
        assert_eq!(registers.pc(), 3);
    }

    #[test]
    fn rlc_b() {
        let rom = vec![
            0b00_000_110,
            0x85,           // LD B, 0x85
            0b11_001_011,
            0b00_000_000    // RLC B

        ];
        let mut cpu = create_cpu(rom);
        run_steps_without_wait_cycles(2, &mut cpu);
        let registers = &cpu.registers;
        assert_eq!(registers.b(), 0x0B);
        assert_eq!(registers.flag_z(), 0);
        assert_eq!(registers.flag_h(), 0);
        assert_eq!(registers.flag_n(), 0);
        assert_eq!(registers.flag_cy(), 1);
        assert_eq!(registers.pc(), 4);
    }

    #[test]
    fn rlc_mem_hl() {
        let rom = vec![
            0b00_100_001,
            0x00,
            0x80,          // LD HL, 0x8000
            0b00_110_110,
            0,             // LD (HL), 0
            0b11_001_011,
            0b00_000_110   // RLC (HL)

        ];
        let mut cpu = create_cpu(rom);
        run_steps_without_wait_cycles(3, &mut cpu);
        let registers = &cpu.registers;
        assert_eq!(read_memory(&cpu, 0x8000), 0);
        assert_eq!(registers.flag_z(), 1);
        assert_eq!(registers.flag_h(), 0);
        assert_eq!(registers.flag_n(), 0);
        assert_eq!(registers.flag_cy(), 0);
        assert_eq!(registers.pc(), 7);
    }

    #[test]
    fn rl_l() {
        let rom = vec![
            0b00_101_110,
            0x80,           // LD L, 0x80
            0b11_001_011,
            0b00_010_101    // RLC L

        ];
        let mut cpu = create_cpu(rom);
        run_steps_without_wait_cycles(2, &mut cpu);
        let registers = &cpu.registers;
        assert_eq!(registers.l(), 0x00);
        assert_eq!(registers.flag_z(), 1);
        assert_eq!(registers.flag_h(), 0);
        assert_eq!(registers.flag_n(), 0);
        assert_eq!(registers.flag_cy(), 1);
        assert_eq!(registers.pc(), 4);
    }

    #[test]
    fn rl_mem_hl() {
        let rom = vec![
            0b00_100_001,
            0x00,
            0x80,          // LD HL, 0x8000
            0b00_110_110,
            0x11,          // LD (HL), 0x11
            0b11_001_011,
            0b00_010_110   // RLC (HL)

        ];
        let mut cpu = create_cpu(rom);
        run_steps_without_wait_cycles(3, &mut cpu);
        let registers = &cpu.registers;
        assert_eq!(read_memory(&cpu, 0x8000), 0x22);
        assert_eq!(registers.flag_z(), 0);
        assert_eq!(registers.flag_h(), 0);
        assert_eq!(registers.flag_n(), 0);
        assert_eq!(registers.flag_cy(), 0);
        assert_eq!(registers.pc(), 7);
    }

    #[test]
    fn rrc_c() {
        let rom = vec![
            0b00_001_110,
            0x1,           // LD C, 0x1
            0b11_001_011,
            0b00_001_001    // RRC C

        ];
        let mut cpu = create_cpu(rom);
        run_steps_without_wait_cycles(2, &mut cpu);
        let registers = &cpu.registers;
        assert_eq!(registers.c(), 0x80);
        assert_eq!(registers.flag_z(), 0);
        assert_eq!(registers.flag_h(), 0);
        assert_eq!(registers.flag_n(), 0);
        assert_eq!(registers.flag_cy(), 1);
        assert_eq!(registers.pc(), 4);
    }

    #[test]
    fn rrc_mem_hl() {
        let rom = vec![
            0b00_100_001,
            0x00,
            0x80,          // LD HL, 0x8000
            0b00_110_110,
            0x0,          // LD (HL), 0x0
            0b11_001_011,
            0b00_001_110   // RRC (HL)

        ];
        let mut cpu = create_cpu(rom);
        run_steps_without_wait_cycles(3, &mut cpu);
        let registers = &cpu.registers;
        assert_eq!(read_memory(&cpu, 0x8000), 0);
        assert_eq!(registers.flag_z(), 1);
        assert_eq!(registers.flag_h(), 0);
        assert_eq!(registers.flag_n(), 0);
        assert_eq!(registers.flag_cy(), 0);
        assert_eq!(registers.pc(), 7);
    }

    #[test]
    fn rr_a() {
        let rom = vec![
            0b00_111_110,
            0x1,           // LD A, 0x1
            0b11_001_011,
            0b00_011_111    // RRC A

        ];
        let mut cpu = create_cpu(rom);
        run_steps_without_wait_cycles(2, &mut cpu);
        let registers = &cpu.registers;
        assert_eq!(registers.a(), 0x00);
        assert_eq!(registers.flag_z(), 1);
        assert_eq!(registers.flag_h(), 0);
        assert_eq!(registers.flag_n(), 0);
        assert_eq!(registers.flag_cy(), 1);
        assert_eq!(registers.pc(), 4);
    }

    #[test]
    fn rr_mem_hl() {
        let rom = vec![
            0b00_100_001,
            0x00,
            0x80,          // LD HL, 0x8000
            0b00_110_110,
            0x8a,          // LD (HL), 0x8A
            0b11_001_011,
            0b00_011_110   // RR (HL)

        ];
        let mut cpu = create_cpu(rom);
        run_steps_without_wait_cycles(3, &mut cpu);
        let registers = &cpu.registers;
        assert_eq!(read_memory(&cpu, 0x8000), 0x45);
        assert_eq!(registers.flag_z(), 0);
        assert_eq!(registers.flag_h(), 0);
        assert_eq!(registers.flag_n(), 0);
        assert_eq!(registers.flag_cy(), 0);
        assert_eq!(registers.pc(), 7);
    }

    #[test]
    fn sla_d() {
        let rom = vec![
            0b00_010_110,
            0x80,           // LD D, 0x80
            0b11_001_011,
            0b00_100_010    // SLA D

        ];
        let mut cpu = create_cpu(rom);
        run_steps_without_wait_cycles(2, &mut cpu);
        let registers = &cpu.registers;
        assert_eq!(registers.d(), 0x00);
        assert_eq!(registers.flag_z(), 1);
        assert_eq!(registers.flag_h(), 0);
        assert_eq!(registers.flag_n(), 0);
        assert_eq!(registers.flag_cy(), 1);
        assert_eq!(registers.pc(), 4);
    }

    #[test]
    fn sla_mem_hl() {
        let rom = vec![
            0b00_100_001,
            0x00,
            0x80,          // LD HL, 0x8000
            0b00_110_110,
            0xFF,          // LD (HL), 0xFF
            0b11_001_011,
            0b00_100_110   // SLA (HL)

        ];
        let mut cpu = create_cpu(rom);
        run_steps_without_wait_cycles(3, &mut cpu);
        let registers = &cpu.registers;
        assert_eq!(read_memory(&cpu, 0x8000), 0xFE);
        assert_eq!(registers.flag_z(), 0);
        assert_eq!(registers.flag_h(), 0);
        assert_eq!(registers.flag_n(), 0);
        assert_eq!(registers.flag_cy(), 1);
        assert_eq!(registers.pc(), 7);
    }

    #[test]
fn sra_a() {
    let rom = vec![
        0b00_111_110,
        0x8A,           // LD A, 0x8A
        0b11_001_011,
        0b00_101_111    // SRA A

    ];
    let mut cpu = create_cpu(rom);
        run_steps_without_wait_cycles(2, &mut cpu);
    let registers = &cpu.registers;
    assert_eq!(registers.a(), 0xC5);
    assert_eq!(registers.flag_z(), 0);
    assert_eq!(registers.flag_h(), 0);
    assert_eq!(registers.flag_n(), 0);
    assert_eq!(registers.flag_cy(), 0);
    assert_eq!(registers.pc(), 4);
}

    #[test]
    fn sra_mem_hl() {
        let rom = vec![
            0b00_100_001,
            0x00,
            0x80,          // LD HL, 0x8000
            0b00_110_110,
            0x01,          // LD (HL), 0x01
            0b11_001_011,
            0b00_101_110   // SRA (HL)

        ];
        let mut cpu = create_cpu(rom);
        run_steps_without_wait_cycles(3, &mut cpu);
        let registers = &cpu.registers;
        assert_eq!(read_memory(&cpu, 0x8000), 0x00);
        assert_eq!(registers.flag_z(), 1);
        assert_eq!(registers.flag_h(), 0);
        assert_eq!(registers.flag_n(), 0);
        assert_eq!(registers.flag_cy(), 1);
        assert_eq!(registers.pc(), 7);
    }

    #[test]
    fn srl_a() {
        let rom = vec![
            0b00_111_110,
            0x01,           // LD A, 0x01
            0b11_001_011,
            0b00_111_111    // SRL A

        ];
        let mut cpu = create_cpu(rom);
        run_steps_without_wait_cycles(2, &mut cpu);
        let registers = &cpu.registers;
        assert_eq!(registers.a(), 0x00);
        assert_eq!(registers.flag_z(), 1);
        assert_eq!(registers.flag_h(), 0);
        assert_eq!(registers.flag_n(), 0);
        assert_eq!(registers.flag_cy(), 1);
        assert_eq!(registers.pc(), 4);
    }

    #[test]
    fn srl_mem_hl() {
        let rom = vec![
            0b00_100_001,
            0x00,
            0x80,          // LD HL, 0x8000
            0b00_110_110,
            0xFF,          // LD (HL), 0xFF
            0b11_001_011,
            0b00_111_110   // SRL (HL)

        ];
        let mut cpu = create_cpu(rom);
        run_steps_without_wait_cycles(3, &mut cpu);
        let registers = &cpu.registers;
        assert_eq!(read_memory(&cpu, 0x8000), 0x7F);
        assert_eq!(registers.flag_z(), 0);
        assert_eq!(registers.flag_h(), 0);
        assert_eq!(registers.flag_n(), 0);
        assert_eq!(registers.flag_cy(), 1);
        assert_eq!(registers.pc(), 7);
    }

    #[test]
    fn swap_a() {
        let rom = vec![
            0b00_111_110,
            0x00,           // LD A, 0x00
            0b11_001_011,
            0b00_110_111    // SWAP A

        ];
        let mut cpu = create_cpu(rom);
        run_steps_without_wait_cycles(2, &mut cpu);
        let registers = &cpu.registers;
        assert_eq!(registers.a(), 0x00);
        assert_eq!(registers.flag_z(), 1);
        assert_eq!(registers.flag_h(), 0);
        assert_eq!(registers.flag_n(), 0);
        assert_eq!(registers.flag_cy(), 0);
        assert_eq!(registers.pc(), 4);
    }

    #[test]
    fn swap_mem_hl() {
        let rom = vec![
            0b00_100_001,
            0x00,
            0x80,          // LD HL, 0x8000
            0b00_110_110,
            0xF0,          // LD (HL), 0xF0
            0b11_001_011,
            0b00_110_110   // SWAP (HL)

        ];
        let mut cpu = create_cpu(rom);
        run_steps_without_wait_cycles(3, &mut cpu);
        let registers = &cpu.registers;
        assert_eq!(read_memory(&cpu, 0x8000), 0x0F);
        assert_eq!(registers.flag_z(), 0);
        assert_eq!(registers.flag_h(), 0);
        assert_eq!(registers.flag_n(), 0);
        assert_eq!(registers.flag_cy(), 0);
        assert_eq!(registers.pc(), 7);
    }

    #[test]
    fn bit_7_a() {
        let rom = vec![
            0b00_111_110,
            0x80,           // LD A, 0x80
            0b11_001_011,
            0b01_111_111    // BIT 7, A

        ];
        let mut cpu = create_cpu(rom);
        run_steps_without_wait_cycles(2, &mut cpu);
        let registers = &cpu.registers;
        assert_eq!(registers.flag_z(), 0);
        assert_eq!(registers.flag_h(), 1);
        assert_eq!(registers.flag_n(), 0);
        assert_eq!(registers.pc(), 4);
    }

    #[test]
    fn bit_4_l() {
        let rom = vec![
            0b00_101_110,
            0xEF,           // LD L, 0xEF
            0b11_001_011,
            0b01_100_101    // BIT 4, L

        ];
        let mut cpu = create_cpu(rom);
        run_steps_without_wait_cycles(2, &mut cpu);
        let registers = &cpu.registers;
        assert_eq!(registers.flag_z(), 1);
        assert_eq!(registers.flag_h(), 1);
        assert_eq!(registers.flag_n(), 0);
        assert_eq!(registers.pc(), 4);
    }

    #[test]
    fn bit_0_mem_hl() {
        let rom = vec![
            0b00_100_001,
            0x00,
            0x80,          // LD HL, 0x8000
            0b00_110_110,
            0xFE,          // LD (HL), 0xFE
            0b11_001_011,
            0b01_000_110   // Bit 0, (HL)

        ];
        let mut cpu = create_cpu(rom);
        run_steps_without_wait_cycles(3, &mut cpu);
        let registers = &cpu.registers;
        assert_eq!(registers.flag_z(), 1);
        assert_eq!(registers.flag_h(), 1);
        assert_eq!(registers.flag_n(), 0);
        assert_eq!(registers.pc(), 7);
    }

    #[test]
    fn bit_1_mem_hl() {
        let rom = vec![
            0b00_100_001,
            0x00,
            0x80,          // LD HL, 0x8000
            0b00_110_110,
            0xFE,          // LD (HL), 0xFE
            0b11_001_011,
            0b01_001_110   // Bit 0, (HL)

        ];
        let mut cpu = create_cpu(rom);
        run_steps_without_wait_cycles(3, &mut cpu);
        let registers = &cpu.registers;
        assert_eq!(registers.flag_z(), 0);
        assert_eq!(registers.flag_h(), 1);
        assert_eq!(registers.flag_n(), 0);
        assert_eq!(registers.pc(), 7);
    }

    #[test]
    fn set_3_a() {
        let rom = vec![
            0b00_111_110,
            0x80,           // LD A, 0x80
            0b11_001_011,
            0b11_011_111    // SET 3, A

        ];
        let mut cpu = create_cpu(rom);
        run_steps_without_wait_cycles(2, &mut cpu);
        let registers = &cpu.registers;
        assert_eq!(registers.a(), 0x88);
        assert_eq!(registers.pc(), 4);
    }

    #[test]
    fn set_4_l() {
        let rom = vec![
            0b00_101_110,
            0x3B,           // LD L, 0x3B
            0b11_001_011,
            0b11_111_101    // SET 7, L

        ];
        let mut cpu = create_cpu(rom);
        run_steps_without_wait_cycles(2, &mut cpu);
        let registers = &cpu.registers;
        assert_eq!(registers.l(), 0xBB);
        assert_eq!(registers.pc(), 4);
    }

    #[test]
    fn set_0_mem_hl() {
        let rom = vec![
            0b00_100_001,
            0x00,
            0x80,          // LD HL, 0x8000
            0b00_110_110,
            0x00,          // LD (HL), 0x00
            0b11_001_011,
            0b11_011_110   // SET 3, (HL)

        ];
        let mut cpu = create_cpu(rom);
        run_steps_without_wait_cycles(3, &mut cpu);
        let registers = &cpu.registers;
        assert_eq!(read_memory(&cpu, 0x8000), 0x08);
        assert_eq!(registers.pc(), 7);
    }

    #[test]
    fn res_7_a() {
        let rom = vec![
            0b00_111_110,
            0x80,           // LD A, 0x80
            0b11_001_011,
            0b10_111_111    // RES 7, A

        ];
        let mut cpu = create_cpu(rom);
        run_steps_without_wait_cycles(2, &mut cpu);
        let registers = &cpu.registers;
        assert_eq!(registers.a(), 0x0);
        assert_eq!(registers.pc(), 4);
    }

    #[test]
    fn set_1_l() {
        let rom = vec![
            0b00_101_110,
            0x3B,           // LD L, 0x39
            0b11_001_011,
            0b10_001_101    // RES 1, L

        ];
        let mut cpu = create_cpu(rom);
        run_steps_without_wait_cycles(2, &mut cpu);
        let registers = &cpu.registers;
        assert_eq!(registers.l(), 0x39);
        assert_eq!(registers.pc(), 4);
    }

    #[test]
    fn res_3_mem_hl() {
        let rom = vec![
            0b00_100_001,
            0x00,
            0x80,          // LD HL, 0x8000
            0b00_110_110,
            0xFF,          // LD (HL), 0xFF
            0b11_001_011,
            0b10_011_110   // RES 3, (HL)
        ];
        let mut cpu = create_cpu(rom);
        run_steps_without_wait_cycles(3, &mut cpu);
        let registers = &cpu.registers;
        assert_eq!(read_memory(&cpu, 0x8000), 0xF7);
        assert_eq!(registers.pc(), 7);
    }

    #[test]
    fn jp_0x8000() {
        let rom = vec![
            0b11_000_011,
            0x00,
            0x80,          // JP 0x8000

        ];
        let mut cpu = create_cpu(rom);
        run_steps_without_wait_cycles(1, &mut cpu);
        let registers = &cpu.registers;
        assert_eq!(registers.pc(), 0x8000);
    }

    #[test]
    fn jp_nz_0x8000() {
        let rom = vec![
            0b00_111_110,
            0x00,           // LD A, 0x00
            0b11_001_011,
            0b00_110_111,    // SWAP A
            0b11_000_010,
            0x00,
            0x80,          // JP NZ 0x8000

        ];
        let mut cpu = create_cpu(rom);
        run_steps_without_wait_cycles(3, &mut cpu);
        let registers = &cpu.registers;
        assert_eq!(registers.pc(), 7);
    }

    #[test]
    fn jp_z_0x8000() {
        let rom = vec![
            0b00_111_110,
            0x00,           // LD A, 0x00
            0b11_001_011,
            0b00_110_111,    // SWAP A
            0b11_001_010,
            0x00,
            0x80,          // JP NZ 0x8000

        ];
        let mut cpu = create_cpu(rom);
        run_steps_without_wait_cycles(3, &mut cpu);
        let registers = &cpu.registers;
        assert_eq!(registers.pc(), 0x8000);
    }

    #[test]
    fn jp_c_0x8000() {
        let rom = vec![
            0b00_111_110,
            0x00,           // LD A, 0x00
            0b11_001_011,
            0b00_110_111,    // SWAP A
            0b11_011_010,
            0x00,
            0x80,          // JP NZ 0x8000

        ];
        let mut cpu = create_cpu(rom);
        run_steps_without_wait_cycles(3, &mut cpu);
        let registers = &cpu.registers;
        assert_eq!(registers.pc(), 7);
    }

    #[test]
    fn jp_nc_0x8000() {
        let rom = vec![
            0b00_111_110,
            0x00,           // LD A, 0x00
            0b11_001_011,
            0b00_110_111,    // SWAP A
            0b11_010_010,
            0x00,
            0x80,          // JP NZ 0x8000

        ];
        let mut cpu = create_cpu(rom);
        run_steps_without_wait_cycles(3, &mut cpu);
        let registers = &cpu.registers;
        assert_eq!(registers.pc(), 0x8000);
    }

    #[test]
    fn jr_neg_5() {
        let rom = vec![
            0x00,           // NOP
            0x00,           // NOP
            0x00,           // NOP
            0b00_011_000,
            -5i8 as u8,              // JP -5

        ];
        let mut cpu = create_cpu(rom);
        run_steps_without_wait_cycles(4, &mut cpu);
        let registers = &cpu.registers;
        assert_eq!(registers.pc(), 0);
    }

    #[test]
    fn jr_pos_5() {
        let rom = vec![
            0b00_011_000,
            5,              // JP 5

        ];
        let mut cpu = create_cpu(rom);
        run_steps_without_wait_cycles(1, &mut cpu);
        let registers = &cpu.registers;
        assert_eq!(registers.pc(), 7);
    }

    #[test]
    fn jr_z_neg_5() {
        let rom = vec![
            0b00_111_110,
            0x00,           // LD A, 0x00
            0b11_001_011,
            0b00_110_111,   // SWAP A
            0b00_101_000,
            -5i8 as u8,     // JR Z, -5

        ];
        let mut cpu = create_cpu(rom);
        run_steps_without_wait_cycles(3, &mut cpu);
        let registers = &cpu.registers;
        assert_eq!(registers.pc(), 1);
    }

    #[test]
    fn jr_nc_pos_5() {
        let rom = vec![
            0b00_111_110,
            0x00,           // LD A, 0x00
            0b11_001_011,
            0b00_110_111,   // SWAP A
            0b00_110_000,
            5,              // JR NC, 5

        ];
        let mut cpu = create_cpu(rom);
        run_steps_without_wait_cycles(3, &mut cpu);
        let registers = &cpu.registers;
        assert_eq!(registers.pc(), 11);
    }

    #[test]
    fn jp_mem_hl() {
        let rom = vec![
            0b00_100_001,
            0x00,
            0x80,          // LD HL, 0x8000
            0b11_101_001   // JP (HL)
        ];
        let mut cpu = create_cpu(rom);
        run_steps_without_wait_cycles(2, &mut cpu);
        let registers = &cpu.registers;
        assert_eq!(registers.pc(), 0x8000);
    }

    #[test]
    fn call_0x1234() {
        let rom = vec![
            0b11_000_011,
            0x00,
            0x80,          // JP 0x8000
        ];
        let mut cpu = create_cpu(rom);
        write_memory(&mut cpu, 0x8000, 0b11_001_101);
        write_memory(&mut cpu, 0x8001, 0x34);
        write_memory(&mut cpu, 0x8002, 0x12);         // CALL 0x1234

        run_steps_without_wait_cycles(2, &mut cpu);
        let registers = &cpu.registers;
        assert_eq!(registers.pc(), 0x1234);
        assert_eq!(registers.sp(), 0xFFFC);
        assert_eq!(read_memory(&cpu, 0xFFFD), 0x80);
        assert_eq!(read_memory(&cpu, 0xFFFC), 0x03);
    }

    #[test]
    fn call_z_0x1234() {
        let rom = vec![
            0b00_111_110,
            0x00,           // LD A, 0x00
            0b11_001_011,
            0b00_110_111,   // SWAP A
            0b11_000_011,
            0x00,
            0x80,          // JP 0x7FFC

        ];
        let mut cpu = create_cpu(rom);
        write_memory(&mut cpu, 0x8000, 0b11_000_100);
        write_memory(&mut cpu, 0x8001, 0xFF);
        write_memory(&mut cpu, 0x8002, 0xFF);         // CALL NZ, 0xFFFF
        write_memory(&mut cpu, 0x8003, 0b11_001_100);
        write_memory(&mut cpu, 0x8004, 0x34);
        write_memory(&mut cpu, 0x8005, 0x12);         // CALL Z, 0x1234

        run_steps_without_wait_cycles(5, &mut cpu);
        let registers = &cpu.registers;
        assert_eq!(registers.pc(), 0x1234);
        assert_eq!(registers.sp(), 0xFFFC);
        assert_eq!(read_memory(&cpu, 0xFFFD), 0x80);
        assert_eq!(read_memory(&cpu, 0xFFFC), 0x06);
    }

    #[test]
    fn call_0x8000_ret() {
        let rom = vec![
            0b11_000_011,
            0x00,
            0x80,          // JP 0x8000
        ];
        let mut cpu = create_cpu(rom);
        write_memory(&mut cpu, 0x8000, 0b11_001_101);
        write_memory(&mut cpu, 0x8001, 0x00);
        write_memory(&mut cpu, 0x8002, 0x90);         // CALL 0x1234
        write_memory(&mut cpu, 0x9000, 0b00_000_000); // NOP
        write_memory(&mut cpu, 0x9001, 0b11_001_001); // RET
        run_steps_without_wait_cycles(4, &mut cpu);
        let registers = &cpu.registers;
        assert_eq!(registers.pc(), 0x8003);
        assert_eq!(registers.sp(), 0xFFFE);
    }

    #[test]
    fn call_0x8000_reti() {
        let rom = vec![
            0b11_110_011,  // DI
            0b11_000_011,
            0x00,
            0x80,          // JP 0x8000
        ];
        let mut cpu = create_cpu(rom);
        write_memory(&mut cpu, 0x8000, 0b11_001_101);
        write_memory(&mut cpu, 0x8001, 0x00);
        write_memory(&mut cpu, 0x8002, 0x90);         // CALL 0x1234
        write_memory(&mut cpu, 0x9000, 0b00_000_000); // NOP
        write_memory(&mut cpu, 0x9001, 0b11_011_001); // RETI

        run_steps_without_wait_cycles(5, &mut cpu);
        let registers = &cpu.registers;
        assert_eq!(registers.pc(), 0x8003);
        assert_eq!(registers.sp(), 0xFFFE);
        assert_eq!(cpu.interrupt.master_enable, true);
    }

    #[test]
    fn call_0x8000_nz_ret() {
        let rom = vec![
            0b00_111_110,
            0x00,           // LD A, 0x00
            0b11_001_011,
            0b00_110_111,   // SWAP A
            0b11_000_011,
            0x00,
            0x80,          // JP 0x8000
        ];
        let mut cpu = create_cpu(rom);
        write_memory(&mut cpu, 0x8000, 0b11_001_101);
        write_memory(&mut cpu, 0x8001, 0x00);
        write_memory(&mut cpu, 0x8002, 0x90);         // CALL 0x1234
        write_memory(&mut cpu, 0x9000, 0b00_000_000); // NOP
        write_memory(&mut cpu, 0x9001, 0b11_000_000); // RET NZ

        run_steps_without_wait_cycles(6, &mut cpu);
        let registers = &cpu.registers;
        assert_eq!(registers.pc(), 0x9002);
        assert_eq!(registers.sp(), 0xFFFC);
    }

    #[test]
    fn call_0x8000_z_ret() {
        let rom = vec![
            0b00_111_110,
            0x00,           // LD A, 0x00
            0b11_001_011,
            0b00_110_111,   // SWAP A
            0b11_000_011,
            0x00,
            0x80,          // JP 0x8000
        ];
        let mut cpu = create_cpu(rom);
        write_memory(&mut cpu, 0x8000, 0b11_001_101);
        write_memory(&mut cpu, 0x8001, 0x00);
        write_memory(&mut cpu, 0x8002, 0x90);         // CALL 0x1234
        write_memory(&mut cpu, 0x9000, 0b00_000_000); // NOP
        write_memory(&mut cpu, 0x9001, 0b11_001_000); // RET Z

        run_steps_without_wait_cycles(6, &mut cpu);
        let registers = &cpu.registers;
        assert_eq!(registers.pc(), 0x8003);
        assert_eq!(registers.sp(), 0xFFFE);
    }

    #[test]
    fn rst_0() {
        let rom = vec![
            0b11_000_011,
            0x00,
            0x80,          // JP 0x8000
        ];
        let mut cpu = create_cpu(rom);
        write_memory(&mut cpu, 0x8000, 0b11_000_111); // RST 0

        run_steps_without_wait_cycles(2, &mut cpu);
        let registers = &cpu.registers;
        assert_eq!(registers.pc(), 0x0000);
        assert_eq!(registers.sp(), 0xFFFC);
        assert_eq!(read_memory(&cpu, 0xFFFD), 0x80);
        assert_eq!(read_memory(&cpu, 0xFFFC), 0x01);
    }

    #[test]
    fn rst_1() {
        let rom = vec![
            0b11_000_011,
            0x00,
            0x80,          // JP 0x8000
        ];
        let mut cpu = create_cpu(rom);
        write_memory(&mut cpu, 0x8000, 0b11_001_111); // RST 1

        run_steps_without_wait_cycles(2, &mut cpu);
        let registers = &cpu.registers;
        assert_eq!(registers.pc(), 0x0008);
        assert_eq!(registers.sp(), 0xFFFC);
        assert_eq!(read_memory(&cpu, 0xFFFD), 0x80);
        assert_eq!(read_memory(&cpu, 0xFFFC), 0x01);
    }

    #[test]
    fn rst_2() {
        let rom = vec![
            0b11_000_011,
            0x00,
            0x80,          // JP 0x8000
        ];
        let mut cpu = create_cpu(rom);
        write_memory(&mut cpu, 0x8000, 0b11_010_111); // RST 2
        run_steps_without_wait_cycles(2, &mut cpu);
        let registers = &cpu.registers;
        assert_eq!(registers.pc(), 0x0010);
        assert_eq!(registers.sp(), 0xFFFC);
        assert_eq!(read_memory(&cpu, 0xFFFD), 0x80);
        assert_eq!(read_memory(&cpu, 0xFFFC), 0x01);
    }

    #[test]
    fn rst_3() {
        let rom = vec![
            0b11_000_011,
            0x00,
            0x80,          // JP 0x8000
        ];
        let mut cpu = create_cpu(rom);
        write_memory(&mut cpu, 0x8000, 0b11_011_111); // RST 3

        run_steps_without_wait_cycles(2, &mut cpu);
        let registers = &cpu.registers;
        assert_eq!(registers.pc(), 0x0018);
        assert_eq!(registers.sp(), 0xFFFC);
        assert_eq!(read_memory(&cpu, 0xFFFD), 0x80);
        assert_eq!(read_memory(&cpu, 0xFFFC), 0x01);
    }

    #[test]
    fn rst_4() {
        let rom = vec![
            0b11_000_011,
            0x00,
            0x80,          // JP 0x8000
        ];
        let mut cpu = create_cpu(rom);
        write_memory(&mut cpu, 0x8000, 0b11_100_111); // RST 4

        run_steps_without_wait_cycles(2, &mut cpu);
        let registers = &cpu.registers;
        assert_eq!(registers.pc(), 0x0020);
        assert_eq!(registers.sp(), 0xFFFC);
        assert_eq!(read_memory(&cpu, 0xFFFD), 0x80);
        assert_eq!(read_memory(&cpu, 0xFFFC), 0x01);
    }

    #[test]
    fn rst_5() {
        let rom = vec![
            0b11_000_011,
            0x00,
            0x80,          // JP 0x8000
        ];
        let mut cpu = create_cpu(rom);
        write_memory(&mut cpu, 0x8000, 0b11_101_111); // RST 5

        run_steps_without_wait_cycles(2, &mut cpu);
        let registers = &cpu.registers;
        assert_eq!(registers.pc(), 0x0028);
        assert_eq!(registers.sp(), 0xFFFC);
        assert_eq!(read_memory(&cpu, 0xFFFD), 0x80);
        assert_eq!(read_memory(&cpu, 0xFFFC), 0x01);
    }

    #[test]
    fn rst_6() {
        let rom = vec![
            0b11_000_011,
            0x00,
            0x80,          // JP 0x8000
        ];
        let mut cpu = create_cpu(rom);
        write_memory(&mut cpu, 0x8000, 0b11_110_111); // RST 6

        run_steps_without_wait_cycles(2, &mut cpu);
        let registers = &cpu.registers;
        assert_eq!(registers.pc(), 0x0030);
        assert_eq!(registers.sp(), 0xFFFC);
        assert_eq!(read_memory(&cpu, 0xFFFD), 0x80);
        assert_eq!(read_memory(&cpu, 0xFFFC), 0x01);
    }

    #[test]
    fn rst_7() {
        let rom = vec![
            0b11_000_011,
            0x00,
            0x80,          // JP 0x8000
        ];
        let mut cpu = create_cpu(rom);
        write_memory(&mut cpu, 0x8000, 0b11_111_111); // RST 7

        run_steps_without_wait_cycles(2, &mut cpu);
        let registers = &cpu.registers;
        assert_eq!(registers.pc(), 0x0038);
        assert_eq!(registers.sp(), 0xFFFC);
        assert_eq!(read_memory(&cpu, 0xFFFD), 0x80);
        assert_eq!(read_memory(&cpu, 0xFFFC), 0x01);
    }

    #[test]
    fn cpl() {
        let rom = vec![
            0b00_111_110,
            0x35,           // LD A, 0x00
            0b00_101_111    // CPL
        ];
        let mut cpu = create_cpu(rom);

        run_steps_without_wait_cycles(2, &mut cpu);
        let registers = &cpu.registers;
        assert_eq!(registers.a(), 0xCA);
        assert_eq!(registers.pc(), 3);
    }

    #[test]
    fn scf() {
        let rom = vec![
            0b00_111_111,   // CCF
            0b00_110_111,   // SCF
        ];
        let mut cpu = create_cpu(rom);

        run_steps_without_wait_cycles(2, &mut cpu);
        let registers = &cpu.registers;
        assert_eq!(registers.flag_cy(), 1);
        assert_eq!(registers.pc(), 2);
    }

    #[test]
    fn ccf() {
        let rom = vec![
            0b00_110_111,   // SCF
            0b00_111_111,   // CCF
        ];
        let mut cpu = create_cpu(rom);

        run_steps_without_wait_cycles(2, &mut cpu);
        let registers = &cpu.registers;
        assert_eq!(registers.flag_cy(), 0);
        assert_eq!(registers.pc(), 2);
    }

    #[test]
    fn nop() {
        let rom = vec![
            0b00_000_000,   // SCF
            0b00_000_000,   // SCF
            0b00_000_000,   // SCF
            0b00_000_000,   // SCF
            0b00_000_000,   // SCF
        ];
        let mut cpu = create_cpu(rom);
        run_steps_without_wait_cycles(5, &mut cpu);
        let registers = &cpu.registers;
        assert_eq!(registers.pc(), 5);
    }

    #[test]
    fn di() {
        let rom = vec![
            0b11_111_011,  // EI
            0b11_110_011,  // DI
        ];
        let mut cpu = create_cpu(rom);
        run_steps_without_wait_cycles(2, &mut cpu);
        let registers = &cpu.registers;
        assert_eq!(cpu.interrupt.master_enable, false);
        assert_eq!(registers.pc(), 2);
    }

    #[test]
    fn ei() {
        let rom = vec![
            0b11_110_011,  // DI
            0b11_111_011,  // EI
        ];
        let mut cpu = create_cpu(rom);
        run_steps_without_wait_cycles(2, &mut cpu);
        let registers = &cpu.registers;
        assert_eq!(cpu.interrupt.master_enable, true);
        assert_eq!(registers.pc(), 2);
    }
}