use std::sync::{Arc, RwLock};

use registers::Registers;

use opcodes;

use memory::Memory;
use interrupt_controller::InterruptController;



pub struct CPU{
    registers: Registers,
    interrupt: Arc<RwLock<InterruptController>>,
    memory: Arc<RwLock<Memory>>
}

impl CPU {
    pub fn new(interrupt: Arc<RwLock<InterruptController>>, memory: Arc<RwLock<Memory>>, boot_sequence: bool) -> CPU {
        CPU { registers: Registers::new(boot_sequence), interrupt, memory }
    }

    pub fn step(&mut self) {
        let pc = self.registers.pc();
        let opcode = self.memory.read().unwrap().read(pc);
        opcodes::execute(opcode, pc, &mut self.registers, &mut self.memory);
    }
}



#[cfg(test)]
mod tests {
    extern crate simplelog;

    use std::sync::{Arc, RwLock};
    use cpu::CPU;
    use rom::ROM;
    use memory::Memory;
    use interrupt_controller::InterruptController;
    use simplelog::CombinedLogger;
    use simplelog::TermLogger;
    use log::LevelFilter;
    use simplelog::Config;
    use simplelog::SimpleLogger;
    use simplelog::WriteLogger;
    use std::io;
    use simplelog::TestLogger;

    fn create_cpu(rom: ROM) -> (CPU, Arc<RwLock<Memory>>){
        let logger = TestLogger::init(LevelFilter::Debug, Config::default());
        if logger.is_ok() {
            logger.unwrap();
        }
        let memory = Arc::new(RwLock::new(Memory::new(rom, false)));
        let interrupt = Arc::new(RwLock::new(InterruptController::new()));
        let mut cpu = CPU::new(interrupt, memory.clone(), false);
        cpu.registers.set_pc(0);
        cpu.registers.set_f(0x0);
        (cpu, memory)
    }

    fn run_steps(steps: usize, cpu: &mut CPU) {
        for _ in 0..steps {
            cpu.step();
        }
    }

    #[test]
    fn ld_a_b() {
        let rom = ROM::new(vec![
            0b00_111_110,
            0b00_000_001,   // LD A, 1
            0b00_000_110,
            0b00_000_010,   // LD B, 2
            0b01_111_000    // LD A, B
        ]);
        let (mut cpu, _) = create_cpu(rom);
        run_steps(3, &mut cpu);
        let registers = &cpu.registers;
        assert_eq!(registers.a(), 0b00000010);
        assert_eq!(registers.pc(), 5);
    }

    #[test]
    fn ld_b_d() {
        let rom = ROM::new(vec![
            0b00_000_110,
            0b00_000_001,   // LD B, 1
            0b00_010_110,
            0b00_000_010,   // LD D, 2
            0b01_000_010    // LD B, D
        ]);
        let (mut cpu, _) = create_cpu(rom);
        run_steps(3, &mut cpu);
        let registers = &cpu.registers;
        assert_eq!(registers.b(), 0b00000010);
        assert_eq!(registers.pc(), 5);
    }

    #[test]
    fn ld_b_24() {
        let rom = ROM::new(vec![
            0b00_000_110,
            0b00_011_000,   // LD B, 24
        ]);
        let (mut cpu, _) = create_cpu(rom);
        run_steps(1, &mut cpu);
        let registers = &cpu.registers;
        assert_eq!(registers.b(), 0b00011000);
        assert_eq!(registers.pc(), 2);
    }

    #[test]
    fn ld_h_mem_hl() {
        let rom = ROM::new(vec![
            0b00_101_110,
            0x00,         // LD L, 1
            0b00_100_110,
            0xA0,         // LD H, 1
            0b00_110_110,
            0x5C,         // LD HL, 24
            0b01_100_110  // LD H, (HL)
        ]);
        let (mut cpu, _) = create_cpu(rom);
        run_steps(4, &mut cpu);
        let registers = &cpu.registers;
        assert_eq!(registers.h(), 0x5C);
        assert_eq!(registers.pc(), 7);
    }

    #[test]
    fn ld_mem_hl_a() {
        let rom = ROM::new(vec![
            0b00_111_110,
            0x3C,         // LD A, 0x3C
            0b00_100_001,
            0xC5,
            0x8A,         // LD HL, 0x8AC5
            0b01_110_111  // LD (HL), A
        ]);
        let (mut cpu, memory) = create_cpu(rom);
        run_steps(3, &mut cpu);
        let registers = &cpu.registers;
        assert_eq!(memory.read().unwrap().read(0x8AC5), 0x3C);
        assert_eq!(registers.pc(), 6);
    }

    #[test]
    fn ld_mem_hl_24() {
        let rom = ROM::new(vec![
            0b00_100_001,
            0xC5,
            0x8A,         // LD HL, 0x8AC5
            0b_00_110_110,
            13              // LD (HL), 13
        ]);
        let (mut cpu, memory) = create_cpu(rom);
        run_steps(2, &mut cpu);
        let registers = &cpu.registers;
        assert_eq!(memory.read().unwrap().read(0x8AC5), 13);
        assert_eq!(registers.pc(), 5);
    }

    #[test]
    fn ld_a_mem_bc() {
        let rom = ROM::new(vec![
            0b00_100_001,
            0xC5,
            0x8A,          // LD HL, 0x8AC5
            0b_00_110_110,
            0x2F,            // LD (HL), 0x2F
            0b00_000_001,
            0xC5,
            0x8A,          // LD BC, 0x8AC5
            0b00_001_010   // LD A, (BC)

        ]);
        let (mut cpu, _) = create_cpu(rom);
        run_steps(4, &mut cpu);
        let registers = &cpu.registers;
        assert_eq!(registers.a(), 0x2F);
        assert_eq!(registers.pc(), 9);
    }

    #[test]
    fn ld_a_mem_de() {
        let rom = ROM::new(vec![
            0b00_100_001,
            0xC5,
            0x8A,          // LD HL, 0x8AC5
            0b_00_110_110,
            0x5F,            // LD (HL), 0x5F
            0b00_010_001,
            0xC5,
            0x8A,          // LD DE, 0x8AC5
            0b00_011_010   // LD A, (DE)

        ]);
        let (mut cpu, _) = create_cpu(rom);
        run_steps(4, &mut cpu);
        let registers = &cpu.registers;
        assert_eq!(registers.a(), 0x5F);
        assert_eq!(registers.pc(), 9);
    }

    #[test]
    fn ld_a_mem_c() {
        let rom = ROM::new(vec![
            0b00_100_001,
            0x95,
            0xFF,          // LD HL, 0xFF95
            0b_00_110_110,
            0x21,          // LD (HL), 0x21
            0b00_001_110,
            0x95,          // LD C, 0x95
            0b11_110_010   // LD A, (C)

        ]);
        let (mut cpu, _) = create_cpu(rom);
        run_steps(4, &mut cpu);
        let registers = &cpu.registers;
        assert_eq!(registers.a(), 0x21);
        assert_eq!(registers.pc(), 8);
    }

    #[test]
    fn ld_mem_c_a() {
        let rom = ROM::new(vec![
            0b00_111_110,
            0x3C,         // LD A, 0x3C
            0b00_001_110,
            0x9F,         // LD C, 0x9F
            0b11_100_010  // LD (C), A
        ]);
        let (mut cpu, memory) = create_cpu(rom);
        run_steps(3, &mut cpu);
        let registers = &cpu.registers;
        assert_eq!(memory.read().unwrap().read(0xFF9F), 0x3C);
        assert_eq!(registers.pc(), 5);
    }

    #[test]
    fn ld_a_mem_34() {
        let rom = ROM::new(vec![
            0b00_111_110,
            0x3C,         // LD A, 0x3C
            0b00_001_110,
            0x9F,         // LD C, 0x9F
            0b11_100_010, // LD (C), A
            0b00_111_110,
            0,            // LD A, 0
            0b11_110_000,
            0x9F        // LD A, (n)
        ]);
        let (mut cpu, _) = create_cpu(rom);
        run_steps(5, &mut cpu);
        let registers = &cpu.registers;
        assert_eq!(registers.a(), 0x3C);
        assert_eq!(registers.pc(), 9);
    }

    #[test]
    fn ld_mem_n_a() {
        let rom = ROM::new(vec![
            0b00_111_110,
            0x3C,         // LD A, 0x3C
            0b11_100_000,
            0x34          // LD (n), A
        ]);
        let (mut cpu, memory) = create_cpu(rom);
        run_steps(2, &mut cpu);
        let registers = &cpu.registers;
        assert_eq!(memory.read().unwrap().read(0xFF34), 0x3C);
        assert_eq!(registers.pc(), 4);
    }

    #[test]
    fn ld_a_mem_nn() {
        let rom = ROM::new(vec![
            0b00_100_001,
            0x44,
            0xFF,          // LD HL, 0xFF44
            0b_00_110_110,
            0x2F,          // LD (HL), 0x2F
            0b_11_111_010,
            0x44,
            0xFF,          // LD A, (nn)
        ]);
        let (mut cpu, _) = create_cpu(rom);
        run_steps(3, &mut cpu);
        let registers = &cpu.registers;
        assert_eq!(registers.a(), 0x2F);
        assert_eq!(registers.pc(), 8);
    }

    #[test]
    fn ld_mem_nn_a() {
        let rom = ROM::new(vec![
            0b00_111_110,
            0x3A,          // LD A, 0x3A
            0b11_101_010,
            0x44,
            0xFF,          // LD (nn), A
        ]);
        let (mut cpu, memory) = create_cpu(rom);
        run_steps(2, &mut cpu);
        let registers = &cpu.registers;
        assert_eq!(memory.read().unwrap().read(0xFF44), 0x3A);
        assert_eq!(registers.pc(), 5);
    }

    #[test]
    fn ld_a_mem_hli() {
        let rom = ROM::new(vec![
            0b00_100_001,
            0x08,
            0x80,          // LD HL, 0x8008
            0b_00_110_110,
            0x77,          // LD (HL), 0x77
            0b00_101_010   // LD A, (HLI)
        ]);
        let (mut cpu, _) = create_cpu(rom);
        run_steps(3, &mut cpu);
        let registers = &cpu.registers;
        assert_eq!(registers.a(), 0x77);
        assert_eq!(registers.hl(), 0x8009);
        assert_eq!(registers.pc(), 6);
    }

    #[test]
    fn ld_a_mem_hld() {
        let rom = ROM::new(vec![
            0b00_100_001,
            0x08,
            0x80,          // LD HL, 0x8008
            0b_00_110_110,
            0x77,          // LD (HL), 0x77
            0b00_111_010   // LD A, (HLD)
        ]);
        let (mut cpu, _) = create_cpu(rom);
        run_steps(3, &mut cpu);
        let registers = &cpu.registers;
        assert_eq!(registers.a(), 0x77);
        assert_eq!(registers.hl(), 0x8007);
        assert_eq!(registers.pc(), 6);
    }

    #[test]
    fn ld_mem_bc_a() {
        let rom = ROM::new(vec![
            0b00_000_001,
            0x00,
            0x80,         // LD BC, 0x8000
            0b00_111_110,
            0xAB,         // LD A, 0x3A
            0b00_000_010  // LD (BC), A
        ]);
        let (mut cpu, memory) = create_cpu(rom);
        run_steps(3, &mut cpu);
        let registers = &cpu.registers;
        assert_eq!(memory.read().unwrap().read(0x8000), 0xAB);
        assert_eq!(registers.pc(), 6);
    }

    #[test]
    fn ld_mem_de_a() {
        let rom = ROM::new(vec![
            0b00_010_001,
            0x00,
            0x80,         // LD DE, 0x8000
            0b00_111_110,
            0xAD,         // LD A, 0x3A
            0b00_010_010  // LD (DE), A
        ]);
        let (mut cpu, memory) = create_cpu(rom);
        run_steps(3, &mut cpu);
        let registers = &cpu.registers;
        assert_eq!(memory.read().unwrap().read(0x8000), 0xAD);
        assert_eq!(registers.pc(), 6);
    }

    #[test]
    fn ld_mem_hli_a() {
        let rom = ROM::new(vec![
            0b00_111_110,
            0x96,          // LD A, 0x96
            0b00_100_001,
            0x08,
            0x80,          // LD HL, 0x8008
            0b00_100_010   // LD (HLI), A

        ]);
        let (mut cpu, memory) = create_cpu(rom);
        run_steps(3, &mut cpu);
        let registers = &cpu.registers;
        assert_eq!(registers.hl(), 0x8009);
        assert_eq!(memory.read().unwrap().read(0x8008), 0x96);
        assert_eq!(registers.pc(), 6);
    }

    #[test]
    fn ld_mem_hld_a() {
        let rom = ROM::new(vec![
            0b00_111_110,
            0x97,          // LD A, 0x96
            0b00_100_001,
            0x08,
            0x80,          // LD HL, 0x8008
            0b00_110_010   // LD (HLD), A

        ]);
        let (mut cpu, memory) = create_cpu(rom);
        run_steps(3, &mut cpu);
        let registers = &cpu.registers;
        assert_eq!(registers.hl(), 0x8007);
        assert_eq!(memory.read().unwrap().read(0x8008), 0x97);
        assert_eq!(registers.pc(), 6);
    }

    #[test]
    fn ld_hl_0x3a5b() {
        let rom = ROM::new(vec![
            0b00_100_001,
            0x5B,
            0x3A          // LD HL, 0x3A5B

        ]);
        let (mut cpu, _) = create_cpu(rom);
        run_steps(1, &mut cpu);
        let registers = &cpu.registers;
        assert_eq!(registers.hl(), 0x3A5B);
        assert_eq!(registers.pc(), 3);
    }

    #[test]
    fn ld_sp_hl() {
        let rom = ROM::new(vec![
            0b00_100_001,
            0x5B,
            0x3A,         // LD HL, 0x3A5B
            0b11_111_001  // LD SP, HL
        ]);
        let (mut cpu, _) = create_cpu(rom);
        run_steps(2, &mut cpu);
        let registers = &cpu.registers;
        assert_eq!(registers.sp(), 0x3A5B);
        assert_eq!(registers.pc(), 4);
    }

    #[test]
    fn push_hl() {
        let rom = ROM::new(vec![
            0b00_100_001,
            0x5B,
            0x3A,         // LD HL, 0x3A5B
            0b11_100_101, // PUSH HL
        ]);
        let (mut cpu, memory) = create_cpu(rom);
        let old_sp = {cpu.registers.sp()};
        run_steps(2, &mut cpu);
        let registers = &cpu.registers;
        let sp = registers.sp();
        assert_eq!(old_sp, 0xFFFE);
        assert_eq!(memory.read().unwrap().read(old_sp-1), 0x3A);
        assert_eq!(memory.read().unwrap().read(old_sp-2), 0x5B);
        assert_eq!(sp, old_sp-2);
        assert_eq!(registers.pc(), 4);
    }

    #[test]
    fn pop_bc() {
        let rom = ROM::new(vec![
            0b00_100_001,
            0x5B,
            0x3A,         // LD HL, 0x3A5B
            0b11_100_101, // PUSH HL

            0b11_000_001  // POP BC
        ]);
        let (mut cpu, _) = create_cpu(rom);
        run_steps(2, &mut cpu);
        let old_sp = cpu.registers.sp();
        run_steps(1, &mut cpu);
        let registers = &cpu.registers;
        let sp = registers.sp();
        assert_eq!(sp, old_sp+2);
        assert_eq!(registers.b(), 0x3A);
        assert_eq!(registers.c(), 0x5B);
        assert_eq!(registers.pc(), 5);
    }

    #[test]
    fn ldhl_sp_sub_5() {
        let rom = ROM::new(vec![
            0b00_110_001,
            0xF5,
            0xFF,           // LD SP, 0xFFF0
            0b11_111_000,
            0b11111011      // LDHL SP, -5
        ]);
        let (mut cpu, _) = create_cpu(rom);
        run_steps(2, &mut cpu);
        let registers = &cpu.registers;
        assert_eq!(registers.hl(), 0xFFF0);
        assert_eq!(registers.pc(), 5);
    }

    #[test]
    fn ldhl_sp_add_5() {
        let rom = ROM::new(vec![
            0b00_110_001,
            0xF8,
            0xFF,           // LD SP, 0xFFF8
            0b11_111_000,
            2               // LDHL SP, 2
        ]);
        let (mut cpu, _) = create_cpu(rom);
        run_steps(2, &mut cpu);
        let registers = &cpu.registers;
        assert_eq!(registers.get_flag_z(), 0);
        assert_eq!(registers.get_flag_h(), 0);
        assert_eq!(registers.get_flag_n(), 0);
        assert_eq!(registers.flag_cy(), 0);
        assert_eq!(registers.hl(), 0xFFFA);
        assert_eq!(registers.pc(), 5);
    }

    #[test]
    fn ld_mem_0xfff8_sp() {
        let rom = ROM::new(vec![
            0b00_110_001,
            0xF8,
            0xFF,           // LD SP, 0xFFF8
            0b00_001_000,
            0x00,
            0x80,           // LD (nn), SP
        ]);
        let (mut cpu, memory) = create_cpu(rom);
        run_steps(2, &mut cpu);
        let registers = &cpu.registers;
        assert_eq!(memory.read().unwrap().read(0x8000), 0xf8);
        assert_eq!(memory.read().unwrap().read(0x8001), 0xff);
        assert_eq!(registers.pc(), 6);
    }

    #[test]
    fn add_a_b() {
        let rom = ROM::new(vec![
            0b00_111_110,
            0x3A,           // LD A, 0x3A
            0b00_000_110,
            0xC6,           // LD B, 0xC6
            0b10_000_000    // ADD A, B
        ]);
        let (mut cpu, _) = create_cpu(rom);
        run_steps(3, &mut cpu);
        let registers = &cpu.registers;
        assert_eq!(registers.a(), 0x0);
        assert_eq!(registers.get_flag_z(), 1);
        assert_eq!(registers.get_flag_h(), 1);
        assert_eq!(registers.get_flag_n(), 0);
        assert_eq!(registers.flag_cy(), 1);
        assert_eq!(registers.pc(), 5);
    }

    #[test]
    fn add_a_0xff() {
        let rom = ROM::new(vec![
            0b00_111_110,
            0x3C,           // LD A, 0x3C
            0b11_000_110,
            0xFF            // ADD A, 0xFF
        ]);
        let (mut cpu, _) = create_cpu(rom);
        run_steps(2, &mut cpu);
        let registers = &cpu.registers;
        assert_eq!(registers.a(), 0x3B);
        assert_eq!(registers.get_flag_z(), 0);
        assert_eq!(registers.get_flag_h(), 1);
        assert_eq!(registers.get_flag_n(), 0);
        assert_eq!(registers.flag_cy(), 1);
        assert_eq!(registers.pc(), 4);
    }

    #[test]
    fn add_a_mem_hl() {
        let rom = ROM::new(vec![
            0b00_111_110,
            0x3C,           // LD A, 0x3C
            0b00_100_001,
            0x00,
            0x80,          // LD HL, 0x8000
            0b_00_110_110,
            0x12,          // LD (HL), 0x12
            0b10_000_110   // ADD A, (HL)
        ]);
        let (mut cpu, _) = create_cpu(rom);
        run_steps(4, &mut cpu);
        let registers = &cpu.registers;
        assert_eq!(registers.a(), 0x4E);
        assert_eq!(registers.get_flag_z(), 0);
        assert_eq!(registers.get_flag_h(), 0);
        assert_eq!(registers.get_flag_n(), 0);
        assert_eq!(registers.flag_cy(), 0);
        assert_eq!(registers.pc(), 8);
    }

    #[test]
    fn adc_a_e() {
        let rom = ROM::new(vec![
            0b00_111_110,
            0x3A,           // LD A, 0x3A
            0b11_000_110,
            0xC6,           // ADD A, 0xC6
            0b00_111_110,
            0xE1,           // LD A, 0xE1
            0b00_011_110,
            0x0F,           // LD E, 0x0F
            0b10_001_011    // ADD A, E
        ]);
        let (mut cpu, _) = create_cpu(rom);
        run_steps(5, &mut cpu);
        let registers = &cpu.registers;
        assert_eq!(registers.a(), 0xF1);
        assert_eq!(registers.get_flag_z(), 0);
        assert_eq!(registers.get_flag_h(), 1);
        assert_eq!(registers.get_flag_n(), 0);
        assert_eq!(registers.flag_cy(), 0);
        assert_eq!(registers.pc(), 9);
    }

    #[test]
    fn adc_a_0x3b() {
        let rom = ROM::new(vec![
            0b00_111_110,
            0x3A,           // LD A, 0x3A
            0b11_000_110,
            0xC6,           // ADD A, 0xC6
            0b00_111_110,
            0xE1,           // LD A, 0xE1
            0b11_001_110,
            0x3B            // ADD A, 0x3B
        ]);
        let (mut cpu, _) = create_cpu(rom);
        run_steps(4, &mut cpu);
        let registers = &cpu.registers;
        assert_eq!(registers.a(), 0x1D);
        assert_eq!(registers.get_flag_z(), 0);
        assert_eq!(registers.get_flag_h(), 0);
        assert_eq!(registers.get_flag_n(), 0);
        assert_eq!(registers.flag_cy(), 1);
        assert_eq!(registers.pc(), 8);
    }

    #[test]
    fn adc_a_mem_hl() {
        let rom = ROM::new(vec![
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
        ]);
        let (mut cpu, _) = create_cpu(rom);
        run_steps(6, &mut cpu);
        let registers = &cpu.registers;
        assert_eq!(registers.a(), 0x0);
        assert_eq!(registers.get_flag_z(), 1);
        assert_eq!(registers.get_flag_h(), 1);
        assert_eq!(registers.get_flag_n(), 0);
        assert_eq!(registers.flag_cy(), 1);
        assert_eq!(registers.pc(), 12);
    }

    #[test]
    fn sub_a_h() {
        let rom = ROM::new(vec![
            0b00_111_110,
            0x3E,           // LD A, 0x3E
            0b00_100_110,
            0x3E,           // LD H, 0x3E
            0b10_010_100    // ADD A, H
        ]);
        let (mut cpu, _) = create_cpu(rom);
        run_steps(3, &mut cpu);
        let registers = &cpu.registers;
        assert_eq!(registers.a(), 0x0);
        assert_eq!(registers.get_flag_z(), 1);
        assert_eq!(registers.get_flag_h(), 0);
        assert_eq!(registers.get_flag_n(), 1);
        assert_eq!(registers.flag_cy(), 0);
        assert_eq!(registers.pc(), 5);
    }

    #[test]
    fn sub_a_0x0f() {
        let rom = ROM::new(vec![
            0b00_111_110,
            0x3E,           // LD A, 0x3E
            0b11_010_110,
            0x0F            // ADD A, 0x0F
        ]);
        let (mut cpu, _) = create_cpu(rom);
        run_steps(2, &mut cpu);
        let registers = &cpu.registers;
        assert_eq!(registers.a(), 0x2F);
        assert_eq!(registers.get_flag_z(), 0);
        assert_eq!(registers.get_flag_h(), 1);
        assert_eq!(registers.get_flag_n(), 1);
        assert_eq!(registers.flag_cy(), 0);
        assert_eq!(registers.pc(), 4);
    }

    #[test]
    fn sub_a_mem_hl() {
        let rom = ROM::new(vec![
            0b00_111_110,
            0x3E,           // LD A, 0x3E
            0b00_100_001,
            0x00,
            0x80,          // LD HL, 0x8000
            0b_00_110_110,
            0x40,          // LD (HL), 0x40
            0b10_010_110   // SUB A, (HL)
        ]);
        let (mut cpu, _) = create_cpu(rom);
        run_steps(4, &mut cpu);
        let registers = &cpu.registers;
        assert_eq!(registers.a(), 0xFE);
        assert_eq!(registers.get_flag_z(), 0);
        assert_eq!(registers.get_flag_h(), 0);
        assert_eq!(registers.get_flag_n(), 1);
        assert_eq!(registers.flag_cy(), 1);
        assert_eq!(registers.pc(), 8);
    }

    #[test]
    fn sbc_a_h() {
        let rom = ROM::new(vec![
            0b00_111_110,
            0x3A,           // LD A, 0x3A
            0b11_000_110,
            0xC6,           // ADD A, 0xC6
            0b00_111_110,
            0x3B,           // LD A, 0x3B
            0b00_100_110,
            0x2A,           // LD H, 0x2A
            0b10_011_100    // SBC A, E
        ]);
        let (mut cpu, _) = create_cpu(rom);
        run_steps(5, &mut cpu);
        let registers = &cpu.registers;
        assert_eq!(registers.a(), 0x10);
        assert_eq!(registers.get_flag_z(), 0);
        assert_eq!(registers.get_flag_h(), 0);
        assert_eq!(registers.get_flag_n(), 1);
        assert_eq!(registers.flag_cy(), 0);
        assert_eq!(registers.pc(), 9);
    }

    #[test]
    fn sbc_a_0x3a() {
        let rom = ROM::new(vec![
            0b00_111_110,
            0x3A,           // LD A, 0x3A
            0b11_000_110,
            0xC6,           // ADD A, 0xC6
            0b00_111_110,
            0x3B,           // LD A, 0x3B
            0b11_011_110,
            0x3A            // SBC A, 0x3A
        ]);
        let (mut cpu, _) = create_cpu(rom);
        run_steps(4, &mut cpu);
        let registers = &cpu.registers;
        assert_eq!(registers.a(), 0x00);
        assert_eq!(registers.get_flag_z(), 1);
        assert_eq!(registers.get_flag_h(), 0);
        assert_eq!(registers.get_flag_n(), 1);
        assert_eq!(registers.flag_cy(), 0);
        assert_eq!(registers.pc(), 8);
    }

    #[test]
    fn sbc_a_mem_hl() {
        let rom = ROM::new(vec![
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
        ]);
        let (mut cpu, _) = create_cpu(rom);
        run_steps(6, &mut cpu);
        let registers = &cpu.registers;
        assert_eq!(registers.a(), 0xEB);
        assert_eq!(registers.get_flag_z(), 0);
        assert_eq!(registers.get_flag_h(), 1);
        assert_eq!(registers.get_flag_n(), 1);
        assert_eq!(registers.flag_cy(), 1);
        assert_eq!(registers.pc(), 12);
    }

    #[test]
    fn and_a_l() {
        let rom = ROM::new(vec![
            0b00_111_110,
            0x5A,           // LD A, 0x5A
            0b00_101_110,
            0x3F,           // LD L, 0x3F
            0b10_100_101    // AND A, L
        ]);
        let (mut cpu, _) = create_cpu(rom);
        run_steps(3, &mut cpu);
        let registers = &cpu.registers;
        assert_eq!(registers.a(), 0x1A);
        assert_eq!(registers.get_flag_z(), 0);
        assert_eq!(registers.get_flag_h(), 1);
        assert_eq!(registers.get_flag_n(), 0);
        assert_eq!(registers.flag_cy(), 0);
        assert_eq!(registers.pc(), 5);
    }

    #[test]
    fn and_a_0x38() {
        let rom = ROM::new(vec![
            0b00_111_110,
            0x5A,           // LD A, 0x5A
            0b11_100_110,
            0x38            // AND A, 0x38
        ]);
        let (mut cpu, _) = create_cpu(rom);
        run_steps(2, &mut cpu);
        let registers = &cpu.registers;
        assert_eq!(registers.a(), 0x18);
        assert_eq!(registers.get_flag_z(), 0);
        assert_eq!(registers.get_flag_h(), 1);
        assert_eq!(registers.get_flag_n(), 0);
        assert_eq!(registers.flag_cy(), 0);
        assert_eq!(registers.pc(), 4);
    }

    #[test]
    fn and_a_mem_hl() {
        let rom = ROM::new(vec![
            0b00_111_110,
            0x5A,           // LD A, 0x5A
            0b00_100_001,
            0x00,
            0x80,          // LD HL, 0x8000
            0b00_110_110,
            0x00,          // LD (HL), 0x00
            0b10_100_110   // AND A, (HL)
        ]);
        let (mut cpu, _) = create_cpu(rom);
        run_steps(4, &mut cpu);
        let registers = &cpu.registers;
        assert_eq!(registers.a(), 0x00);
        assert_eq!(registers.get_flag_z(), 1);
        assert_eq!(registers.get_flag_h(), 1);
        assert_eq!(registers.get_flag_n(), 0);
        assert_eq!(registers.flag_cy(), 0);
        assert_eq!(registers.pc(), 8);
    }

    #[test]
    fn or_a_l() {
        let rom = ROM::new(vec![
            0b00_111_110,
            0x5A,           // LD A, 0x5A
            0b10_110_111    // OR A, A
        ]);
        let (mut cpu, _) = create_cpu(rom);
        run_steps(2, &mut cpu);
        let registers = &cpu.registers;
        assert_eq!(registers.a(), 0x5A);
        assert_eq!(registers.get_flag_z(), 0);
        assert_eq!(registers.get_flag_h(), 0);
        assert_eq!(registers.get_flag_n(), 0);
        assert_eq!(registers.flag_cy(), 0);
        assert_eq!(registers.pc(), 3);
    }

    #[test]
    fn or_a_0x03() {
        let rom = ROM::new(vec![
            0b00_111_110,
            0x5A,           // LD A, 0x5A
            0b11_110_110,
            0x03            // OR A, 0x03
        ]);
        let (mut cpu, _) = create_cpu(rom);
        run_steps(2, &mut cpu);
        let registers = &cpu.registers;
        assert_eq!(registers.a(), 0x5B);
        assert_eq!(registers.get_flag_z(), 0);
        assert_eq!(registers.get_flag_h(), 0);
        assert_eq!(registers.get_flag_n(), 0);
        assert_eq!(registers.flag_cy(), 0);
        assert_eq!(registers.pc(), 4);
    }

    #[test]
    fn or_a_mem_hl() {
        let rom = ROM::new(vec![
            0b00_111_110,
            0x5A,           // LD A, 0x5A
            0b00_100_001,
            0x00,
            0x80,          // LD HL, 0x8000
            0b00_110_110,
            0x00,          // LD (HL), 0x00
            0b10_110_110   // OR A, (HL)
        ]);
        let (mut cpu, _) = create_cpu(rom);
        run_steps(4, &mut cpu);
        let registers = &cpu.registers;
        assert_eq!(registers.a(), 0x5A);
        assert_eq!(registers.get_flag_z(), 0);
        assert_eq!(registers.get_flag_h(), 0);
        assert_eq!(registers.get_flag_n(), 0);
        assert_eq!(registers.flag_cy(), 0);
        assert_eq!(registers.pc(), 8);
    }

    #[test]
    fn xor_a_l() {
        let rom = ROM::new(vec![
            0b00_111_110,
            0xFF,           // LD A, 0xFF
            0b10_101_111    // XOR A, A
        ]);
        let (mut cpu, _) = create_cpu(rom);
        run_steps(2, &mut cpu);
        let registers = &cpu.registers;
        assert_eq!(registers.a(), 0x00);
        assert_eq!(registers.get_flag_z(), 1);
        assert_eq!(registers.get_flag_h(), 0);
        assert_eq!(registers.get_flag_n(), 0);
        assert_eq!(registers.flag_cy(), 0);
        assert_eq!(registers.pc(), 3);
    }

    #[test]
    fn xor_a_0x0f() {
        let rom = ROM::new(vec![
            0b00_111_110,
            0xFF,           // LD A, 0xFF
            0b11_101_110,
            0x0F            // XOR A, 0x0F
        ]);
        let (mut cpu, _) = create_cpu(rom);
        run_steps(2, &mut cpu);
        let registers = &cpu.registers;
        assert_eq!(registers.a(), 0xF0);
        assert_eq!(registers.get_flag_z(), 0);
        assert_eq!(registers.get_flag_h(), 0);
        assert_eq!(registers.get_flag_n(), 0);
        assert_eq!(registers.flag_cy(), 0);
        assert_eq!(registers.pc(), 4);
    }

    #[test]
    fn xor_a_mem_hl() {
        let rom = ROM::new(vec![
            0b00_111_110,
            0xFF,           // LD A, 0xFF
            0b00_100_001,
            0x00,
            0x80,          // LD HL, 0x8000
            0b00_110_110,
            0x8A,          // LD (HL), 0x8A
            0b10_101_110   // XOR A, (HL)
        ]);
        let (mut cpu, _) = create_cpu(rom);
        run_steps(4, &mut cpu);
        let registers = &cpu.registers;
        assert_eq!(registers.a(), 0x75);
        assert_eq!(registers.get_flag_z(), 0);
        assert_eq!(registers.get_flag_h(), 0);
        assert_eq!(registers.get_flag_n(), 0);
        assert_eq!(registers.flag_cy(), 0);
        assert_eq!(registers.pc(), 8);
    }

    #[test]
    fn cp_a_b() {
        let rom = ROM::new(vec![
            0b00_111_110,
            0x3C,           // LD A, 0x3C
            0b00_000_110,
            0x2F,           // LD B, 0x2F
            0b10_111_000    // CP A, B
        ]);
        let (mut cpu, _) = create_cpu(rom);
        run_steps(3, &mut cpu);
        let registers = &cpu.registers;
        assert_eq!(registers.a(), 0x3C);
        assert_eq!(registers.get_flag_z(), 0);
        assert_eq!(registers.get_flag_h(), 1);
        assert_eq!(registers.get_flag_n(), 1);
        assert_eq!(registers.flag_cy(), 0);
        assert_eq!(registers.pc(), 5);
    }

    #[test]
    fn cp_a_0x3c() {
        let rom = ROM::new(vec![
            0b00_111_110,
            0x3C,           // LD A, 0x3C
            0b11_111_110,
            0x3C            // CP A, 0x3C
        ]);
        let (mut cpu, _) = create_cpu(rom);
        run_steps(2, &mut cpu);
        let registers = &cpu.registers;
        assert_eq!(registers.a(), 0x3C);
        assert_eq!(registers.get_flag_z(), 1);
        assert_eq!(registers.get_flag_h(), 0);
        assert_eq!(registers.get_flag_n(), 1);
        assert_eq!(registers.flag_cy(), 0);
        assert_eq!(registers.pc(), 4);
    }

    #[test]
    fn cp_a_mem_hl() {
        let rom = ROM::new(vec![
            0b00_111_110,
            0x3C,           // LD A, 0x3C
            0b00_100_001,
            0x00,
            0x80,          // LD HL, 0x8000
            0b_00_110_110,
            0x40,          // LD (HL), 0x40
            0b10_111_110   // SUB A, (HL)
        ]);
        let (mut cpu, _) = create_cpu(rom);
        run_steps(4, &mut cpu);
        let registers = &cpu.registers;
        assert_eq!(registers.a(), 0x3C);
        assert_eq!(registers.get_flag_z(), 0);
        assert_eq!(registers.get_flag_h(), 0);
        assert_eq!(registers.get_flag_n(), 1);
        assert_eq!(registers.flag_cy(), 1);
        assert_eq!(registers.pc(), 8);
    }

    #[test]
    fn inc_r() {
        let rom = ROM::new(vec![
            0b00_111_110,
            0xFF,           // LD A, 0xFF
            0b00_111_100    // INC A
        ]);
        let (mut cpu, _) = create_cpu(rom);
        let cy = cpu.registers.flag_cy();
        run_steps(2, &mut cpu);
        let registers = &cpu.registers;
        assert_eq!(registers.a(), 0x00);
        assert_eq!(registers.get_flag_z(), 1);
        assert_eq!(registers.get_flag_h(), 1);
        assert_eq!(registers.get_flag_n(), 0);
        assert_eq!(registers.flag_cy(), cy);
        assert_eq!(registers.pc(), 3);
    }

    #[test]
    fn inc_mem_hl() {
        let rom = ROM::new(vec![
            0b00_100_001,
            0x00,
            0x80,          // LD HL, 0x8000
            0b00_110_110,
            0x50,          // LD (HL), 0x50
            0b00_110_100   // INC (HL)
        ]);
        let (mut cpu, memory) = create_cpu(rom);
        let cy = cpu.registers.flag_cy();
        run_steps(3, &mut cpu);
        let registers = &cpu.registers;
        assert_eq!(memory.read().unwrap().read(0x8000), 0x51);
        assert_eq!(registers.get_flag_z(), 0);
        assert_eq!(registers.get_flag_h(), 0);
        assert_eq!(registers.get_flag_n(), 0);
        assert_eq!(registers.flag_cy(), cy);
        assert_eq!(registers.pc(), 6);
    }

    #[test]
    fn dec_r() {
        let rom = ROM::new(vec![
            0b00_101_110,
            0x01,           // LD L, 0x01
            0b00_101_101    // DEC L
        ]);
        let (mut cpu, _) = create_cpu(rom);
        let cy = cpu.registers.flag_cy();
        run_steps(2, &mut cpu);
        let registers = &cpu.registers;
        assert_eq!(registers.l(), 0x00);
        assert_eq!(registers.get_flag_z(), 1);
        assert_eq!(registers.get_flag_h(), 0);
        assert_eq!(registers.get_flag_n(), 1);
        assert_eq!(registers.flag_cy(), cy);
        assert_eq!(registers.pc(), 3);
    }

    #[test]
    fn dec_mem_hl() {
        let rom = ROM::new(vec![
            0b00_100_001,
            0x00,
            0x80,          // LD HL, 0x8000
            0b00_110_110,
            0x00,          // LD (HL), 0x00
            0b00_110_101   // DEC (HL)
        ]);
        let (mut cpu, memory) = create_cpu(rom);
        let cy = cpu.registers.flag_cy();
        run_steps(3, &mut cpu);
        let registers = &cpu.registers;
        assert_eq!(memory.read().unwrap().read(0x8000), 0xFF);
        assert_eq!(registers.get_flag_z(), 0);
        assert_eq!(registers.get_flag_h(), 1);
        assert_eq!(registers.get_flag_n(), 1);
        assert_eq!(registers.flag_cy(), cy);
        assert_eq!(registers.pc(), 6);
    }

    #[test]
    fn add_hl_bc() {
        let rom = ROM::new(vec![
            0b00_100_001,
            0x23,
            0x8A,           // LD HL, 0x8A23
            0b00_000_001,
            0x05,
            0x06,           // LD BC, 0x0605
            0b00_001_001    // ADD HL, BC

        ]);
        let (mut cpu, _) = create_cpu(rom);
        run_steps(3, &mut cpu);
        let registers = &cpu.registers;
        assert_eq!(registers.hl(), 0x9028);
        assert_eq!(registers.get_flag_h(), 1);
        assert_eq!(registers.get_flag_n(), 0);
        assert_eq!(registers.flag_cy(), 0);
        assert_eq!(registers.pc(), 7);
    }

    #[test]
    fn add_hl_hl() {
        let rom = ROM::new(vec![
            0b00_100_001,
            0x23,
            0x8A,           // LD HL, 0x8A23
            0b00_101_001    // ADD HL, BC

        ]);
        let (mut cpu, _) = create_cpu(rom);
        let z = cpu.registers.get_flag_z();
        run_steps(2, &mut cpu);
        let registers = &cpu.registers;
        assert_eq!(registers.hl(), 0x1446);
        assert_eq!(registers.get_flag_z(), z);
        assert_eq!(registers.get_flag_h(), 1);
        assert_eq!(registers.get_flag_n(), 0);
        assert_eq!(registers.flag_cy(), 1);
        assert_eq!(registers.pc(), 4);
    }

    #[test]
    fn add_sp_2() {
        let rom = ROM::new(vec![
            0b00_110_001,
            0xF8,
            0xFF,           // LD SP, 0xFFF8
            0b11_101_000,
            2               // ADD HL, BC

        ]);
        let (mut cpu, _) = create_cpu(rom);
        run_steps(2, &mut cpu);
        let registers = &cpu.registers;
        assert_eq!(registers.sp(), 0xFFFA);
        assert_eq!(registers.get_flag_z(), 0);
        assert_eq!(registers.get_flag_h(), 0);
        assert_eq!(registers.get_flag_n(), 0);
        assert_eq!(registers.flag_cy(), 0);
        assert_eq!(registers.pc(), 5);
    }

    #[test]
    fn inc_de() {
        let rom = ROM::new(vec![
            0b00_010_001,
            0x5F,
            0x23,           // LD HL, 0x235F
            0b00_010_011    // INC DE

        ]);
        let (mut cpu, _) = create_cpu(rom);
        let z = cpu.registers.get_flag_z();
        let h = cpu.registers.get_flag_h();
        let n = cpu.registers.get_flag_n();
        let cy = cpu.registers.flag_cy();
        run_steps(2, &mut cpu);
        let registers = &cpu.registers;
        assert_eq!(registers.de(), 0x2360);
        assert_eq!(registers.get_flag_z(), z);
        assert_eq!(registers.get_flag_h(), h);
        assert_eq!(registers.get_flag_n(), n);
        assert_eq!(registers.flag_cy(), cy);
        assert_eq!(registers.pc(), 4);
    }

    #[test]
    fn dec_de() {
        let rom = ROM::new(vec![
            0b00_010_001,
            0x5F,
            0x23,           // LD HL, 0x235F
            0b00_011_011    // DEC DE

        ]);
        let (mut cpu, _) = create_cpu(rom);
        let z = cpu.registers.get_flag_z();
        let h = cpu.registers.get_flag_h();
        let n = cpu.registers.get_flag_n();
        let cy = cpu.registers.flag_cy();
        run_steps(2, &mut cpu);
        let registers = &cpu.registers;
        assert_eq!(registers.de(), 0x235E);
        assert_eq!(registers.get_flag_z(), z);
        assert_eq!(registers.get_flag_h(), h);
        assert_eq!(registers.get_flag_n(), n);
        assert_eq!(registers.flag_cy(), cy);
        assert_eq!(registers.pc(), 4);
    }

    #[test]
    fn rlca() {
        let rom = ROM::new(vec![
            0b00_111_110,
            0x85,           // LD A, 0x85
            0b00_000_111    // RLCA

        ]);
        let (mut cpu, _) = create_cpu(rom);
        run_steps(2, &mut cpu);
        let registers = &cpu.registers;
        assert_eq!(registers.a(), 0x0B);
        assert_eq!(registers.get_flag_z(), 0);
        assert_eq!(registers.get_flag_h(), 0);
        assert_eq!(registers.get_flag_n(), 0);
        assert_eq!(registers.flag_cy(), 1);
        assert_eq!(registers.pc(), 3);
    }

    #[test]
    fn rla() {
        let rom = ROM::new(vec![
            0b00_111_110,
            0xFF,           // LD A, 0xFF
            0b11_000_110,
            1,              // ADD A, 1
            0b00_111_110,
            0x95,           // LD A, 0x95
            0b00_010_111    // RLA

        ]);
        let (mut cpu, _) = create_cpu(rom);
        run_steps(4, &mut cpu);
        let registers = &cpu.registers;
        assert_eq!(registers.a(), 0x2B);
        assert_eq!(registers.get_flag_z(), 0);
        assert_eq!(registers.get_flag_h(), 0);
        assert_eq!(registers.get_flag_n(), 0);
        assert_eq!(registers.flag_cy(), 1);
        assert_eq!(registers.pc(), 7);
    }

    #[test]
    fn rrca() {
        let rom = ROM::new(vec![
            0b00_111_110,
            0x3b,           // LD A, 0x3b
            0b00_001_111    // RRCA

        ]);
        let (mut cpu, _) = create_cpu(rom);
        run_steps(2, &mut cpu);
        let registers = &cpu.registers;
        assert_eq!(registers.a(), 0x9D);
        assert_eq!(registers.get_flag_z(), 0);
        assert_eq!(registers.get_flag_h(), 0);
        assert_eq!(registers.get_flag_n(), 0);
        assert_eq!(registers.flag_cy(), 1);
        assert_eq!(registers.pc(), 3);
    }

    #[test]
    fn rra() {
        let rom = ROM::new(vec![
            0b00_111_110,
            0x81,           // LD A, 0x81
            0b00_011_111    // RRA

        ]);
        let (mut cpu, _) = create_cpu(rom);
        run_steps(2, &mut cpu);
        let registers = &cpu.registers;
        assert_eq!(registers.a(), 0x40);
        assert_eq!(registers.get_flag_z(), 0);
        assert_eq!(registers.get_flag_h(), 0);
        assert_eq!(registers.get_flag_n(), 0);
        assert_eq!(registers.flag_cy(), 1);
        assert_eq!(registers.pc(), 3);
    }

    #[test]
    fn rlc_b() {
        let rom = ROM::new(vec![
            0b00_000_110,
            0x85,           // LD B, 0x85
            0b11_001_011,
            0b00_000_000    // RLC B

        ]);
        let (mut cpu, _) = create_cpu(rom);
        run_steps(2, &mut cpu);
        let registers = &cpu.registers;
        assert_eq!(registers.b(), 0x0B);
        assert_eq!(registers.get_flag_z(), 0);
        assert_eq!(registers.get_flag_h(), 0);
        assert_eq!(registers.get_flag_n(), 0);
        assert_eq!(registers.flag_cy(), 1);
        assert_eq!(registers.pc(), 4);
    }

    #[test]
    fn rlc_mem_hl() {
        let rom = ROM::new(vec![
            0b00_100_001,
            0x00,
            0x80,          // LD HL, 0x8000
            0b00_110_110,
            0,             // LD (HL), 0
            0b11_001_011,
            0b00_000_110   // RLC (HL)

        ]);
        let (mut cpu, memory) = create_cpu(rom);
        run_steps(3, &mut cpu);
        let registers = &cpu.registers;
        assert_eq!(memory.read().unwrap().read(0x8000), 0);
        assert_eq!(registers.get_flag_z(), 1);
        assert_eq!(registers.get_flag_h(), 0);
        assert_eq!(registers.get_flag_n(), 0);
        assert_eq!(registers.flag_cy(), 0);
        assert_eq!(registers.pc(), 7);
    }

    #[test]
    fn rl_l() {
        let rom = ROM::new(vec![
            0b00_101_110,
            0x80,           // LD L, 0x80
            0b11_001_011,
            0b00_010_101    // RLC L

        ]);
        let (mut cpu, _) = create_cpu(rom);
        run_steps(2, &mut cpu);
        let registers = &cpu.registers;
        assert_eq!(registers.l(), 0x00);
        assert_eq!(registers.get_flag_z(), 1);
        assert_eq!(registers.get_flag_h(), 0);
        assert_eq!(registers.get_flag_n(), 0);
        assert_eq!(registers.flag_cy(), 1);
        assert_eq!(registers.pc(), 4);
    }

    #[test]
    fn rl_mem_hl() {
        let rom = ROM::new(vec![
            0b00_100_001,
            0x00,
            0x80,          // LD HL, 0x8000
            0b00_110_110,
            0x11,          // LD (HL), 0x11
            0b11_001_011,
            0b00_010_110   // RLC (HL)

        ]);
        let (mut cpu, memory) = create_cpu(rom);
        run_steps(3, &mut cpu);
        let registers = &cpu.registers;
        assert_eq!(memory.read().unwrap().read(0x8000), 0x22);
        assert_eq!(registers.get_flag_z(), 0);
        assert_eq!(registers.get_flag_h(), 0);
        assert_eq!(registers.get_flag_n(), 0);
        assert_eq!(registers.flag_cy(), 0);
        assert_eq!(registers.pc(), 7);
    }

    #[test]
    fn rrc_c() {
        let rom = ROM::new(vec![
            0b00_001_110,
            0x1,           // LD C, 0x1
            0b11_001_011,
            0b00_001_001    // RRC C

        ]);
        let (mut cpu, _) = create_cpu(rom);
        run_steps(2, &mut cpu);
        let registers = &cpu.registers;
        assert_eq!(registers.c(), 0x80);
        assert_eq!(registers.get_flag_z(), 0);
        assert_eq!(registers.get_flag_h(), 0);
        assert_eq!(registers.get_flag_n(), 0);
        assert_eq!(registers.flag_cy(), 1);
        assert_eq!(registers.pc(), 4);
    }

    #[test]
    fn rrc_mem_hl() {
        let rom = ROM::new(vec![
            0b00_100_001,
            0x00,
            0x80,          // LD HL, 0x8000
            0b00_110_110,
            0x0,          // LD (HL), 0x0
            0b11_001_011,
            0b00_001_110   // RRC (HL)

        ]);
        let (mut cpu, memory) = create_cpu(rom);
        run_steps(3, &mut cpu);
        let registers = &cpu.registers;
        assert_eq!(memory.read().unwrap().read(0x8000), 0);
        assert_eq!(registers.get_flag_z(), 1);
        assert_eq!(registers.get_flag_h(), 0);
        assert_eq!(registers.get_flag_n(), 0);
        assert_eq!(registers.flag_cy(), 0);
        assert_eq!(registers.pc(), 7);
    }

    #[test]
    fn rr_a() {
        let rom = ROM::new(vec![
            0b00_111_110,
            0x1,           // LD A, 0x1
            0b11_001_011,
            0b00_011_111    // RRC A

        ]);
        let (mut cpu, _) = create_cpu(rom);
        run_steps(2, &mut cpu);
        let registers = &cpu.registers;
        assert_eq!(registers.a(), 0x00);
        assert_eq!(registers.get_flag_z(), 1);
        assert_eq!(registers.get_flag_h(), 0);
        assert_eq!(registers.get_flag_n(), 0);
        assert_eq!(registers.flag_cy(), 1);
        assert_eq!(registers.pc(), 4);
    }

    #[test]
    fn rr_mem_hl() {
        let rom = ROM::new(vec![
            0b00_100_001,
            0x00,
            0x80,          // LD HL, 0x8000
            0b00_110_110,
            0x8a,          // LD (HL), 0x8A
            0b11_001_011,
            0b00_011_110   // RR (HL)

        ]);
        let (mut cpu, memory) = create_cpu(rom);
        run_steps(3, &mut cpu);
        let registers = &cpu.registers;
        assert_eq!(memory.read().unwrap().read(0x8000), 0x45);
        assert_eq!(registers.get_flag_z(), 0);
        assert_eq!(registers.get_flag_h(), 0);
        assert_eq!(registers.get_flag_n(), 0);
        assert_eq!(registers.flag_cy(), 0);
        assert_eq!(registers.pc(), 7);
    }

    #[test]
    fn sla_d() {
        let rom = ROM::new(vec![
            0b00_010_110,
            0x80,           // LD D, 0x80
            0b11_001_011,
            0b00_100_010    // SLA D

        ]);
        let (mut cpu, _) = create_cpu(rom);
        run_steps(2, &mut cpu);
        let registers = &cpu.registers;
        assert_eq!(registers.d(), 0x00);
        assert_eq!(registers.get_flag_z(), 1);
        assert_eq!(registers.get_flag_h(), 0);
        assert_eq!(registers.get_flag_n(), 0);
        assert_eq!(registers.flag_cy(), 1);
        assert_eq!(registers.pc(), 4);
    }

    #[test]
    fn sla_mem_hl() {
        let rom = ROM::new(vec![
            0b00_100_001,
            0x00,
            0x80,          // LD HL, 0x8000
            0b00_110_110,
            0xFF,          // LD (HL), 0xFF
            0b11_001_011,
            0b00_100_110   // SLA (HL)

        ]);
        let (mut cpu, memory) = create_cpu(rom);
        run_steps(3, &mut cpu);
        let registers = &cpu.registers;
        assert_eq!(memory.read().unwrap().read(0x8000), 0xFE);
        assert_eq!(registers.get_flag_z(), 0);
        assert_eq!(registers.get_flag_h(), 0);
        assert_eq!(registers.get_flag_n(), 0);
        assert_eq!(registers.flag_cy(), 1);
        assert_eq!(registers.pc(), 7);
    }

    #[test]
fn sra_a() {
    let rom = ROM::new(vec![
        0b00_111_110,
        0x8A,           // LD A, 0x8A
        0b11_001_011,
        0b00_101_111    // SRA A

    ]);
    let (mut cpu, _) = create_cpu(rom);
        run_steps(2, &mut cpu);
    let registers = &cpu.registers;
    assert_eq!(registers.a(), 0xC5);
    assert_eq!(registers.get_flag_z(), 0);
    assert_eq!(registers.get_flag_h(), 0);
    assert_eq!(registers.get_flag_n(), 0);
    assert_eq!(registers.flag_cy(), 0);
    assert_eq!(registers.pc(), 4);
}

    #[test]
    fn sra_mem_hl() {
        let rom = ROM::new(vec![
            0b00_100_001,
            0x00,
            0x80,          // LD HL, 0x8000
            0b00_110_110,
            0x01,          // LD (HL), 0x01
            0b11_001_011,
            0b00_101_110   // SRA (HL)

        ]);
        let (mut cpu, memory) = create_cpu(rom);
        run_steps(3, &mut cpu);
        let registers = &cpu.registers;
        assert_eq!(memory.read().unwrap().read(0x8000), 0x00);
        assert_eq!(registers.get_flag_z(), 1);
        assert_eq!(registers.get_flag_h(), 0);
        assert_eq!(registers.get_flag_n(), 0);
        assert_eq!(registers.flag_cy(), 1);
        assert_eq!(registers.pc(), 7);
    }

    #[test]
    fn srl_a() {
        let rom = ROM::new(vec![
            0b00_111_110,
            0x01,           // LD A, 0x01
            0b11_001_011,
            0b00_111_111    // SRL A

        ]);
        let (mut cpu, _) = create_cpu(rom);
        run_steps(2, &mut cpu);
        let registers = &cpu.registers;
        assert_eq!(registers.a(), 0x00);
        assert_eq!(registers.get_flag_z(), 1);
        assert_eq!(registers.get_flag_h(), 0);
        assert_eq!(registers.get_flag_n(), 0);
        assert_eq!(registers.flag_cy(), 1);
        assert_eq!(registers.pc(), 4);
    }

    #[test]
    fn srl_mem_hl() {
        let rom = ROM::new(vec![
            0b00_100_001,
            0x00,
            0x80,          // LD HL, 0x8000
            0b00_110_110,
            0xFF,          // LD (HL), 0xFF
            0b11_001_011,
            0b00_111_110   // SRL (HL)

        ]);
        let (mut cpu, memory) = create_cpu(rom);
        run_steps(3, &mut cpu);
        let registers = &cpu.registers;
        assert_eq!(memory.read().unwrap().read(0x8000), 0x7F);
        assert_eq!(registers.get_flag_z(), 0);
        assert_eq!(registers.get_flag_h(), 0);
        assert_eq!(registers.get_flag_n(), 0);
        assert_eq!(registers.flag_cy(), 1);
        assert_eq!(registers.pc(), 7);
    }

    #[test]
    fn swap_a() {
        let rom = ROM::new(vec![
            0b00_111_110,
            0x00,           // LD A, 0x00
            0b11_001_011,
            0b00_110_111    // SWAP A

        ]);
        let (mut cpu, _) = create_cpu(rom);
        run_steps(2, &mut cpu);
        let registers = &cpu.registers;
        assert_eq!(registers.a(), 0x00);
        assert_eq!(registers.get_flag_z(), 1);
        assert_eq!(registers.get_flag_h(), 0);
        assert_eq!(registers.get_flag_n(), 0);
        assert_eq!(registers.flag_cy(), 0);
        assert_eq!(registers.pc(), 4);
    }

    #[test]
    fn swap_mem_hl() {
        let rom = ROM::new(vec![
            0b00_100_001,
            0x00,
            0x80,          // LD HL, 0x8000
            0b00_110_110,
            0xF0,          // LD (HL), 0xF0
            0b11_001_011,
            0b00_110_110   // SWAP (HL)

        ]);
        let (mut cpu, memory) = create_cpu(rom);
        run_steps(3, &mut cpu);
        let registers = &cpu.registers;
        assert_eq!(memory.read().unwrap().read(0x8000), 0x0F);
        assert_eq!(registers.get_flag_z(), 0);
        assert_eq!(registers.get_flag_h(), 0);
        assert_eq!(registers.get_flag_n(), 0);
        assert_eq!(registers.flag_cy(), 0);
        assert_eq!(registers.pc(), 7);
    }

    #[test]
    fn bit_7_a() {
        let rom = ROM::new(vec![
            0b00_111_110,
            0x80,           // LD A, 0x80
            0b11_001_011,
            0b01_111_111    // BIT 7, A

        ]);
        let (mut cpu, _) = create_cpu(rom);
        run_steps(2, &mut cpu);
        let registers = &cpu.registers;
        assert_eq!(registers.get_flag_z(), 0);
        assert_eq!(registers.get_flag_h(), 1);
        assert_eq!(registers.get_flag_n(), 0);
        assert_eq!(registers.pc(), 4);
    }

    #[test]
    fn bit_4_l() {
        let rom = ROM::new(vec![
            0b00_101_110,
            0xEF,           // LD L, 0xEF
            0b11_001_011,
            0b01_100_101    // BIT 4, L

        ]);
        let (mut cpu, _) = create_cpu(rom);
        run_steps(2, &mut cpu);
        let registers = &cpu.registers;
        assert_eq!(registers.get_flag_z(), 1);
        assert_eq!(registers.get_flag_h(), 1);
        assert_eq!(registers.get_flag_n(), 0);
        assert_eq!(registers.pc(), 4);
    }

    #[test]
    fn bit_0_mem_hl() {
        let rom = ROM::new(vec![
            0b00_100_001,
            0x00,
            0x80,          // LD HL, 0x8000
            0b00_110_110,
            0xFE,          // LD (HL), 0xFE
            0b11_001_011,
            0b01_000_110   // Bit 0, (HL)

        ]);
        let (mut cpu, _) = create_cpu(rom);
        run_steps(3, &mut cpu);
        let registers = &cpu.registers;
        assert_eq!(registers.get_flag_z(), 1);
        assert_eq!(registers.get_flag_h(), 1);
        assert_eq!(registers.get_flag_n(), 0);
        assert_eq!(registers.pc(), 7);
    }

    #[test]
    fn bit_1_mem_hl() {
        let rom = ROM::new(vec![
            0b00_100_001,
            0x00,
            0x80,          // LD HL, 0x8000
            0b00_110_110,
            0xFE,          // LD (HL), 0xFE
            0b11_001_011,
            0b01_001_110   // Bit 0, (HL)

        ]);
        let (mut cpu, _) = create_cpu(rom);
        run_steps(3, &mut cpu);
        let registers = &cpu.registers;
        assert_eq!(registers.get_flag_z(), 0);
        assert_eq!(registers.get_flag_h(), 1);
        assert_eq!(registers.get_flag_n(), 0);
        assert_eq!(registers.pc(), 7);
    }

    #[test]
    fn set_3_a() {
        let rom = ROM::new(vec![
            0b00_111_110,
            0x80,           // LD A, 0x80
            0b11_001_011,
            0b11_011_111    // SET 3, A

        ]);
        let (mut cpu, _) = create_cpu(rom);
        run_steps(2, &mut cpu);
        let registers = &cpu.registers;
        assert_eq!(registers.a(), 0x88);
        assert_eq!(registers.pc(), 4);
    }

    #[test]
    fn set_4_l() {
        let rom = ROM::new(vec![
            0b00_101_110,
            0x3B,           // LD L, 0x3B
            0b11_001_011,
            0b11_111_101    // SET 7, L

        ]);
        let (mut cpu, _) = create_cpu(rom);
        run_steps(2, &mut cpu);
        let registers = &cpu.registers;
        assert_eq!(registers.l(), 0xBB);
        assert_eq!(registers.pc(), 4);
    }

    #[test]
    fn set_0_mem_hl() {
        let rom = ROM::new(vec![
            0b00_100_001,
            0x00,
            0x80,          // LD HL, 0x8000
            0b00_110_110,
            0x00,          // LD (HL), 0x00
            0b11_001_011,
            0b11_011_110   // SET 3, (HL)

        ]);
        let (mut cpu, memory) = create_cpu(rom);
        run_steps(3, &mut cpu);
        let registers = &cpu.registers;
        assert_eq!(memory.read().unwrap().read(0x8000), 0x08);
        assert_eq!(registers.pc(), 7);
    }

    #[test]
    fn res_7_a() {
        let rom = ROM::new(vec![
            0b00_111_110,
            0x80,           // LD A, 0x80
            0b11_001_011,
            0b10_111_111    // RES 7, A

        ]);
        let (mut cpu, _) = create_cpu(rom);
        run_steps(2, &mut cpu);
        let registers = &cpu.registers;
        assert_eq!(registers.a(), 0x0);
        assert_eq!(registers.pc(), 4);
    }

    #[test]
    fn set_1_l() {
        let rom = ROM::new(vec![
            0b00_101_110,
            0x3B,           // LD L, 0x39
            0b11_001_011,
            0b10_001_101    // RES 1, L

        ]);
        let (mut cpu, _) = create_cpu(rom);
        run_steps(2, &mut cpu);
        let registers = &cpu.registers;
        assert_eq!(registers.l(), 0x39);
        assert_eq!(registers.pc(), 4);
    }

    #[test]
    fn res_3_mem_hl() {
        let rom = ROM::new(vec![
            0b00_100_001,
            0x00,
            0x80,          // LD HL, 0x8000
            0b00_110_110,
            0xFF,          // LD (HL), 0xFF
            0b11_001_011,
            0b10_011_110   // RES 3, (HL)
        ]);
        let (mut cpu, memory) = create_cpu(rom);
        run_steps(3, &mut cpu);
        let registers = &cpu.registers;
        assert_eq!(memory.read().unwrap().read(0x8000), 0xF7);
        assert_eq!(registers.pc(), 7);
    }

    #[test]
    fn jp_0x8000() {
        let rom = ROM::new(vec![
            0b11_000_011,
            0x00,
            0x80,          // JP 0x8000

        ]);
        let (mut cpu, _) = create_cpu(rom);
        run_steps(1, &mut cpu);
        let registers = &cpu.registers;
        assert_eq!(registers.pc(), 0x8000);
    }

    #[test]
    fn jp_nz_0x8000() {
        let rom = ROM::new(vec![
            0b00_111_110,
            0x00,           // LD A, 0x00
            0b11_001_011,
            0b00_110_111,    // SWAP A
            0b11_000_010,
            0x00,
            0x80,          // JP NZ 0x8000

        ]);
        let (mut cpu, _) = create_cpu(rom);
        run_steps(3, &mut cpu);
        let registers = &cpu.registers;
        assert_eq!(registers.pc(), 7);
    }

    #[test]
    fn jp_z_0x8000() {
        let rom = ROM::new(vec![
            0b00_111_110,
            0x00,           // LD A, 0x00
            0b11_001_011,
            0b00_110_111,    // SWAP A
            0b11_001_010,
            0x00,
            0x80,          // JP NZ 0x8000

        ]);
        let (mut cpu, _) = create_cpu(rom);
        run_steps(3, &mut cpu);
        let registers = &cpu.registers;
        assert_eq!(registers.pc(), 0x8000);
    }

    #[test]
    fn jp_c_0x8000() {
        let rom = ROM::new(vec![
            0b00_111_110,
            0x00,           // LD A, 0x00
            0b11_001_011,
            0b00_110_111,    // SWAP A
            0b11_011_010,
            0x00,
            0x80,          // JP NZ 0x8000

        ]);
        let (mut cpu, _) = create_cpu(rom);
        run_steps(3, &mut cpu);
        let registers = &cpu.registers;
        assert_eq!(registers.pc(), 7);
    }

    #[test]
    fn jp_nc_0x8000() {
        let rom = ROM::new(vec![
            0b00_111_110,
            0x00,           // LD A, 0x00
            0b11_001_011,
            0b00_110_111,    // SWAP A
            0b11_010_010,
            0x00,
            0x80,          // JP NZ 0x8000

        ]);
        let (mut cpu, _) = create_cpu(rom);
        run_steps(3, &mut cpu);
        let registers = &cpu.registers;
        assert_eq!(registers.pc(), 0x8000);
    }

    #[test]
    fn jr_neg_5() {
        let rom = ROM::new(vec![
            0x00,           // NOP
            0x00,           // NOP
            0x00,           // NOP
            0b00_011_000,
            -5i8 as u8,              // JP -5

        ]);
        let (mut cpu, _) = create_cpu(rom);
        run_steps(4, &mut cpu);
        let registers = &cpu.registers;
        assert_eq!(registers.pc(), 0);
    }

    #[test]
    fn jr_pos_5() {
        let rom = ROM::new(vec![
            0b00_011_000,
            5,              // JP 5

        ]);
        let (mut cpu, _) = create_cpu(rom);
        run_steps(1, &mut cpu);
        let registers = &cpu.registers;
        assert_eq!(registers.pc(), 7);
    }

    #[test]
    fn jr_z_neg_5() {
        let rom = ROM::new(vec![
            0b00_111_110,
            0x00,           // LD A, 0x00
            0b11_001_011,
            0b00_110_111,   // SWAP A
            0b00_101_000,
            -5i8 as u8,     // JR Z, -5

        ]);
        let (mut cpu, _) = create_cpu(rom);
        run_steps(3, &mut cpu);
        let registers = &cpu.registers;
        assert_eq!(registers.pc(), 1);
    }

    #[test]
    fn jr_nc_pos_5() {
        let rom = ROM::new(vec![
            0b00_111_110,
            0x00,           // LD A, 0x00
            0b11_001_011,
            0b00_110_111,   // SWAP A
            0b00_110_000,
            5,              // JR NC, 5

        ]);
        let (mut cpu, _) = create_cpu(rom);
        run_steps(3, &mut cpu);
        let registers = &cpu.registers;
        assert_eq!(registers.pc(), 11);
    }

    #[test]
    fn jp_mem_hl() {
        let rom = ROM::new(vec![
            0b00_100_001,
            0x00,
            0x80,          // LD HL, 0x8000
            0b11_101_001   // JP (HL)
        ]);
        let (mut cpu, _) = create_cpu(rom);
        run_steps(2, &mut cpu);
        let registers = &cpu.registers;
        assert_eq!(registers.pc(), 0x8000);
    }

    #[test]
    fn call_0x1234() {
        let rom = ROM::new(vec![
            0b11_000_011,
            0x00,
            0x80,          // JP 0x8000
        ]);
        let (mut cpu, memory) = create_cpu(rom);
        {
            let mut memory = memory.write().unwrap();
            memory.write(0x8000, 0b11_001_101);
            memory.write(0x8001, 0x34);
            memory.write(0x8002, 0x12);         // CALL 0x1234
        }
        run_steps(2, &mut cpu);
        let registers = &cpu.registers;
        assert_eq!(registers.pc(), 0x1234);
        assert_eq!(registers.sp(), 0xFFFC);
        assert_eq!(memory.read().unwrap().read(0xFFFD), 0x80);
        assert_eq!(memory.read().unwrap().read(0xFFFC), 0x03);
    }

    #[test]
    fn call_z_0x1234() {
        let rom = ROM::new(vec![
            0b00_111_110,
            0x00,           // LD A, 0x00
            0b11_001_011,
            0b00_110_111,   // SWAP A
            0b11_000_011,
            0x00,
            0x80,          // JP 0x7FFC

        ]);
        let (mut cpu, memory) = create_cpu(rom);
        {
            let mut memory = memory.write().unwrap();
            memory.write(0x8000, 0b11_000_100);
            memory.write(0x8001, 0xFF);
            memory.write(0x8002, 0xFF);         // CALL NZ, 0xFFFF
            memory.write(0x8003, 0b11_001_100);
            memory.write(0x8004, 0x34);
            memory.write(0x8005, 0x12);         // CALL Z, 0x1234
        }
        run_steps(5, &mut cpu);
        let registers = &cpu.registers;
        assert_eq!(registers.pc(), 0x1234);
        assert_eq!(registers.sp(), 0xFFFC);
        assert_eq!(memory.read().unwrap().read(0xFFFD), 0x80);
        assert_eq!(memory.read().unwrap().read(0xFFFC), 0x06);
    }

    #[test]
    fn call_0x8000_ret() {
        let rom = ROM::new(vec![
            0b11_000_011,
            0x00,
            0x80,          // JP 0x8000
        ]);
        let (mut cpu, memory) = create_cpu(rom);
        {
            let mut memory = memory.write().unwrap();
            memory.write(0x8000, 0b11_001_101);
            memory.write(0x8001, 0x00);
            memory.write(0x8002, 0x90);         // CALL 0x1234
            memory.write(0x9000, 0b00_000_000); // NOP
            memory.write(0x9001, 0b11_001_001); // RET
        }
        run_steps(4, &mut cpu);
        let registers = &cpu.registers;
        assert_eq!(registers.pc(), 0x8003);
        assert_eq!(registers.sp(), 0xFFFE);
    }

    #[test]
    fn call_0x8000_reti() {
        let rom = ROM::new(vec![
            0b11_110_011,  // DI
            0b11_000_011,
            0x00,
            0x80,          // JP 0x8000
        ]);
        let (mut cpu, memory) = create_cpu(rom);
        {
            let mut memory = memory.write().unwrap();
            memory.write(0x8000, 0b11_001_101);
            memory.write(0x8001, 0x00);
            memory.write(0x8002, 0x90);         // CALL 0x1234
            memory.write(0x9000, 0b00_000_000); // NOP
            memory.write(0x9001, 0b11_011_001); // RETI
        }
        run_steps(5, &mut cpu);
        let registers = &cpu.registers;
        assert_eq!(registers.pc(), 0x8003);
        assert_eq!(registers.sp(), 0xFFFE);
        assert_eq!(cpu.interrupt.read().unwrap().master_enable, true);
    }

    #[test]
    fn call_0x8000_nz_ret() {
        let rom = ROM::new(vec![
            0b00_111_110,
            0x00,           // LD A, 0x00
            0b11_001_011,
            0b00_110_111,   // SWAP A
            0b11_000_011,
            0x00,
            0x80,          // JP 0x8000
        ]);
        let (mut cpu, memory) = create_cpu(rom);
        {
            let mut memory = memory.write().unwrap();
            memory.write(0x8000, 0b11_001_101);
            memory.write(0x8001, 0x00);
            memory.write(0x8002, 0x90);         // CALL 0x1234
            memory.write(0x9000, 0b00_000_000); // NOP
            memory.write(0x9001, 0b11_000_000); // RET NZ
        }
        run_steps(6, &mut cpu);
        let registers = &cpu.registers;
        assert_eq!(registers.pc(), 0x9002);
        assert_eq!(registers.sp(), 0xFFFC);
    }

    #[test]
    fn call_0x8000_z_ret() {
        let rom = ROM::new(vec![
            0b00_111_110,
            0x00,           // LD A, 0x00
            0b11_001_011,
            0b00_110_111,   // SWAP A
            0b11_000_011,
            0x00,
            0x80,          // JP 0x8000
        ]);
        let (mut cpu, memory) = create_cpu(rom);
        {
            let mut memory = memory.write().unwrap();
            memory.write(0x8000, 0b11_001_101);
            memory.write(0x8001, 0x00);
            memory.write(0x8002, 0x90);         // CALL 0x1234
            memory.write(0x9000, 0b00_000_000); // NOP
            memory.write(0x9001, 0b11_001_000); // RET Z
        }
        run_steps(6, &mut cpu);
        let registers = &cpu.registers;
        assert_eq!(registers.pc(), 0x8003);
        assert_eq!(registers.sp(), 0xFFFE);
    }

    #[test]
    fn rst_0() {
        let rom = ROM::new(vec![
            0b11_000_011,
            0x00,
            0x80,          // JP 0x8000
        ]);
        let (mut cpu, memory) = create_cpu(rom);
        {
            let mut memory = memory.write().unwrap();
            memory.write(0x8000, 0b11_000_111); // RST 0
        }
        run_steps(2, &mut cpu);
        let registers = &cpu.registers;
        assert_eq!(registers.pc(), 0x0000);
        assert_eq!(registers.sp(), 0xFFFC);
        assert_eq!(memory.read().unwrap().read(0xFFFD), 0x80);
        assert_eq!(memory.read().unwrap().read(0xFFFC), 0x01);
    }

    #[test]
    fn rst_1() {
        let rom = ROM::new(vec![
            0b11_000_011,
            0x00,
            0x80,          // JP 0x8000
        ]);
        let (mut cpu, memory) = create_cpu(rom);
        {
            let mut memory = memory.write().unwrap();
            memory.write(0x8000, 0b11_001_111); // RST 1
        }
        run_steps(2, &mut cpu);
        let registers = &cpu.registers;
        assert_eq!(registers.pc(), 0x0008);
        assert_eq!(registers.sp(), 0xFFFC);
        assert_eq!(memory.read().unwrap().read(0xFFFD), 0x80);
        assert_eq!(memory.read().unwrap().read(0xFFFC), 0x01);
    }

    #[test]
    fn rst_2() {
        let rom = ROM::new(vec![
            0b11_000_011,
            0x00,
            0x80,          // JP 0x8000
        ]);
        let (mut cpu, memory) = create_cpu(rom);
        {
            let mut memory = memory.write().unwrap();
            memory.write(0x8000, 0b11_010_111); // RST 2
        }
        run_steps(2, &mut cpu);
        let registers = &cpu.registers;
        assert_eq!(registers.pc(), 0x0010);
        assert_eq!(registers.sp(), 0xFFFC);
        assert_eq!(memory.read().unwrap().read(0xFFFD), 0x80);
        assert_eq!(memory.read().unwrap().read(0xFFFC), 0x01);
    }

    #[test]
    fn rst_3() {
        let rom = ROM::new(vec![
            0b11_000_011,
            0x00,
            0x80,          // JP 0x8000
        ]);
        let (mut cpu, memory) = create_cpu(rom);
        {
            let mut memory = memory.write().unwrap();
            memory.write(0x8000, 0b11_011_111); // RST 3
        }
        run_steps(2, &mut cpu);
        let registers = &cpu.registers;
        assert_eq!(registers.pc(), 0x0018);
        assert_eq!(registers.sp(), 0xFFFC);
        assert_eq!(memory.read().unwrap().read(0xFFFD), 0x80);
        assert_eq!(memory.read().unwrap().read(0xFFFC), 0x01);
    }

    #[test]
    fn rst_4() {
        let rom = ROM::new(vec![
            0b11_000_011,
            0x00,
            0x80,          // JP 0x8000
        ]);
        let (mut cpu, memory) = create_cpu(rom);
        {
            let mut memory = memory.write().unwrap();
            memory.write(0x8000, 0b11_100_111); // RST 4
        }
        run_steps(2, &mut cpu);
        let registers = &cpu.registers;
        assert_eq!(registers.pc(), 0x0020);
        assert_eq!(registers.sp(), 0xFFFC);
        assert_eq!(memory.read().unwrap().read(0xFFFD), 0x80);
        assert_eq!(memory.read().unwrap().read(0xFFFC), 0x01);
    }

    #[test]
    fn rst_5() {
        let rom = ROM::new(vec![
            0b11_000_011,
            0x00,
            0x80,          // JP 0x8000
        ]);
        let (mut cpu, memory) = create_cpu(rom);
        {
            let mut memory = memory.write().unwrap();
            memory.write(0x8000, 0b11_101_111); // RST 5
        }
        run_steps(2, &mut cpu);
        let registers = &cpu.registers;
        assert_eq!(registers.pc(), 0x0028);
        assert_eq!(registers.sp(), 0xFFFC);
        assert_eq!(memory.read().unwrap().read(0xFFFD), 0x80);
        assert_eq!(memory.read().unwrap().read(0xFFFC), 0x01);
    }

    #[test]
    fn rst_6() {
        let rom = ROM::new(vec![
            0b11_000_011,
            0x00,
            0x80,          // JP 0x8000
        ]);
        let (mut cpu, memory) = create_cpu(rom);
        {
            let mut memory = memory.write().unwrap();
            memory.write(0x8000, 0b11_110_111); // RST 6
        }
        run_steps(2, &mut cpu);
        let registers = &cpu.registers;
        assert_eq!(registers.pc(), 0x0030);
        assert_eq!(registers.sp(), 0xFFFC);
        assert_eq!(memory.read().unwrap().read(0xFFFD), 0x80);
        assert_eq!(memory.read().unwrap().read(0xFFFC), 0x01);
    }

    #[test]
    fn rst_7() {
        let rom = ROM::new(vec![
            0b11_000_011,
            0x00,
            0x80,          // JP 0x8000
        ]);
        let (mut cpu, memory) = create_cpu(rom);
        {
            let mut memory = memory.write().unwrap();
            memory.write(0x8000, 0b11_111_111); // RST 7
        }
        run_steps(2, &mut cpu);
        let registers = &cpu.registers;
        assert_eq!(registers.pc(), 0x0038);
        assert_eq!(registers.sp(), 0xFFFC);
        assert_eq!(memory.read().unwrap().read(0xFFFD), 0x80);
        assert_eq!(memory.read().unwrap().read(0xFFFC), 0x01);
    }

    #[test]
    fn cpl() {
        let rom = ROM::new(vec![
            0b00_111_110,
            0x35,           // LD A, 0x00
            0b00_101_111    // CPL
        ]);
        let (mut cpu, memory) = create_cpu(rom);
        run_steps(2, &mut cpu);
        let registers = &cpu.registers;
        assert_eq!(registers.a(), 0xCA);
        assert_eq!(registers.pc(), 3);
    }

    #[test]
    fn scf() {
        let rom = ROM::new(vec![
            0b00_111_111,   // CCF
            0b00_110_111,   // SCF
        ]);
        let (mut cpu, memory) = create_cpu(rom);
        run_steps(2, &mut cpu);
        let registers = &cpu.registers;
        assert_eq!(registers.flag_cy(), 1);
        assert_eq!(registers.pc(), 2);
    }

    #[test]
    fn ccf() {
        let rom = ROM::new(vec![
            0b00_110_111,   // SCF
            0b00_111_111,   // CCF
        ]);
        let (mut cpu, memory) = create_cpu(rom);
        run_steps(2, &mut cpu);
        let registers = &cpu.registers;
        assert_eq!(registers.flag_cy(), 0);
        assert_eq!(registers.pc(), 2);
    }

    #[test]
    fn nop() {
        let rom = ROM::new(vec![
            0b00_000_000,   // SCF
            0b00_000_000,   // SCF
            0b00_000_000,   // SCF
            0b00_000_000,   // SCF
            0b00_000_000,   // SCF
        ]);
        let (mut cpu, memory) = create_cpu(rom);
        run_steps(5, &mut cpu);
        let registers = &cpu.registers;
        assert_eq!(registers.pc(), 5);
    }

    #[test]
    fn di() {
        let rom = ROM::new(vec![
            0b11_111_011,  // EI
            0b11_110_011,  // DI
        ]);
        let (mut cpu, memory) = create_cpu(rom);
        run_steps(2, &mut cpu);
        let registers = &cpu.registers;
        assert_eq!(cpu.interrupt.read().unwrap().master_enable, false);
        assert_eq!(registers.pc(), 2);
    }

    #[test]
    fn ei() {
        let rom = ROM::new(vec![
            0b11_110_011,  // DI
            0b11_111_011,  // EI
        ]);
        let (mut cpu, memory) = create_cpu(rom);
        run_steps(2, &mut cpu);
        let registers = &cpu.registers;
        assert_eq!(cpu.interrupt.read().unwrap().master_enable, true);
        assert_eq!(registers.pc(), 2);
    }
}