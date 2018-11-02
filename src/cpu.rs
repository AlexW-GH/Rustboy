use std::sync::{Arc, RwLock};

use registers::Registers;
use registers::RegisterR;
use registers::RegisterDD;
use registers::RegisterQQ;
use registers::RegisterSS;
use registers::Condition;

use memory::Memory;
use interrupt_controller::InterruptController;
use util::bit_op;



pub struct CPU{
    registers: Registers,
    interrupt: Arc<RwLock<InterruptController>>,
    memory: Arc<RwLock<Memory>>
}

impl CPU{
    pub fn new(interrupt: Arc<RwLock<InterruptController>>, memory: Arc<RwLock<Memory>>, boot_sequence: bool) -> CPU{
        CPU {registers: Registers::new(boot_sequence), interrupt, memory}
    }

    pub fn step(&mut self){
        let pc = self.registers.pc();
        let opcode = self.read_memory(pc);
        match opcode {
            0x00 => self.nop(opcode, pc),
            0x01 => self.ld_dd_nn(opcode, pc),
            0x02 => self.ld_mem_bc_a(opcode, pc),
            0x05 => self.dec_r(opcode, pc),
            0x06 => self.ld_r_n(opcode, pc),
            0x07 => self.rlca(opcode, pc),
            0x08 => self.ld_mem_nn_sp(opcode, pc),
            0x09 => self.add_hl_ss(opcode, pc),
            0x0A => self.ld_a_mem_bc(opcode, pc),
            0x0D => self.dec_r(opcode, pc),
            0x0E => self.ld_r_n(opcode, pc),
            0x0F => self.rrca(opcode, pc),
            0x10 => self.ld_dd_nn(opcode, pc),
            0x11 => self.ld_dd_nn(opcode, pc),
            0x12 => self.ld_mem_de_a(opcode, pc),
            0x13 => self.inc_ss(opcode, pc),
            0x14 => self.inc_r(opcode, pc),
            0x16 => self.ld_r_n(opcode, pc),
            0x17 => self.rla(opcode, pc),
            0x18 => self.jr_e(opcode, pc),
            0x1A => self.ld_a_mem_de(opcode, pc),
            0x1B => self.dec_ss(opcode, pc),
            0x1C => self.inc_r(opcode, pc),
            0x1E => self.ld_r_n(opcode, pc),
            0x1F => self.rra(opcode, pc),
            0x0C => self.inc_r(opcode, pc),
            0x20 => self.jr_cc_e(opcode, pc),
            0x21 => self.ld_dd_nn(opcode, pc),
            0x22 => self.ld_mem_hli_a(opcode, pc),
            0x23 => self.inc_ss(opcode, pc),
            0x26 => self.ld_r_n(opcode, pc),
            0x29 => self.add_hl_ss(opcode, pc),
            0x2A => self.ld_a_mem_hli(opcode, pc),
            0x2E => self.ld_r_n(opcode, pc),
            0x2D => self.dec_r(opcode, pc),
            0x31 => self.ld_dd_nn(opcode, pc),
            0x32 => self.ld_mem_hld_a(opcode, pc),
            0x34 => self.inc_mem_hl(opcode, pc),
            0x35 => self.dec_mem_hl(opcode, pc),
            0x36 => self.ld_mem_hl_n(opcode, pc),
            0x3C => self.inc_r(opcode, pc),
            0x3E => self.ld_r_n(opcode, pc),
            0x3A => self.ld_a_mem_hld(opcode, pc),
            0x42 => self.ld_r_r(opcode, pc),
            0x45 => self.ld_r_r(opcode, pc),
            0x46 => self.ld_r_mem_hl(opcode, pc),
            0x47 => self.ld_r_r(opcode, pc),
            0x4F => self.ld_r_r(opcode, pc),
            0x60 => self.ld_mem_hl_r(opcode, pc),
            0x66 => self.ld_r_mem_hl(opcode, pc),
            0x67 => self.ld_mem_hl_r(opcode, pc),
            0x77 => self.ld_mem_hl_r(opcode, pc),
            0x78 => self.ld_r_r(opcode, pc),
            0x7C => self.ld_r_r(opcode, pc),
            0x7D => self.ld_r_r(opcode, pc),
            0x80 ... 0x85 => self.add_a_r(opcode, pc),
            0x86 => self.add_a_mem_hl(opcode, pc),
            0x87 => self.add_a_r(opcode, pc),
            0x88 ... 0x8D => self.adc_a_r(opcode, pc),
            0x8E => self.adc_a_mem_hl(opcode, pc),
            0x8F => self.adc_a_r(opcode, pc),
            0x90 ... 0x95 => self.sub_a_r(opcode, pc),
            0x96 => self.sub_a_mem_hl(opcode, pc),
            0x97 => self.sub_a_r(opcode, pc),
            0x98 ... 0x9D => self.sbc_a_r(opcode, pc),
            0x9E => self.sbc_a_mem_hl(opcode, pc),
            0x9F => self.sbc_a_r(opcode, pc),
            0xA0 ... 0xA5 => self.and_r(opcode, pc),
            0xA6 => self.and_mem_hl(opcode, pc),
            0xA7 => self.and_r(opcode, pc),
            0xA8 ... 0xAD => self.xor_r(opcode, pc),
            0xAE => self.xor_mem_hl(opcode, pc),
            0xAF => self.xor_r(opcode, pc),
            0xB0 ... 0xB5 => self.or_r(opcode, pc),
            0xB6 => self.or_mem_hl(opcode, pc),
            0xB7 => self.or_r(opcode, pc),
            0xB8 ... 0xBD => self.cp_r(opcode, pc),
            0xBE => self.cp_mem_hl(opcode, pc),
            0xBF => self.cp_r(opcode, pc),
            0xC1 => self.pop_qq(opcode, pc),
            0xC3 => self.jp_nn(opcode, pc),
            0xC5 => self.push_qq(opcode, pc),
            0xC6 => self.add_a_n(opcode, pc),
            0xC9 => self.ret(opcode, pc),
            0xCB => self.extended_operations(pc),
            0xCD => self.call_nn(opcode, pc),
            0xCE => self.adc_a_n(opcode, pc),
            0xD6 => self.sub_a_n(opcode, pc),
            0xDC => self.call_cc_nn(opcode, pc),
            0xDE => self.sbc_a_n(opcode, pc),
            0xE0 => self.ld_mem_n_a(opcode, pc),
            0xE2 => self.ld_mem_c_a(opcode, pc),
            0xE5 => self.push_qq(opcode, pc),
            0xE6 => self.and_n(opcode, pc),
            0xE8 => self.add_sp_e(opcode, pc),
            0xEA => self.ld_mem_nn_a(opcode, pc),
            0xEE => self.xor_n(opcode, pc),
            0xF0 => self.ld_a_mem_n(opcode, pc),
            0xF2 => self.ld_a_mem_c(opcode, pc),
            0xF3 => self.di(opcode, pc),
            0xF5 => self.push_qq(opcode, pc),
            0xF6 => self.or_n(opcode, pc),
            0xF8 => self.ldhl_sp_e(opcode, pc),
            0xF9 => self.ld_sp_hl(opcode, pc),
            0xFA => self.ld_a_mem_nn(opcode, pc),
            0xFB => self.ei(opcode, pc),
            0xFE => self.cp_n(opcode, pc),
            _ => {
                debug!("{:#06X}: {:#04X} | ({:#010b})", pc, opcode, opcode);
                unimplemented!();
            }
        }
    }

    fn extended_operations(&mut self, pc: u16){
        let extended_opcode = self.read_memory_following_u8(pc);
        match extended_opcode {
            0x06 => self.rlc_mem_hl(extended_opcode, pc),
            0x00 ... 0x07 => self.rlc_r(extended_opcode, pc),
            0x0E => self.rrc_mem_hl(extended_opcode, pc),
            0x08 ... 0x0F => self.rrc_r(extended_opcode, pc),
            0x16 => self.rl_mem_hl(extended_opcode, pc),
            0x10 ... 0x17 => self.rl_r(extended_opcode, pc),
            0x1E => self.rr_mem_hl(extended_opcode, pc),
            0x18 ... 0x1F => self.rr_r(extended_opcode, pc),
            0x26 => self.sla_mem_hl(extended_opcode, pc),
            0x20 ... 0x27 => self.sla_r(extended_opcode, pc),
            0x2E => self.sra_mem_hl(extended_opcode, pc),
            0x28 ... 0x2F => self.sra_r(extended_opcode, pc),
            0x36 => self.swap_mem_hl(extended_opcode, pc),
            0x30 ... 0x37 => self.swap_r(extended_opcode, pc),
            0x3E => self.srl_mem_hl(extended_opcode, pc),
            0x38 ... 0x3F => self.srl_r(extended_opcode, pc),
            0x46 | 0x4E | 0x56 | 0x5E |
            0x66 | 0x6E | 0x76 | 0x7E => self.bit_b_mem_hl(extended_opcode, pc),
            0x40 ... 0x7F => self.bit_b_r(extended_opcode, pc),
            0x86 | 0x8E | 0x96 | 0x9E |
            0xA6 | 0xAE | 0xB6 | 0xBE => self.res_b_mem_hl(extended_opcode, pc),
            0x80 ... 0xBF => self.res_b_r(extended_opcode, pc),
            0xC6 | 0xCE | 0xD6 | 0xDE |
            0xE6 | 0xEE | 0xF6 | 0xFE => self.set_b_mem_hl(extended_opcode, pc),
            0xC0 ... 0xFF => self.set_b_r(extended_opcode, pc),
            _ => {
                debug!("extended opcode: {:#04x}({:#010b})", extended_opcode, extended_opcode);
                unimplemented!();
            }
        }
    }

// -------------------------------------------- //
// 8-Bit Transfer and Input/Output Instructions //
// -------------------------------------------- //

    /// LD      r, r'
    /// 01 rrr rrr'
    pub fn ld_r_r(&mut self, opcode: u8, pc: u16){
        let target = RegisterR::new((opcode >> 3) & 0b111);
        let source = RegisterR::new(opcode & 0b111);
        let value = self.registers.read_r(source);
        debug!("{:#06X}: {:#04X} | LD   {:?}, {:?}({:?})", pc, opcode, target, source, value);
        self.registers.write_r(target, value);
        self.registers.inc_pc(1);
    }

    /// LD      r, n
    /// 00 rrr 110
    /// nnnnnnnn
    pub fn ld_r_n(&mut self, opcode: u8, pc: u16){
        let pc = self.registers.pc();
        let target = RegisterR::new((opcode >> 3) & 0b111);
        let value = self.read_memory_following_u8(pc);
        debug!("{:#06X}: {:#04X} | LD   {:?}, {:?}", pc, opcode, target, value);
        self.registers.write_r(target, value);
        self.registers.inc_pc(2);
    }

    /// LD      r, (HL)
    /// 01 rrr 110
    pub fn ld_r_mem_hl(&mut self, opcode: u8, pc: u16){
        let target = RegisterR::new((opcode >> 3) & 0b111);
        let address = self.registers.hl();
        let value = self.read_memory(address);
        debug!("{:#06X}: {:#04X} | LD   {:?}, {:?}[{:#06X}]({:?})", pc, opcode, target, RegisterSS::HL, address, value);
        self.registers.write_r(target, value);
        self.registers.inc_pc(1);
    }

    /// LD      (HL), r
    /// 01 110 rrr
    pub fn ld_mem_hl_r(&mut self, opcode: u8, pc: u16){
        let source = RegisterR::new(opcode & 0b111);
        let address = self.registers.hl();
        let value = self.registers.read_r(source);
        debug!("{:#06X}: {:#04X} | LD   {:?}[{:#06X}], {:?}", pc, opcode, RegisterSS::HL, address, value);
        self.write_memory(address, value);
        self.registers.inc_pc(1);
    }

    /// LD      (HL), n
    /// 00 110 110
    /// nnnnnnnn
    pub fn ld_mem_hl_n(&mut self, opcode: u8, pc: u16){
        let pc = self.registers.pc();
        let address = self.registers.hl();
        let value = self.read_memory_following_u8(pc);
        debug!("{:#06X}: {:#04X} | LD   {:?}[{:#06X}], {:?}", pc, opcode, RegisterSS::HL, address, value);
        self.write_memory(address, value);
        self.registers.inc_pc(2);
    }

    /// LD      A, (BC)
    /// 00 001 010
    pub fn ld_a_mem_bc(&mut self, opcode: u8, pc: u16){
        let address = self.registers.bc();
        let value = self.read_memory(address);
        debug!("{:#06X}: {:#04X} | LD   {:?}, {:?}[{:#06X}]({:?})", pc, opcode, RegisterR::A, RegisterSS::BC, address, value);
        self.registers.set_a(value);
        self.registers.inc_pc(1);
    }

    /// LD      A, (DE)
    /// 00 011 010
    pub fn ld_a_mem_de(&mut self, opcode: u8, pc: u16){
        let address = self.registers.de();
        let value = self.read_memory(address);
        debug!("{:#06X}: {:#04X} | LD   {:?}, {:?}[{:#06X}]({:?})", pc, opcode, RegisterR::A, RegisterSS::DE, address, value);
        self.registers.set_a(value);
        self.registers.inc_pc(1);
    }

    /// LD      A, (C)
    /// 11 110 010
    pub fn ld_a_mem_c(&mut self, opcode: u8, pc: u16){
        let address = 0xFF00  + self.registers.c() as u16;
        let value = self.read_memory(address);
        debug!("{:#06X}: {:#04X} | LD   {:?}, {:?}[{:#06X}]({:?})", pc, opcode, RegisterR::A, RegisterR::C, address, value);
        self.registers.set_a(value);
        self.registers.inc_pc(1);
    }

    /// LD      (C), A
    /// 11 100 010
    pub fn ld_mem_c_a(&mut self, opcode: u8, pc: u16){
        let address = 0xFF00  + self.registers.c() as u16;
        let value = self.registers.a();
        debug!("{:#06X}: {:#04X} | LD   {:?}[{:#06X}], {:?}", pc, opcode, RegisterR::C, address, RegisterR::A);
        self.write_memory(address, value);
        self.registers.inc_pc(1);
    }

    /// LD      A, (n)
    /// 11 110 000
    /// nnnnnnnn
    pub fn ld_a_mem_n(&mut self, opcode: u8, pc: u16){
        let pc = self.registers.pc();
        let address;
        let value;
        {
            let memory = self.memory.read().unwrap();
            address = 0xff00 + memory.following_u8(pc) as u16;
            value = memory.read(address);
        }
        debug!("{:#06X}: {:#04X} | LD   {:?}, [{:#06x}]({:?})", pc, opcode, RegisterR::A, address, value);

        self.registers.set_a(value);
        self.registers.inc_pc(2);
    }

    /// LD      (n), A
    /// 11 100 000
    /// nnnnnnnn
    pub fn ld_mem_n_a(&mut self, opcode: u8, pc: u16){
        let pc = self.registers.pc();
        let value = self.registers.a();
        let address;
        {
            let mut memory = self.memory.write().unwrap();
            address = 0xff00 + memory.following_u8(pc) as u16;
            memory.write(address, value);
        }
        debug!("{:#06X}: {:#04X} | LD   [{:#06X}], {:?}({:?})", pc, opcode, address, RegisterR::A, value);

        self.registers.inc_pc(2);
    }

    /// LD      A, (nn)
    /// 11 111 010
    /// nnnnnnnn
    /// nnnnnnnn
    pub fn ld_a_mem_nn(&mut self, opcode: u8, pc: u16){
        let pc = self.registers.pc();
        let address;
        let value;
        {
            let memory = self.memory.read().unwrap();
            address = memory.following_u16(pc);
            value = memory.read(address);
        }
        debug!("{:#06X}: {:#04X} | LD   {:?}, [{:#06x}]({:?})", pc, opcode, RegisterR::A, address, value);

        self.registers.set_a(value);
        self.registers.inc_pc(3);
    }

    /// LD      (nn), A
    /// 11 101 010
    /// nnnnnnnn
    /// nnnnnnnn
    pub fn ld_mem_nn_a(&mut self, opcode: u8, pc: u16){
        let pc = self.registers.pc();
        let value = self.registers.a();
        let address;
        {
            let mut memory = self.memory.write().unwrap();
            address = memory.following_u16(pc);
            memory.write(address, value);
        }
        debug!("{:#06X}: {:#04X} | LD   [{:#06X}], {:?}({:?})", pc, opcode, address, RegisterR::A, value);

        self.registers.inc_pc(3);
    }

    /// LD      A, (HLI)
    /// 00 101 010
    pub fn ld_a_mem_hli(&mut self, opcode: u8, pc: u16){
        let address = self.registers.hl();
        let value = self.read_memory(address);
        debug!("{:#06X}: {:#04X} | LD   {:?}, {:?}+[{:#06x}]({:?})", pc, opcode, RegisterR::A, RegisterDD::HL, address, value);

        self.registers.set_a(value);
        self.registers.set_hl(address.wrapping_add(1));
        self.registers.inc_pc(1);
    }

    /// LD      A, (HLD)
    /// 00 111 010
    pub fn ld_a_mem_hld(&mut self, opcode: u8, pc: u16){
        let address = self.registers.hl();
        let value = self.read_memory(address);
        debug!("{:#06X}: {:#04X} | LD   {:?}, {:?}-[{:#06x}]({:?})", pc, opcode, RegisterR::A, RegisterDD::HL, address, value);

        self.registers.set_a(value);
        self.registers.set_hl(address.wrapping_sub(1));
        self.registers.inc_pc(1);
    }

    /// LD      (BC), A
    /// 00 010 010
    pub fn ld_mem_bc_a(&mut self, opcode: u8, pc: u16){
        let pc = self.registers.pc();
        let value = self.registers.a();
        let address = self.registers.bc();
        debug!("{:#06X}: {:#04X} | LD   [{:#06X}], {:?}({:?})", pc, opcode, address, RegisterR::A, value);

        self.write_memory(address, value);
        self.registers.inc_pc(1);
    }

    /// LD      (DE), A
    /// 00 010 010
    pub fn ld_mem_de_a(&mut self, opcode: u8, pc: u16){
        let pc = self.registers.pc();
        let value = self.registers.a();
        let address = self.registers.de();
        debug!("{:#06X}: {:#04X} | LD   [{:#06X}], {:?}({:?})", pc, opcode, address, RegisterR::A, value);

        self.write_memory(address, value);
        self.registers.inc_pc(1);
    }

    /// LD      (HLI), A
    /// 00 100 010
    pub fn ld_mem_hli_a(&mut self, opcode: u8, pc: u16){
        let value = self.registers.a();
        let address = self.registers.hl();
        debug!("{:#06X}: {:#04X} | LD   {:?}+[{:#06X}], {:?}({:?})", pc, opcode, RegisterQQ::HL, address, RegisterR::A, value);

        self.write_memory(address, value);
        self.registers.set_hl(address+1);
        self.registers.inc_pc(1);
    }

    /// LD      (HLD), A
    /// 00 110 010
    pub fn ld_mem_hld_a(&mut self, opcode: u8, pc: u16){
        let value = self.registers.a();
        let address = self.registers.hl();
        debug!("{:#06X}: {:#04X} | LD   {:?}-[{:#06X}], {:?}({:?})", pc, opcode, RegisterQQ::HL, address, RegisterR::A, value);

        self.write_memory(address, value);
        self.registers.set_hl(address-1);
        self.registers.inc_pc(1);
    }

// ---------------------------- //
// 16-Bit Transfer Instructions //
// ---------------------------- //

    /// LD      dd, nn
    /// 00 dd0 001
    /// nnnnnnnn
    /// nnnnnnnn
    pub fn ld_dd_nn(&mut self, opcode: u8, pc: u16){
        let pc = self.registers.pc();
        let target = RegisterDD::new((opcode >> 4) & 0b11);
        let value = self.read_memory_following_u16(pc);
        debug!("{:#06X}: {:#04X} | LD   {:?}, {:?}", pc, opcode, target, value);
        self.registers.write_dd(target, value);
        self.registers.inc_pc(3);
    }

    /// LD      sp, hl
    /// 11 111 001
    pub fn ld_sp_hl(&mut self, opcode: u8, pc: u16){
        let value = self.registers.hl();
        debug!("{:#06X}: {:#04X} | LD   {:?}, {:?}({:?})", pc, opcode, RegisterDD::SP, RegisterDD::HL, value);
        self.registers.set_sp(value);
        self.registers.inc_pc(1);

    }

    /// PUSH    qq
    /// 11 qq0 101
    pub fn push_qq(&mut self, opcode: u8, pc: u16){
        let register = RegisterQQ::new((opcode >> 4) & 0b11);
        let value = self.registers.read_qq(register);
        let sp = self.registers.sp();
        debug!("{:#06X}: {:#04X} | PUSH {:?}({:?})", pc, opcode, register, value);
        {
            let mut memory = self.memory.write().unwrap();
            memory.push_u16_stack(value, sp);
        }
        self.registers.set_sp(sp-2);
        self.registers.inc_pc(1);
    }

    /// POP    qq
    /// 11 qq0 001
    pub fn pop_qq(&mut self, opcode: u8, pc: u16){
        let register = RegisterQQ::new((opcode >> 4) & 0b11);
        let sp = self.registers.sp();
        let value = {
            let memory = self.memory.read().unwrap();
            memory.pop_u16_stack(sp)
        };
        debug!("{:#06X}: {:#04X} | POP  {:?}({:?})", pc, opcode, register, value);

        self.registers.write_qq(register, value);
        self.registers.set_sp(sp+2);
        self.registers.inc_pc(1);
    }

    /// LDHL    SP, e
    /// 11 111 00
    /// eeeeeeee
    pub fn ldhl_sp_e(&mut self, opcode: u8, pc: u16){
        let pc = self.registers.pc();
        let sp = self.registers.sp();
        let value = self.read_memory_following_u8(pc);
        debug!("{:#06X}: {:#04X} | LDHL {:?}, {:?}", pc, opcode, RegisterDD::SP, value);
        use registers::FlagCalculationStatus::*;
        self.registers.set_flags_add_u16(sp, value as u16, Clear, Clear, Calculate, Calculate);
        self.registers.set_hl(sp.wrapping_add(value as i8 as u16));
        self.registers.inc_pc(2);
    }

    /// LD      (nn), SP
    /// 00 001 000
    /// nnnnnnnn
    /// nnnnnnnn
    pub fn ld_mem_nn_sp(&mut self, opcode: u8, pc: u16){
        let pc = self.registers.pc();
        let address = self.read_memory_following_u16(pc);
        let value = self.registers.sp();
        debug!("{:#06X}: {:#04X} | LD   {:#06x}, {:?}({:?})", pc, opcode, address, RegisterDD::SP, value);
        {
            let mut memory = self.memory.write().unwrap();
            memory.write(address, (value & 0xFF) as u8);
            memory.write(address+1, ((value >> 8) & 0xFF) as u8);
        }
        self.registers.inc_pc(3);
    }

// --------------------------------------------------- //
// 8-Bit Arithmetic and Logical Operation Instructions //
// --------------------------------------------------- //

    /// ADD     A, r
    /// 10 000 rrr
    pub fn add_a_r(&mut self, opcode: u8, pc: u16){
        let register = RegisterR::new(opcode & 0b111);
        let val_a = self.registers.a();
        let val_r = self.registers.read_r(register);
        debug!("{:#06X}: {:#04X} | ADD  {:?}({:?}), {:?}({:?})", pc, opcode, RegisterR::A, val_a, register, val_r);
        let result = val_a.wrapping_add(val_r);
        use registers::FlagCalculationStatus::*;
        self.registers.set_flags_add(val_a, val_r, Calculate, Clear, Calculate, Calculate);
        self.registers.set_a(result);
        self.registers.inc_pc(1);
    }

    /// ADD     A, n
    /// 11 000 110
    /// nnnnnnnn
    pub fn add_a_n(&mut self, opcode: u8, pc: u16){
        let pc = self.registers.pc();
        let val_a = self.registers.a();
        let val_n = self.read_memory_following_u8(pc);
        debug!("{:#06X}: {:#04X} | ADD  {:?}({:?}), ({:?})", pc, opcode, RegisterR::A, val_a, val_n);
        let result = val_a.wrapping_add(val_n);
        use registers::FlagCalculationStatus::*;
        self.registers.set_flags_add(val_a, val_n, Calculate, Clear, Calculate, Calculate);
        self.registers.set_a(result);
        self.registers.inc_pc(2);
    }

    /// ADD     A, (HL)
    /// 10 000 110
    pub fn add_a_mem_hl(&mut self, opcode: u8, pc: u16){
        let hl = self.registers.hl();
        let val_a = self.registers.a();
        let val_hl = self.read_memory(hl);
        debug!("{:#06X}: {:#04X} | ADD  {:?}({:?}), {:?}{:#06x}({:?})", pc, opcode, RegisterR::A, val_a, RegisterDD::HL, hl, val_hl);
        let result = val_a.wrapping_add(val_hl);
        use registers::FlagCalculationStatus::*;
        self.registers.set_flags_add(val_a, val_hl, Calculate, Clear, Calculate, Calculate);
        self.registers.set_a(result);
        self.registers.inc_pc(1);
    }

    /// ADC     A, r
    /// 10 001 rrr
    pub fn adc_a_r(&mut self, opcode: u8, pc: u16){
        let register = RegisterR::new(opcode & 0b111);
        let val_a = self.registers.a();
        let val_r = self.registers.read_r(register);
        let cy_flag = self.registers.get_flag_cy();
        debug!("{:#06X}: {:#04X} | ADC  {:?}({:?}), {:?}({:?})", pc, opcode, RegisterR::A, val_a, register, val_r);
        let result = val_a.wrapping_add(val_r).wrapping_add(cy_flag);
        use registers::FlagCalculationStatus::*;
        self.registers.set_flags_add_with_carry(val_a, val_r, Calculate, Clear, Calculate, Calculate);
        self.registers.set_a(result);
        self.registers.inc_pc(1);
    }

    /// ADC     A, n
    /// 11 001 110
    /// nnnnnnnn
    pub fn adc_a_n(&mut self, opcode: u8, pc: u16){
        let pc = self.registers.pc();
        let val_a = self.registers.a();
        let val_n = self.read_memory_following_u8(pc);
        let cy_flag = self.registers.get_flag_cy();
        debug!("{:#06X}: {:#04X} | ADC  {:?}({:?}), ({:?})", pc, opcode, RegisterR::A, val_a, val_n);
        let result = val_a.wrapping_add(val_n).wrapping_add(cy_flag);
        use registers::FlagCalculationStatus::*;
        self.registers.set_flags_add_with_carry(val_a, val_n, Calculate, Clear, Calculate, Calculate);
        self.registers.set_a(result);
        self.registers.inc_pc(2);
    }

    /// ADC     A, (HL)
    /// 10 001 110
    pub fn adc_a_mem_hl(&mut self, opcode: u8, pc: u16){
        let hl = self.registers.hl();
        let val_a = self.registers.a();
        let val_hl = self.read_memory(hl);
        let cy_flag = self.registers.get_flag_cy();
        debug!("{:#06X}: {:#04X} | ADC  {:?}({:?}), {:?}{:#06x}({:?})", pc, opcode, RegisterR::A, val_a, RegisterDD::HL, hl, val_hl);
        let result = val_a.wrapping_add(val_hl).wrapping_add(cy_flag);
        use registers::FlagCalculationStatus::*;
        self.registers.set_flags_add_with_carry(val_a, val_hl, Calculate, Clear, Calculate, Calculate);
        self.registers.set_a(result);
        self.registers.inc_pc(1);
    }

    /// SUB     A, r
    /// 10 010 rrr
    pub fn sub_a_r(&mut self, opcode: u8, pc: u16){
        let register = RegisterR::new(opcode & 0b111);
        let val_a = self.registers.a();
        let val_r = self.registers.read_r(register);
        debug!("{:#06X}: {:#04X} | SUB  {:?}({:?}), {:?}({:?})", pc, opcode, RegisterR::A, val_a, register, val_r);
        let result = val_a.wrapping_sub(val_r);
        use registers::FlagCalculationStatus::*;
        self.registers.set_flags_sub(val_a, val_r, Calculate, Set, Calculate, Calculate);
        self.registers.set_a(result);
        self.registers.inc_pc(1);
    }

    /// SUB     A, n
    /// 11 010 110
    /// nnnnnnnn
    pub fn sub_a_n(&mut self, opcode: u8, pc: u16){
        let pc = self.registers.pc();
        let val_a = self.registers.a();
        let val_n = self.read_memory_following_u8(pc);
        debug!("{:#06X}: {:#04X} | SUB  {:?}({:?}), ({:?})", pc, opcode, RegisterR::A, val_a, val_n);
        let result = val_a.wrapping_sub(val_n);
        use registers::FlagCalculationStatus::*;
        self.registers.set_flags_sub(val_a, val_n, Calculate, Set, Calculate, Calculate);
        self.registers.set_a(result);
        self.registers.inc_pc(2);
    }

    /// SUB     A, (HL)
    /// 10 010 110
    pub fn sub_a_mem_hl(&mut self, opcode: u8, pc: u16){
        let hl = self.registers.hl();
        let val_a = self.registers.a();
        let val_hl = self.read_memory(hl);
        debug!("{:#06X}: {:#04X} | SUB  {:?}({:?}), {:?}{:#06x}({:?})", pc, opcode, RegisterR::A, val_a, RegisterDD::HL, hl, val_hl);
        let result = val_a.wrapping_sub(val_hl);
        use registers::FlagCalculationStatus::*;
        self.registers.set_flags_sub(val_a, val_hl, Calculate, Set, Calculate, Calculate);
        self.registers.set_a(result);
        self.registers.inc_pc(1);
    }

    /// SBC     A, r
    /// 10 010 rrr
    pub fn sbc_a_r(&mut self, opcode: u8, pc: u16){
        let register = RegisterR::new(opcode & 0b111);
        let val_a = self.registers.a();
        let val_r = self.registers.read_r(register);
        let cy_flag = self.registers.get_flag_cy();
        debug!("{:#06X}: {:#04X} | SBC  {:?}({:?}), {:?}({:?})", pc, opcode, RegisterR::A, val_a, register, val_r);
        let result = val_a.wrapping_sub(val_r).wrapping_sub(cy_flag);
        use registers::FlagCalculationStatus::*;
        self.registers.set_flags_sub_with_carry(val_a, val_r, Calculate, Set, Calculate, Calculate);
        self.registers.set_a(result);
        self.registers.inc_pc(1);
    }

    /// SBC     A, n
    /// 11 010 110
    /// nnnnnnnn
    pub fn sbc_a_n(&mut self, opcode: u8, pc: u16){
        let pc = self.registers.pc();
        let val_a = self.registers.a();
        let val_n = self.read_memory_following_u8(pc);
        let cy_flag = self.registers.get_flag_cy();
        debug!("{:#06X}: {:#04X} | SBC  {:?}({:?}), ({:?})", pc, opcode, RegisterR::A, val_a, val_n);
        let result = val_a.wrapping_sub(val_n).wrapping_sub(cy_flag);
        use registers::FlagCalculationStatus::*;
        self.registers.set_flags_sub_with_carry(val_a, val_n, Calculate, Set, Calculate, Calculate);
        self.registers.set_a(result);
        self.registers.inc_pc(2);
    }

    /// SBC     A, (HL)
    /// 10 010 110
    pub fn sbc_a_mem_hl(&mut self, opcode: u8, pc: u16){
        let hl = self.registers.hl();
        let val_a = self.registers.a();
        let val_hl = self.read_memory(hl);
        let cy_flag = self.registers.get_flag_cy();
        debug!("{:#06X}: {:#04X} | SBC  {:?}({:?}), {:?}{:#06x}({:?})", pc, opcode, RegisterR::A, val_a, RegisterDD::HL, hl, val_hl);
        let result = val_a.wrapping_sub(val_hl).wrapping_sub(cy_flag);
        use registers::FlagCalculationStatus::*;
        self.registers.set_flags_sub_with_carry(val_a, val_hl, Calculate, Set, Calculate, Calculate);
        self.registers.set_a(result);
        self.registers.inc_pc(1);
    }

    /// AND     r
    /// 10 100 rrr
    pub fn and_r(&mut self, opcode: u8, pc: u16){
        let register = RegisterR::new(opcode & 0b111);
        let value = self.registers.read_r(register);
        let reg_a_value = self.registers.a();
        debug!("{:#06X}: {:#04X} | AND  {:?}({:?}), {:?}({:?})", pc, opcode, RegisterR::A, reg_a_value, register, value);
        let result = reg_a_value & value;
        self.registers.set_flags(if result == 0 {1} else {0}, 0,1, 0);
        self.registers.set_a(result);
        self.registers.inc_pc(1);
    }

    /// AND     n
    /// 11 100 110
    /// nnnnnnnn
    pub fn and_n(&mut self, opcode: u8, pc: u16){
        let pc = self.registers.pc();
        let value = self.read_memory_following_u8(pc);
        let reg_a_value = self.registers.a();
        debug!("{:#06X}: {:#04X} | AND  {:?}({:?}), {:?}", pc, opcode, RegisterR::A, reg_a_value, value);
        let result = reg_a_value & value;
        self.registers.set_flags(if result == 0 {1} else {0}, 0,1, 0);
        self.registers.set_a(result);
        self.registers.inc_pc(2);
    }

    /// AND     (HL)
    /// 10 100 110
    pub fn and_mem_hl(&mut self, opcode: u8, pc: u16){
        let address = self.registers.hl();
        let value = self.read_memory(address);
        let reg_a_value = self.registers.a();
        debug!("{:#06X}: {:#04X} | AND  {:?}({:?}), {:?}[{:?}]({:?})", pc, opcode,
                 RegisterR::A, reg_a_value, RegisterDD::HL, address, value);
        let result = reg_a_value & value;
        self.registers.set_flags(if result == 0 {1} else {0}, 0,1, 0);
        self.registers.set_a(result);
        self.registers.inc_pc(1);
    }

    /// OR      r
    /// 10 110 rrr
    pub fn or_r(&mut self, opcode: u8, pc: u16){
        let register = RegisterR::new(opcode & 0b111);
        let value = self.registers.read_r(register);
        let reg_a_value = self.registers.a();
        debug!("{:#06X}: {:#04X} | OR   {:?}({:?}), {:?}({:?})", pc, opcode, RegisterR::A, reg_a_value, register, value);
        let result = reg_a_value | value;
        self.registers.set_flags(if result == 0 {1} else {0}, 0,0, 0);
        self.registers.set_a(result);
        self.registers.inc_pc(1);
    }

    /// OR      n
    /// 11 110 110
    /// nnnnnnnn
    pub fn or_n(&mut self, opcode: u8, pc: u16){
        let pc = self.registers.pc();
        let value = self.read_memory_following_u8(pc);
        let reg_a_value = self.registers.a();
        debug!("{:#06X}: {:#04X} | OR   {:?}({:?}), {:?}", pc, opcode, RegisterR::A, reg_a_value, value);
        let result = reg_a_value | value;
        self.registers.set_flags(if result == 0 {1} else {0}, 0,0, 0);
        self.registers.set_a(result);
        self.registers.inc_pc(2);
    }

    /// OR      (HL)
    /// 10 110 110
    pub fn or_mem_hl(&mut self, opcode: u8, pc: u16){
        let address = self.registers.hl();
        let value = self.read_memory(address);
        let reg_a_value = self.registers.a();
        debug!("{:#06X}: {:#04X} | OR   {:?}({:?}), {:?}[{:?}]({:?})", pc, opcode,
                 RegisterR::A, reg_a_value, RegisterDD::HL, address, value);
        let result = reg_a_value | value;
        self.registers.set_flags(if result == 0 {1} else {0}, 0,0, 0);
        self.registers.set_a(result);
        self.registers.inc_pc(1);
    }

    /// XOR     r
    /// 10 101 rrr
    pub fn xor_r(&mut self, opcode: u8, pc: u16){
        let register = RegisterR::new(opcode & 0b111);
        let value = self.registers.read_r(register);
        let reg_a_value = self.registers.a();
        debug!("{:#06X}: {:#04X} | XOR  {:?}({:?}), A({:?})", pc, opcode, register, value, reg_a_value);
        let result = reg_a_value ^ value;
        self.registers.set_flags(if result == 0 {1} else {0}, 0,0, 0);
        self.registers.set_a(result);
        self.registers.inc_pc(1);
    }

    /// XOR     n
    /// 11 101 110
    /// nnnnnnnn
    pub fn xor_n(&mut self, opcode: u8, pc: u16){
        let pc = self.registers.pc();
        let value = self.read_memory_following_u8(pc);
        let reg_a_value = self.registers.a();
        debug!("{:#06X}: {:#04X} | XOR  {:?}({:?}), {:?}", pc, opcode, RegisterR::A, reg_a_value, value);
        let result = reg_a_value ^ value;
        self.registers.set_flags(if result == 0 {1} else {0}, 0,0, 0);
        self.registers.set_a(result);
        self.registers.inc_pc(2);
    }

    /// XOR     (HL)
    /// 10 101 110
    pub fn xor_mem_hl(&mut self, opcode: u8, pc: u16){
        let address = self.registers.hl();
        let value = self.read_memory(address);
        let reg_a_value = self.registers.a();
        debug!("{:#06X}: {:#04X} | XOR  {:?}({:?}), {:?}[{:?}]({:?})", pc, opcode, RegisterR::A, reg_a_value, RegisterDD::HL, address, value);
        let result = reg_a_value ^ value;
        self.registers.set_flags(if result == 0 {1} else {0}, 0,0, 0);
        self.registers.set_a(result);
        self.registers.inc_pc(1);
    }

    /// CP      r
    /// 10 111 rrr
    pub fn cp_r(&mut self, opcode: u8, pc: u16){
        let register = RegisterR::new(opcode & 0b111);
        let val_a = self.registers.a();
        let val_r = self.registers.read_r(register);
        debug!("{:#06X}: {:#04X} | CP   {:?}({:?}), {:?}({:?})", pc, opcode, RegisterR::A, val_a, register, val_r);
        use registers::FlagCalculationStatus::*;
        self.registers.set_flags_sub(val_a, val_r, Calculate, Set, Calculate, Calculate);
        self.registers.inc_pc(1);
    }

    /// CP      n
    /// 11 111 110
    /// nnnnnnnn
    pub fn cp_n(&mut self, opcode: u8, pc: u16){
        let pc = self.registers.pc();
        let val_a = self.registers.a();
        let val_n = self.read_memory_following_u8(pc);
        debug!("{:#06X}: {:#04X} | CP   {:?}({:?}), ({:?})", pc, opcode, RegisterR::A, val_a, val_n);
        use registers::FlagCalculationStatus::*;
        self.registers.set_flags_sub(val_a, val_n, Calculate, Set, Calculate, Calculate);
        self.registers.inc_pc(2);
    }

    /// CP      (HL)
    /// 10 111 110
    pub fn cp_mem_hl(&mut self, opcode: u8, pc: u16){
        let hl = self.registers.hl();
        let val_a = self.registers.a();
        let val_hl = self.read_memory(hl);
        debug!("{:#06X}: {:#04X} | SUB  {:?}({:?}), {:?}{:#06x}({:?})", pc, opcode, RegisterR::A, val_a, RegisterDD::HL, hl, val_hl);
        use registers::FlagCalculationStatus::*;
        self.registers.set_flags_sub(val_a, val_hl, Calculate, Set, Calculate, Calculate);
        self.registers.inc_pc(1);
    }

    /// INC     r
    /// 00 rrr 100
    pub fn inc_r(&mut self, opcode: u8, pc: u16){
        let register = RegisterR::new((opcode >> 3) & 0b111);
        let value = self.registers.read_r(register);
        debug!("{:#06X}: {:#04X} | INC  {:?}({:?})", pc, opcode, register, value);
        use registers::FlagCalculationStatus::*;
        self.registers.set_flags_add(value, 1,
                                     Calculate, Clear, Calculate, Ignore);
        self.registers.write_r(register, value.wrapping_add(1));
        self.registers.inc_pc(1);
    }

    /// INC     (HL)
    /// 00 110 100
    pub fn inc_mem_hl(&mut self, opcode: u8, pc: u16){
        let address = self.registers.hl();
        let value = {
            let mut memory = self.memory.write().unwrap();
            let value = memory.read(address);
            memory.write(address, value.wrapping_add(1));
            value
        };
        debug!("{:#06X}: {:#04X} | INC  {:?}{:#06x}({:?})", pc, opcode, RegisterDD::HL, address, value);
        use registers::FlagCalculationStatus::*;
        self.registers.set_flags_add(value, 1,
                                     Calculate, Clear, Calculate, Ignore);
        self.registers.inc_pc(1);
    }

    /// DEC     r
    /// 00 rrr 101
    pub fn dec_r(&mut self, opcode: u8, pc: u16){
        let register = RegisterR::new((opcode >> 3) & 0b111);
        let value = self.registers.read_r(register);
        debug!("{:#06X}: {:#04X} | DEC  {:?}({:?})", pc, opcode, register, value);
        use registers::FlagCalculationStatus::*;
        self.registers.set_flags_sub(value, 1,
                                     Calculate, Set, Calculate, Ignore);
        self.registers.write_r(register, value.wrapping_sub(1));
        self.registers.inc_pc(1);
    }

    /// DEC     (HL)
    /// 00 110 101
    pub fn dec_mem_hl(&mut self, opcode: u8, pc: u16){
        let address = self.registers.hl();
        let value = {
            let mut memory = self.memory.write().unwrap();
            let value = memory.read(address);
            memory.write(address, value.wrapping_sub(1));
            value
        };
        debug!("{:#06X}: {:#04X} | DEC  {:?}{:#06x}({:?})", pc, opcode, RegisterDD::HL, address, value);
        use registers::FlagCalculationStatus::*;
        self.registers.set_flags_sub(value, 1,
                                     Calculate, Set, Calculate, Ignore);
        self.registers.inc_pc(1);
    }

// ---------------------------------------- //
// 16-Bit Arithmetic Operation Instructions //
// ---------------------------------------- //

    /// ADD     HL, ss
    /// 00 ss1 001
    pub fn add_hl_ss(&mut self, opcode: u8, pc: u16){
        let register = RegisterSS::new((opcode >> 4) & 0b111);
        let value = self.registers.read_ss(register);
        let reg_hl_value = self.registers.hl();
        debug!("{:#06X}: {:#04X} | ADD  {:?}({:?}), {:?}({:?})", pc, opcode, RegisterSS::HL, reg_hl_value, register, value);
        let result = reg_hl_value.wrapping_add(value);
        use registers::FlagCalculationStatus::*;
        self.registers.set_flags_add_u16(reg_hl_value, value as u16, Ignore, Clear, Calculate, Calculate);
        self.registers.set_hl(result);
        self.registers.inc_pc(1);
    }

    /// ADD     SP, e
    /// 11 101 000
    /// eeeeeeee
    pub fn add_sp_e(&mut self, opcode: u8, pc: u16){
        let pc = self.registers.pc();
        let val_sp = self.registers.sp();
        let val_n = self.read_memory_following_u8(pc);
        debug!("{:#06X}: {:#04X} | ADD  {:?}({:?}), ({:?})", pc, opcode, RegisterSS::SP, val_sp, val_n);
        let result = Self::add_signed(val_sp, val_n);
        use registers::FlagCalculationStatus::*;
        self.registers.set_flags_add_u16(val_sp, val_n as u16, Clear, Clear, Calculate, Calculate);
        self.registers.set_sp(result);
        self.registers.inc_pc(2);
    }

    /// INC     ss
    /// 00 ss0 011
    pub fn inc_ss(&mut self, opcode: u8, pc: u16){
        let register = RegisterSS::new((opcode >> 4) & 0b11);
        let value = self.registers.read_ss(register);
        debug!("{:#06X}: {:#04X} | INC  {:?}({:?})", pc, opcode, register, value);
        use registers::FlagCalculationStatus::*;
        self.registers.set_flags_add_u16(value, 1,
                                     Ignore, Ignore, Ignore, Ignore);
        self.registers.write_ss(register, value+1);
        self.registers.inc_pc(1);
    }

    /// DEC     ss
    /// 00 ss1 011
    pub fn dec_ss(&mut self, opcode: u8, pc: u16){
        let register = RegisterSS::new((opcode >> 4) & 0b11);
        let value = self.registers.read_ss(register);
        debug!("{:#06X}: {:#04X} | DEC  {:?}({:?})", pc, opcode, register, value);
        use registers::FlagCalculationStatus::*;
        self.registers.set_flags_sub_u16(value, 1,
                                         Ignore, Ignore, Ignore, Ignore);
        self.registers.write_ss(register, value-1);
        self.registers.inc_pc(1);
    }

// ------------------------- //
// Rotate Shift Instructions //
// ------------------------- //

    /// RLCA
    /// 00 000 111
    pub fn rlca(&mut self, opcode: u8, pc: u16){
        self.rlc_r_internal(opcode, pc,RegisterR::A, false);
        self.registers.inc_pc(1);
    }

    /// RLA
    /// 00 010 111
    pub fn rla(&mut self, opcode: u8, pc: u16){
        self.rl_r_internal(opcode, pc,RegisterR::A, false);
        self.registers.inc_pc(1);
    }

    /// RRCA
    /// 00 001 111
    pub fn rrca(&mut self, opcode: u8, pc: u16){
        self.rrc_r_internal(opcode, pc,RegisterR::A, false);
        self.registers.inc_pc(1);
    }

    /// RRA
    /// 00 011 111
    pub fn rra(&mut self, opcode: u8, pc: u16){
        self.rr_r_internal(opcode, pc,RegisterR::A, false);
        self.registers.inc_pc(1);
    }

    /// RLC     r
    /// 11 001 011
    /// 00 000 rrr
    pub fn rlc_r(&mut self, ext_opcode: u8, pc: u16){
        let register = RegisterR::new(ext_opcode & 0b111);
        self.rlc_r_internal(ext_opcode, pc, register, true);
        self.registers.inc_pc(2);
    }

    /// RLC     (HL)
    /// 11 001 011
    /// 00 000 110
    pub fn rlc_mem_hl(&mut self, ext_opcode: u8, pc: u16){
        let address = self.registers.hl();
        let value = self.read_memory(address);
        debug!("{:#06X}: {:#04X} | RLC   {:?}[{:#06x}]({:#010b})", pc, ext_opcode, RegisterDD::HL, address, value);
        let rotated = self.rlc_m(value, true);
        self.write_memory(address, rotated);
        self.registers.inc_pc(2);
    }

    /// RL      r
    /// 11 001 011
    /// 00 010 rrr
    pub fn rl_r(&mut self, ext_opcode: u8, pc: u16){
        let register = RegisterR::new(ext_opcode & 0b111);
        self.rl_r_internal(ext_opcode, pc, register, true);
        self.registers.inc_pc(2);
    }

    /// RL      (HL)
    /// 11 001 011
    /// 00 010 110
    pub fn rl_mem_hl(&mut self, ext_opcode: u8, pc: u16){
        let address = self.registers.hl();
        let value = self.read_memory(address);
        debug!("{:#06X}: {:#04X} | RL   {:?}[{:#06x}]({:#010b})", pc, ext_opcode, RegisterDD::HL, address, value);
        let rotated = self.rl_m(value, true);
        self.write_memory(address, rotated);
        self.registers.inc_pc(2);
    }

    /// RRC     r
    /// 11 001 011
    /// 00 001 rrr
    pub fn rrc_r(&mut self, ext_opcode: u8, pc: u16){
        let register = RegisterR::new(ext_opcode & 0b111);
        self.rrc_r_internal(ext_opcode, pc, register, true);
        self.registers.inc_pc(2);
    }

    /// RRC     (HL)
    /// 11 001 011
    /// 00 001 110
    pub fn rrc_mem_hl(&mut self, ext_opcode: u8, pc: u16){
        let address = self.registers.hl();
        let value = self.read_memory(address);
        debug!("{:#06X}: {:#04X} | RRC   {:?}[{:#06x}]({:#010b})", pc, ext_opcode, RegisterDD::HL, address, value);
        let rotated = self.rrc_m(value, true);
        self.write_memory(address, rotated);
        self.registers.inc_pc(2);
    }

    /// RR      r
    /// 11 001 011
    /// 00 011 rrr
    pub fn rr_r(&mut self, ext_opcode: u8, pc: u16){
        let register = RegisterR::new(ext_opcode & 0b111);
        self.rr_r_internal(ext_opcode, pc, register, true);
        self.registers.inc_pc(2);
    }

    /// RR      (HL)
    /// 11 001 011
    /// 00 011 110
    pub fn rr_mem_hl(&mut self, ext_opcode: u8, pc: u16){
        let address = self.registers.hl();
        let value = self.read_memory(address);
        debug!("{:#06X}: {:#04X} | RR   {:?}[{:#06x}]({:#010b})", pc, ext_opcode, RegisterDD::HL, address, value);
        let rotated = self.rr_m(value, true);
        self.write_memory(address, rotated);
        self.registers.inc_pc(2);
    }

    fn rlc_r_internal(&mut self, opcode: u8, pc: u16, register: RegisterR, calc_zero: bool) {
        let value = self.registers.read_r(register);
        debug!("{:#06X}: {:#04X} | RLC   {:?}({:#010b})", pc, opcode, register, value);
        let rotated = self.rlc_m(value, calc_zero);
        self.registers.write_r(register, rotated);
    }

    fn rl_r_internal(&mut self, opcode: u8, pc: u16, register: RegisterR, calc_zero: bool) {
        let value = self.registers.read_r(register);
        debug!("{:#06X}: {:#04X} | RL   {:?}({:#010b})", pc, opcode, register, value);
        let rotated = self.rl_m(value, calc_zero);
        self.registers.write_r(register, rotated);
    }

    fn rrc_r_internal(&mut self, opcode: u8, pc: u16, register: RegisterR, calc_zero: bool) {
        let value = self.registers.read_r(register);
        debug!("{:#06X}: {:#04X} | RRC   {:?}({:#010b})", pc, opcode, register, value);
        let rotated = self.rrc_m(value, calc_zero);
        self.registers.write_r(register, rotated);
    }

    fn rr_r_internal(&mut self, opcode: u8, pc: u16, register: RegisterR, calc_zero: bool) {
        let value = self.registers.read_r(register);
        debug!("{:#06X}: {:#04X} | RR   {:?}({:#010b})", pc, opcode, register, value);
        let rotated = self.rr_m(value, calc_zero);
        self.registers.write_r(register, rotated);
    }

    fn rlc_m(&mut self, value: u8, calc_zero: bool) -> u8 {
        let mut flags = self.registers.f();
        let bit7 = (value >> 7) & 1;
        let cy = self.registers.get_flag_cy();
        let rotated = (value << 1) | bit7;
        let flags = CPU::calc_flags_for_shift_and_rotate(flags, bit7, rotated, calc_zero);

        self.registers.set_f(flags);
        rotated
    }

    fn rl_m(&mut self, value: u8, calc_zero: bool) -> u8 {
        let mut flags = self.registers.f();
        let bit7 = (value >> 7) & 1;
        let cy = self.registers.get_flag_cy();
        let rotated = (value << 1) | cy;
        let flags = CPU::calc_flags_for_shift_and_rotate(flags, bit7, rotated, calc_zero);

        self.registers.set_f(flags);
        rotated
    }

    fn rrc_m(&mut self, value: u8, calc_zero: bool) -> u8 {
        let mut flags = self.registers.f();
        let bit0 = (value) & 1;
        let cy = self.registers.get_flag_cy();
        let rotated = (value >> 1) | (bit0 << 7);
        let flags = CPU::calc_flags_for_shift_and_rotate(flags, bit0, rotated, calc_zero);

        self.registers.set_f(flags);
        rotated
    }

    fn rr_m(&mut self, value: u8, calc_zero: bool) -> u8 {
        let mut flags = self.registers.f();
        let bit0 = value & 1;
        let cy = self.registers.get_flag_cy();
        let rotated = (value >> 1) | (cy << 7);
        let flags = CPU::calc_flags_for_shift_and_rotate(flags, bit0, rotated, calc_zero);

        self.registers.set_f(flags);
        rotated
    }

    fn calc_flags_for_shift_and_rotate(mut flags: u8, bit_value: u8, calculated_result: u8, calc_zero: bool) -> u8 {
        flags = bit_op::change_bit_to(flags, 4, bit_value);
        flags = bit_op::clear_bit(flags, 5);
        flags = bit_op::clear_bit(flags, 6);

        if calc_zero {
            if calculated_result == 0 {
                bit_op::set_bit(flags, 7)
            } else {
                bit_op::clear_bit(flags, 7)
            }
        } else {
            bit_op::clear_bit(flags, 7)
        }
    }

    /// SLA     r
    /// 11 001 011
    /// 00 100 rrr
    pub fn sla_r(&mut self, ext_opcode: u8, pc: u16){
        let mut register = RegisterR::new(ext_opcode & 0b111);
        let value = self.registers.read_r(register);
        debug!("{:#06X}: {:#04X} | SLA   {:?}({:#010b})", pc, ext_opcode, register, value);
        let bit7 = (value>>7) & 1;
        let result = value << 1;
        let flags = Self::calc_flags_for_shift_and_rotate(self.registers.f(), bit7, result, true);
        self.registers.set_f(flags);
        self.registers.write_r(register, result);
        self.registers.inc_pc(2);
    }

    /// SLA     (HL)
    /// 11 001 011
    /// 00 100 110
    pub fn sla_mem_hl(&mut self, ext_opcode: u8, pc: u16){
        let mut address = self.registers.hl();
        let value = self.read_memory(address);
        debug!("{:#06X}: {:#04X} | SLA   {:?}[{:#06x}]({:#010b})", pc, ext_opcode, RegisterDD::HL, address, value);
        let bit7 = (value>>7) & 1;
        let result = value << 1;
        let flags = Self::calc_flags_for_shift_and_rotate(self.registers.f(), bit7, result, true);
        self.registers.set_f(flags);
        self.write_memory(address, result);
        self.registers.inc_pc(2);
    }

    /// SRA     r
    /// 11 001 011
    /// 00 100 rrr
    pub fn sra_r(&mut self, ext_opcode: u8, pc: u16){
        let mut register = RegisterR::new(ext_opcode & 0b111);
        let value = self.registers.read_r(register);
        debug!("{:#06X}: {:#04X} | SRA   {:?}({:#010b})", pc, ext_opcode, register, value);
        let bit0 = value & 1;
        let bit7 = (value >> 7) & 1;
        let result = (value >> 1) | (bit7 << 7);
        let flags = Self::calc_flags_for_shift_and_rotate(self.registers.f(), bit0, result, true);
        self.registers.set_f(flags);
        self.registers.write_r(register, result);
        self.registers.inc_pc(2);
    }

    /// SRA     (HL)
    /// 11 001 011
    /// 00 100 110
    pub fn sra_mem_hl(&mut self, ext_opcode: u8, pc: u16){
        let mut address = self.registers.hl();
        let value = self.read_memory(address);
        debug!("{:#06X}: {:#04X} | SRA   {:?}[{:#06x}]({:#010b})", pc, ext_opcode, RegisterDD::HL, address, value);
        let bit0 = value & 1;
        let bit7 = (value >> 7) & 1;
        let result = (value >> 1) | (bit7 << 7);
        let flags = Self::calc_flags_for_shift_and_rotate(self.registers.f(), bit0, result, true);
        self.registers.set_f(flags);
        self.write_memory(address, result);
        self.registers.inc_pc(2);
    }

    /// SRL     r
    /// 11 001 011
    /// 00 111 rrr
    pub fn srl_r(&mut self, ext_opcode: u8, pc: u16){
        let mut register = RegisterR::new(ext_opcode & 0b111);
        let value = self.registers.read_r(register);
        debug!("{:#06X}: {:#04X} | SRL   {:?}({:#010b})", pc, ext_opcode, register, value);
        let bit0 = value & 1;
        let result = value >> 1;
        let flags = Self::calc_flags_for_shift_and_rotate(self.registers.f(), bit0, result, true);
        self.registers.set_f(flags);
        self.registers.write_r(register, result);
        self.registers.inc_pc(2);
    }

    /// SRL     (HL)
    /// 11 001 011
    /// 00 111 110
    pub fn srl_mem_hl(&mut self, ext_opcode: u8, pc: u16){
        let mut address = self.registers.hl();
        let value = self.read_memory(address);
        debug!("{:#06X}: {:#04X} | SRL   {:?}[{:#06x}]({:#010b})", pc, ext_opcode, RegisterDD::HL, address, value);
        let bit0 = value & 1;
        let result = value >> 1;
        let flags = Self::calc_flags_for_shift_and_rotate(self.registers.f(), bit0, result, true);
        self.registers.set_f(flags);
        self.write_memory(address, result);
        self.registers.inc_pc(2);
    }

    /// SWAP    r
    /// 11 001 011
    /// 00 110 rrr
    pub fn swap_r(&mut self, ext_opcode: u8, pc: u16){
        let mut register = RegisterR::new(ext_opcode & 0b111);
        let value = self.registers.read_r(register);
        debug!("{:#06X}: {:#04X} | SWAP  {:?}({:#010b})", pc, ext_opcode, register, value);
        let result = ((value & 0b111) << 4) | (value >> 4) & 0b1111;
        self.registers.set_flags(if result == 0 {1} else {0}, 0, 0, 0);
        self.registers.write_r(register, result);
        self.registers.inc_pc(2);
    }

    /// SWAP    (HL)
    /// 11 001 011
    /// 00 110 110
    pub fn swap_mem_hl(&mut self, ext_opcode: u8, pc: u16){
        let mut address = self.registers.hl();
        let value = self.read_memory(address);
        debug!("{:#06X}: {:#04X} | SWAP  {:?}[{:#06x}]({:#010b})", pc, ext_opcode, RegisterDD::HL, address, value);
        let result = ((value & 0b111) << 4) | (value >> 4) & 0b1111;
        self.registers.set_flags(if result == 0 {1} else {0}, 0, 0, 0);
        self.write_memory(address, result);
        self.registers.inc_pc(2);
    }

// -------------- //
// Bit Operations //
// -------------- //

    /// BIT     b, r
    /// 11 001 011
    /// 01 bbb rrr
    pub fn bit_b_r(&mut self, ext_opcode: u8, pc: u16){
        let register = RegisterR::new(ext_opcode & 0b111);
        let value = self.registers.read_r(register);
        let bit = (ext_opcode >> 3) & 0b111;
        debug!("{:#06X}: {:#04X} | BIT  {:?}, {:?}({:#010b})", pc, ext_opcode, bit, register, value);

        let bit_value = if ((value >> bit) & 0b1) == 0 {1} else {0};
        let mut flags = self.registers.f();
        flags = bit_op::set_bit(flags, 5);
        flags = bit_op::clear_bit(flags, 6);
        flags = bit_op::change_bit_to(flags, 7, bit_value);
        self.registers.set_f(flags);
        self.registers.inc_pc(2);
    }

    /// BIT     b, (HL)
    /// 11 001 011
    /// 01 bbb 110
    pub fn bit_b_mem_hl(&mut self, ext_opcode: u8, pc: u16){
        let address = self.registers.hl();
        let value = self.read_memory(address);
        let bit = (ext_opcode >> 3) & 0b111;
        debug!("{:#06X}: {:#04X} | BIT  {:?}, [{:#06x}]({:#010b})", pc, ext_opcode, bit, address, value);

        let bit_value = if ((value >> bit) & 0b1) == 0 {1} else {0};
        let mut flags = self.registers.f();
        flags = bit_op::set_bit(flags, 5);
        flags = bit_op::clear_bit(flags, 6);
        flags = bit_op::change_bit_to(flags, 7, bit_value);
        self.registers.set_f(flags);
        self.registers.inc_pc(2);
    }

    /// SET     b, r
    /// 11 001 011
    /// 11 bbb rrr
    pub fn set_b_r(&mut self, ext_opcode: u8, pc: u16){
        let register = RegisterR::new(ext_opcode & 0b111);
        let value = self.registers.read_r(register);
        let bit = (ext_opcode >> 3) & 0b111;
        debug!("{:#06X}: {:#04X} | SET  {:?}, {:?}({:#010b})", pc, ext_opcode, bit, register, value);

        let result = bit_op::set_bit(value, bit);
        self.registers.write_r(register, result);
        self.registers.inc_pc(2);
    }

    /// SET     b, (HL)
    /// 11 001 011
    /// 11 bbb 110
    pub fn set_b_mem_hl(&mut self, ext_opcode: u8, pc: u16){
        let address = self.registers.hl();
        let value = self.read_memory(address);
        let bit = (ext_opcode >> 3) & 0b111;
        debug!("{:#06X}: {:#04X} | SET  {:?}, [{:#06x}]({:#010b})", pc, ext_opcode, bit, address, value);

        let result = bit_op::set_bit(value, bit);
        self.write_memory(address, result);
        self.registers.inc_pc(2);
    }

    /// RES     b, r
    /// 11 001 011
    /// 10 bbb rrr
    pub fn res_b_r(&mut self, ext_opcode: u8, pc: u16){
        let register = RegisterR::new(ext_opcode & 0b111);
        let value = self.registers.read_r(register);
        let bit = (ext_opcode >> 3) & 0b111;
        debug!("{:#06X}: {:#04X} | RES  {:?}, {:?}({:#010b})", pc, ext_opcode, bit, register, value);

        let result = bit_op::clear_bit(value, bit);
        self.registers.write_r(register, result);
        self.registers.inc_pc(2);
    }

    /// RES     b, (HL)
    /// 11 001 011
    /// 10 bbb 110
    pub fn res_b_mem_hl(&mut self, ext_opcode: u8, pc: u16){
        let address = self.registers.hl();
        let value = self.read_memory(address);
        let bit = (ext_opcode >> 3) & 0b111;
        debug!("{:#06X}: {:#04X} | RES  {:?}, [{:#06x}]({:#010b})", pc, ext_opcode, bit, address, value);

        let result = bit_op::clear_bit(value, bit);
        self.write_memory(address, result);
        self.registers.inc_pc(2);
    }

// --------------- //
// Jump Operations //
// --------------- //

    /// JP      nn
    /// 11 000 011
    /// nnnnnnnn
    /// nnnnnnnn
    pub fn jp_nn(&mut self, opcode: u8, pc: u16){
        let address = self.read_memory_following_u16(pc);
        debug!("{:#06X}: {:#04X} | JP   {:#06X}", pc, opcode, address);
        self.registers.set_pc(address);
    }

    /// JP      cc, nn
    /// 11 0cc 011
    /// nnnnnnnn
    /// nnnnnnnn
    pub fn jp_cc_nn(&mut self, opcode: u8, pc: u16){
        let condition = Condition::new((opcode>>3) & 0b11);
        let address = self.read_memory_following_u16(pc);
        if self.registers.check_condition(condition) {
            debug!("{:#06X}: {:#04X} | JP   {:?}, {:#06X} ||| (jp)", pc, opcode, condition, address);
            self.registers.set_pc(address);
        } else {
            debug!("{:#06X}: {:#04X} | JP   {:?}, {:#06X}  ||| (skip)", pc, opcode, condition, address);
            self.registers.inc_pc(3);
        }
    }

    /// JR      e
    /// 00 011 000
    /// eeeeeeee
    pub fn jr_e(&mut self, opcode: u8, pc: u16){
        let value = self.read_memory_following_u8(pc);
        debug!("{:#06X}: {:#04X} | JR   {:?}", pc, opcode, value as i8);
        let pc = Self::add_signed(pc, value);
        self.registers.set_pc(pc+2);
    }

    /// JR      cc, e
    /// 00 1cc 000
    /// eeeeeeee
    pub fn jr_cc_e(&mut self, opcode: u8, pc: u16){
        let condition = Condition::new((opcode>>3) & 0b11);
        let value = self.read_memory_following_u8(pc);
        if self.registers.check_condition(condition) {
            debug!("{:#06X}: {:#04X} | JR   {:?}, {:?} ||| (jp)", pc, opcode, condition, value as i8);
            let pc = Self::add_signed(pc, value);
            self.registers.set_pc(pc+2);
        } else {
            debug!("{:#06X}: {:#04X} | JR   {:?}, {:?} ||| (skip)", pc, opcode, condition, value as i8);
            self.registers.inc_pc(2);
        }
    }

    /// JP      (HL)
    /// 11 101 001
    pub fn jp_mem_hl(&mut self, opcode: u8, pc: u16){
        let condition = Condition::new((opcode>>3) & 0b11);
        let address = self.registers.hl();
        debug!("{:#06X}: {:#04X} | JP   {:?}({:#06X})", pc, opcode, RegisterDD::HL, address);
        self.registers.set_pc(address);
    }

// ---------------------------- //
// Call and Return Instructions //
// ---------------------------- //

    /// CALL    nn
    /// 11 001 101
    /// nnnnnnnn
    /// nnnnnnnn
    pub fn call_nn(&mut self, opcode: u8, pc: u16){
        let mut pc = self.registers.pc();
        let address = self.read_memory_following_u16(pc);
        let mut sp = self.registers.sp();
        debug!("{:#06X}: {:#04X} | CALL {:#06x}", pc, opcode, address);
        {
            let mut memory = self.memory.write().unwrap();
            memory.push_u16_stack(pc+3, sp);
        }
        sp = sp -2;
        pc = address;
        self.registers.set_sp(sp);
        self.registers.set_pc(pc);
    }

    /// CALL    cc, nn
    /// 11 0cc 100
    /// nnnnnnnn
    /// nnnnnnnn
    pub fn call_cc_nn(&mut self, opcode: u8, pc: u16){
        let mut pc = self.registers.pc();
        let condition = Condition::new((opcode>>3) & 0b11);
        let address = self.read_memory_following_u16(pc);
        if self.registers.check_condition(condition) {
            debug!("{:#06X}: {:#04X} | CALL {:#06x} ||| (jp)", pc, opcode, address);
            let mut sp = self.registers.sp();
            {
                let mut memory = self.memory.write().unwrap();
                memory.push_u16_stack(pc+3, sp);
            }
            sp = sp -2;
            pc = address;
            self.registers.set_sp(sp);
            self.registers.set_pc(pc);
        } else {
            debug!("{:#06X}: {:#04X} | CALL {:#06x} ||| (skip)", pc, opcode, address);
            self.registers.inc_pc(3);
        }

    }

    /// RET
    /// 11 001 001
    pub fn ret(&mut self, opcode: u8, pc: u16){
        let mut sp = self.registers.sp();
        let pc = {
            let memory = self.memory.read().unwrap();
            memory.pop_u16_stack(sp)
        };
        debug!("{:#06X}: {:#04X} | RET  [{:#06x}]", pc, opcode, pc);
        sp = sp + 2;
        self.registers.set_sp(sp);
        self.registers.set_pc(pc);
    }

    /// RET
    /// 11 001 001
    pub fn reti(&mut self, opcode: u8, pc: u16){
        unimplemented!();
    }

    /// RET     cc
    /// 11 0cc 000
    pub fn ret_cc(&mut self, opcode: u8, pc: u16){
        unimplemented!();
    }

    /// RST     t
    /// 11 ttt 111
    pub fn rst_t(&mut self, opcode: u8, pc: u16){
        unimplemented!();
    }


// ------------------------------------------------------------------ //
// General-Purpose Arithmetic Operations and CPU Control Instructions //
// ------------------------------------------------------------------ //

    /// DAA
    /// 00 100 111
    pub fn daa(&mut self, opcode: u8, pc: u16){
        unimplemented!();
    }

    /// CPL
    /// 00 101 111
    pub fn cpl(&mut self, opcode: u8, pc: u16){
        unimplemented!();
    }

    /// NOP
    /// 00 000 000
    pub fn nop(&mut self, opcode: u8, pc: u16){
        debug!("{:#06X}: {:#04X} | NOP", pc, opcode);
        self.registers.inc_pc(1);
    }

    /// HALT
    /// 01 110 110
    pub fn halt(&mut self, opcode: u8, pc: u16){
        unimplemented!();
    }

    /// STOP
    /// 00 010 000
    /// 00 000 000
    pub fn stop(&mut self, opcode: u8, pc: u16){
        unimplemented!();
    }

    /// EI
    /// 11 111 011
    pub fn ei(&mut self, opcode: u8, pc: u16){
        debug!("{:#06X}: {:#04X} | EI", pc, opcode);
        self.interrupt.write().unwrap().master_enable = true;
        self.registers.inc_pc(1);
    }

    /// DI
    /// 11 110 011
    pub fn di(&mut self, opcode: u8, pc: u16){
        debug!("{:#06X}: {:#04X} | DI", pc, opcode);
        self.interrupt.write().unwrap().master_enable = false;
        self.registers.inc_pc(1);
    }

    // ---------------- //
    // Helper Functions //
    // ---------------- //

    fn read_memory(&self, address: u16) -> u8 {
        let memory = self.memory.read().unwrap();
        memory.read(address)
    }

    fn write_memory(&mut self, address: u16, value: u8){
        let mut memory = self.memory.write().unwrap();
        memory.write(address, value)
    }

    fn read_memory_following_u8(&self, address: u16) -> u8 {
        let memory = self.memory.read().unwrap();
        memory.following_u8(address)
    }

    fn read_memory_following_u16(&self, address: u16) -> u16 {
        let memory = self.memory.read().unwrap();
        memory.following_u16(address)
    }

    fn add_signed(a: u16, b: u8) -> u16 {
        (a as i16 + (b as i8 as i16)) as u16
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

    #[test]
    fn ld_a_b() {
        let rom = ROM::new(vec![
            0b00_111_110,
            0b00_000_001,   // LD A, 1
            0b00_000_110,
            0b00_000_010,   // LD B, 2
            0b01_111_000    // LD A, B
        ]);
        let (mut cpu, memory) = create_cpu(rom);
        for i in 0..3{
            cpu.step();
        }
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
        let (mut cpu, memory) = create_cpu(rom);
        for i in 0..3{
            cpu.step();
        }
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
        let (mut cpu, memory) = create_cpu(rom);
        for i in 0..1{
            cpu.step();
        }
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
        let (mut cpu, memory) = create_cpu(rom);
        for i in 0..4{
            cpu.step();
        }
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
            0b01_100_111, // LD (HL), A
        ]);
        let (mut cpu, memory) = create_cpu(rom);
        for i in 0..3{
            cpu.step();
        }
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
        for i in 0..2{
            cpu.step();
        }
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
        let (mut cpu, memory) = create_cpu(rom);
        for i in 0..4{
            cpu.step();
        }
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
        let (mut cpu, memory) = create_cpu(rom);
        for i in 0..4{
            cpu.step();
        }
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
        let (mut cpu, memory) = create_cpu(rom);
        for i in 0..4{
            cpu.step();
        }
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
        for i in 0..3{
            cpu.step();
        }
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
        let (mut cpu, memory) = create_cpu(rom);
        for i in 0..5{
            cpu.step();
        }
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
        for i in 0..2{
            cpu.step();
        }
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
        let (mut cpu, memory) = create_cpu(rom);
        for i in 0..3{
            cpu.step();
        }
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
        for i in 0..2{
            cpu.step();
        }
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
        let (mut cpu, memory) = create_cpu(rom);
        for i in 0..3{
            cpu.step();
        }
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
        let (mut cpu, memory) = create_cpu(rom);
        for i in 0..3{
            cpu.step();
        }
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
        for i in 0..3{
            cpu.step();
        }
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
        for i in 0..3{
            cpu.step();
        }
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
        for i in 0..3{
            cpu.step();
        }
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
        for i in 0..3{
            cpu.step();
        }
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
        let (mut cpu, memory) = create_cpu(rom);
        for i in 0..1{
            cpu.step();
        }
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
        let (mut cpu, memory) = create_cpu(rom);
        for i in 0..2{
            cpu.step();
        }
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

        for i in 0..2{
            cpu.step();
        }
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
        let (mut cpu, memory) = create_cpu(rom);
        for i in 0..2{
            cpu.step();
        }
        let old_sp = {&cpu.registers.sp()};
        for i in 0..1{
            cpu.step();
        }
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
        let (mut cpu, memory) = create_cpu(rom);
        for i in 0..2{
            cpu.step();
        }
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
        let (mut cpu, memory) = create_cpu(rom);
        for i in 0..2{
            cpu.step();
        }
        let registers = &cpu.registers;
        assert_eq!(registers.get_flag_z(), 0);
        assert_eq!(registers.get_flag_h(), 0);
        assert_eq!(registers.get_flag_n(), 0);
        assert_eq!(registers.get_flag_cy(), 0);
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
        for i in 0..2{
            cpu.step();
        }
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
        let (mut cpu, memory) = create_cpu(rom);
        for i in 0..3{
            cpu.step();
        }
        let registers = &cpu.registers;
        assert_eq!(registers.a(), 0x0);
        assert_eq!(registers.get_flag_z(), 1);
        assert_eq!(registers.get_flag_h(), 1);
        assert_eq!(registers.get_flag_n(), 0);
        assert_eq!(registers.get_flag_cy(), 1);
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
        let (mut cpu, memory) = create_cpu(rom);
        for i in 0..2{
            cpu.step();
        }
        let registers = &cpu.registers;
        assert_eq!(registers.a(), 0x3B);
        assert_eq!(registers.get_flag_z(), 0);
        assert_eq!(registers.get_flag_h(), 1);
        assert_eq!(registers.get_flag_n(), 0);
        assert_eq!(registers.get_flag_cy(), 1);
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
        let (mut cpu, memory) = create_cpu(rom);
        for i in 0..4{
            cpu.step();
        }
        let registers = &cpu.registers;
        assert_eq!(registers.a(), 0x4E);
        assert_eq!(registers.get_flag_z(), 0);
        assert_eq!(registers.get_flag_h(), 0);
        assert_eq!(registers.get_flag_n(), 0);
        assert_eq!(registers.get_flag_cy(), 0);
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
        let (mut cpu, memory) = create_cpu(rom);
        for i in 0..5{
            cpu.step();
        }
        let registers = &cpu.registers;
        assert_eq!(registers.a(), 0xF1);
        assert_eq!(registers.get_flag_z(), 0);
        assert_eq!(registers.get_flag_h(), 1);
        assert_eq!(registers.get_flag_n(), 0);
        assert_eq!(registers.get_flag_cy(), 0);
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
        let (mut cpu, memory) = create_cpu(rom);
        for i in 0..4{
            cpu.step();
        }
        let registers = &cpu.registers;
        assert_eq!(registers.a(), 0x1D);
        assert_eq!(registers.get_flag_z(), 0);
        assert_eq!(registers.get_flag_h(), 0);
        assert_eq!(registers.get_flag_n(), 0);
        assert_eq!(registers.get_flag_cy(), 1);
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
        let (mut cpu, memory) = create_cpu(rom);
        for i in 0..6{
            cpu.step();
        }
        let registers = &cpu.registers;
        assert_eq!(registers.a(), 0x0);
        assert_eq!(registers.get_flag_z(), 1);
        assert_eq!(registers.get_flag_h(), 1);
        assert_eq!(registers.get_flag_n(), 0);
        assert_eq!(registers.get_flag_cy(), 1);
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
        let (mut cpu, memory) = create_cpu(rom);
        for i in 0..3{
            cpu.step();
        }
        let registers = &cpu.registers;
        assert_eq!(registers.a(), 0x0);
        assert_eq!(registers.get_flag_z(), 1);
        assert_eq!(registers.get_flag_h(), 0);
        assert_eq!(registers.get_flag_n(), 1);
        assert_eq!(registers.get_flag_cy(), 0);
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
        let (mut cpu, memory) = create_cpu(rom);
        for i in 0..2{
            cpu.step();
        }
        let registers = &cpu.registers;
        assert_eq!(registers.a(), 0x2F);
        assert_eq!(registers.get_flag_z(), 0);
        assert_eq!(registers.get_flag_h(), 1);
        assert_eq!(registers.get_flag_n(), 1);
        assert_eq!(registers.get_flag_cy(), 0);
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
        let (mut cpu, memory) = create_cpu(rom);
        for i in 0..4{
            cpu.step();
        }
        let registers = &cpu.registers;
        assert_eq!(registers.a(), 0xFE);
        assert_eq!(registers.get_flag_z(), 0);
        assert_eq!(registers.get_flag_h(), 0);
        assert_eq!(registers.get_flag_n(), 1);
        assert_eq!(registers.get_flag_cy(), 1);
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
        let (mut cpu, memory) = create_cpu(rom);
        for i in 0..5{
            cpu.step();
        }
        let registers = &cpu.registers;
        assert_eq!(registers.a(), 0x10);
        assert_eq!(registers.get_flag_z(), 0);
        assert_eq!(registers.get_flag_h(), 0);
        assert_eq!(registers.get_flag_n(), 1);
        assert_eq!(registers.get_flag_cy(), 0);
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
        let (mut cpu, memory) = create_cpu(rom);
        for i in 0..4{
            cpu.step();
        }
        let registers = &cpu.registers;
        assert_eq!(registers.a(), 0x00);
        assert_eq!(registers.get_flag_z(), 1);
        assert_eq!(registers.get_flag_h(), 0);
        assert_eq!(registers.get_flag_n(), 1);
        assert_eq!(registers.get_flag_cy(), 0);
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
        let (mut cpu, memory) = create_cpu(rom);
        for i in 0..6{
            cpu.step();
        }
        let registers = &cpu.registers;
        assert_eq!(registers.a(), 0xEB);
        assert_eq!(registers.get_flag_z(), 0);
        assert_eq!(registers.get_flag_h(), 1);
        assert_eq!(registers.get_flag_n(), 1);
        assert_eq!(registers.get_flag_cy(), 1);
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
        let (mut cpu, memory) = create_cpu(rom);
        for i in 0..3{
            cpu.step();
        }
        let registers = &cpu.registers;
        assert_eq!(registers.a(), 0x1A);
        assert_eq!(registers.get_flag_z(), 0);
        assert_eq!(registers.get_flag_h(), 1);
        assert_eq!(registers.get_flag_n(), 0);
        assert_eq!(registers.get_flag_cy(), 0);
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
        let (mut cpu, memory) = create_cpu(rom);
        for i in 0..2{
            cpu.step();
        }
        let registers = &cpu.registers;
        assert_eq!(registers.a(), 0x18);
        assert_eq!(registers.get_flag_z(), 0);
        assert_eq!(registers.get_flag_h(), 1);
        assert_eq!(registers.get_flag_n(), 0);
        assert_eq!(registers.get_flag_cy(), 0);
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
        let (mut cpu, memory) = create_cpu(rom);
        for i in 0..4{
            cpu.step();
        }
        let registers = &cpu.registers;
        assert_eq!(registers.a(), 0x00);
        assert_eq!(registers.get_flag_z(), 1);
        assert_eq!(registers.get_flag_h(), 1);
        assert_eq!(registers.get_flag_n(), 0);
        assert_eq!(registers.get_flag_cy(), 0);
        assert_eq!(registers.pc(), 8);
    }

    #[test]
    fn or_a_l() {
        let rom = ROM::new(vec![
            0b00_111_110,
            0x5A,           // LD A, 0x5A
            0b10_110_111    // OR A, A
        ]);
        let (mut cpu, memory) = create_cpu(rom);
        for i in 0..2{
            cpu.step();
        }
        let registers = &cpu.registers;
        assert_eq!(registers.a(), 0x5A);
        assert_eq!(registers.get_flag_z(), 0);
        assert_eq!(registers.get_flag_h(), 0);
        assert_eq!(registers.get_flag_n(), 0);
        assert_eq!(registers.get_flag_cy(), 0);
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
        let (mut cpu, memory) = create_cpu(rom);
        for i in 0..2{
            cpu.step();
        }
        let registers = &cpu.registers;
        assert_eq!(registers.a(), 0x5B);
        assert_eq!(registers.get_flag_z(), 0);
        assert_eq!(registers.get_flag_h(), 0);
        assert_eq!(registers.get_flag_n(), 0);
        assert_eq!(registers.get_flag_cy(), 0);
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
        let (mut cpu, memory) = create_cpu(rom);
        for i in 0..4{
            cpu.step();
        }
        let registers = &cpu.registers;
        assert_eq!(registers.a(), 0x5A);
        assert_eq!(registers.get_flag_z(), 0);
        assert_eq!(registers.get_flag_h(), 0);
        assert_eq!(registers.get_flag_n(), 0);
        assert_eq!(registers.get_flag_cy(), 0);
        assert_eq!(registers.pc(), 8);
    }

    #[test]
    fn xor_a_l() {
        let rom = ROM::new(vec![
            0b00_111_110,
            0xFF,           // LD A, 0xFF
            0b10_101_111    // XOR A, A
        ]);
        let (mut cpu, memory) = create_cpu(rom);
        for i in 0..2{
            cpu.step();
        }
        let registers = &cpu.registers;
        assert_eq!(registers.a(), 0x00);
        assert_eq!(registers.get_flag_z(), 1);
        assert_eq!(registers.get_flag_h(), 0);
        assert_eq!(registers.get_flag_n(), 0);
        assert_eq!(registers.get_flag_cy(), 0);
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
        let (mut cpu, memory) = create_cpu(rom);
        for i in 0..2{
            cpu.step();
        }
        let registers = &cpu.registers;
        assert_eq!(registers.a(), 0xF0);
        assert_eq!(registers.get_flag_z(), 0);
        assert_eq!(registers.get_flag_h(), 0);
        assert_eq!(registers.get_flag_n(), 0);
        assert_eq!(registers.get_flag_cy(), 0);
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
        let (mut cpu, memory) = create_cpu(rom);
        for i in 0..4{
            cpu.step();
        }
        let registers = &cpu.registers;
        assert_eq!(registers.a(), 0x75);
        assert_eq!(registers.get_flag_z(), 0);
        assert_eq!(registers.get_flag_h(), 0);
        assert_eq!(registers.get_flag_n(), 0);
        assert_eq!(registers.get_flag_cy(), 0);
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
        let (mut cpu, memory) = create_cpu(rom);
        for i in 0..3{
            cpu.step();
        }
        let registers = &cpu.registers;
        assert_eq!(registers.a(), 0x3C);
        assert_eq!(registers.get_flag_z(), 0);
        assert_eq!(registers.get_flag_h(), 1);
        assert_eq!(registers.get_flag_n(), 1);
        assert_eq!(registers.get_flag_cy(), 0);
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
        let (mut cpu, memory) = create_cpu(rom);
        for i in 0..2{
            cpu.step();
        }
        let registers = &cpu.registers;
        assert_eq!(registers.a(), 0x3C);
        assert_eq!(registers.get_flag_z(), 1);
        assert_eq!(registers.get_flag_h(), 0);
        assert_eq!(registers.get_flag_n(), 1);
        assert_eq!(registers.get_flag_cy(), 0);
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
        let (mut cpu, memory) = create_cpu(rom);
        for i in 0..4{
            cpu.step();
        }
        let registers = &cpu.registers;
        assert_eq!(registers.a(), 0x3C);
        assert_eq!(registers.get_flag_z(), 0);
        assert_eq!(registers.get_flag_h(), 0);
        assert_eq!(registers.get_flag_n(), 1);
        assert_eq!(registers.get_flag_cy(), 1);
        assert_eq!(registers.pc(), 8);
    }

    #[test]
    fn inc_r() {
        let rom = ROM::new(vec![
            0b00_111_110,
            0xFF,           // LD A, 0xFF
            0b00_111_100    // INC A
        ]);
        let (mut cpu, memory) = create_cpu(rom);
        let cy = cpu.registers.get_flag_cy();
        for i in 0..2{
            cpu.step();
        }
        let registers = &cpu.registers;
        assert_eq!(registers.a(), 0x00);
        assert_eq!(registers.get_flag_z(), 1);
        assert_eq!(registers.get_flag_h(), 1);
        assert_eq!(registers.get_flag_n(), 0);
        assert_eq!(registers.get_flag_cy(), cy);
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
        let cy = cpu.registers.get_flag_cy();
        for i in 0..3 {
            cpu.step();
        }
        let registers = &cpu.registers;
        assert_eq!(memory.read().unwrap().read(0x8000), 0x51);
        assert_eq!(registers.get_flag_z(), 0);
        assert_eq!(registers.get_flag_h(), 0);
        assert_eq!(registers.get_flag_n(), 0);
        assert_eq!(registers.get_flag_cy(), cy);
        assert_eq!(registers.pc(), 6);
    }

    #[test]
    fn dec_r() {
        let rom = ROM::new(vec![
            0b00_101_110,
            0x01,           // LD L, 0x01
            0b00_101_101    // DEC L
        ]);
        let (mut cpu, memory) = create_cpu(rom);
        let cy = cpu.registers.get_flag_cy();
        for i in 0..2{
            cpu.step();
        }
        let registers = &cpu.registers;
        assert_eq!(registers.l(), 0x00);
        assert_eq!(registers.get_flag_z(), 1);
        assert_eq!(registers.get_flag_h(), 0);
        assert_eq!(registers.get_flag_n(), 1);
        assert_eq!(registers.get_flag_cy(), cy);
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
        let cy = cpu.registers.get_flag_cy();
        for i in 0..3 {
            cpu.step();
        }
        let registers = &cpu.registers;
        assert_eq!(memory.read().unwrap().read(0x8000), 0xFF);
        assert_eq!(registers.get_flag_z(), 0);
        assert_eq!(registers.get_flag_h(), 1);
        assert_eq!(registers.get_flag_n(), 1);
        assert_eq!(registers.get_flag_cy(), cy);
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
        let (mut cpu, memory) = create_cpu(rom);
        for i in 0..3{
            cpu.step();
        }
        let registers = &cpu.registers;
        assert_eq!(registers.hl(), 0x9028);
        assert_eq!(registers.get_flag_h(), 1);
        assert_eq!(registers.get_flag_n(), 0);
        assert_eq!(registers.get_flag_cy(), 0);
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
        let (mut cpu, memory) = create_cpu(rom);
        let z = cpu.registers.get_flag_z();
        for i in 0..2{
            cpu.step();
        }
        let registers = &cpu.registers;
        assert_eq!(registers.hl(), 0x1446);
        assert_eq!(registers.get_flag_z(), z);
        assert_eq!(registers.get_flag_h(), 1);
        assert_eq!(registers.get_flag_n(), 0);
        assert_eq!(registers.get_flag_cy(), 1);
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
        let (mut cpu, memory) = create_cpu(rom);
        for i in 0..2{
            cpu.step();
        }
        let registers = &cpu.registers;
        assert_eq!(registers.sp(), 0xFFFA);
        assert_eq!(registers.get_flag_z(), 0);
        assert_eq!(registers.get_flag_h(), 0);
        assert_eq!(registers.get_flag_n(), 0);
        assert_eq!(registers.get_flag_cy(), 0);
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
        let (mut cpu, memory) = create_cpu(rom);
        let z = cpu.registers.get_flag_z();
        let h = cpu.registers.get_flag_h();
        let n = cpu.registers.get_flag_n();
        let cy = cpu.registers.get_flag_cy();
        for i in 0..2{
            cpu.step();
        }
        let registers = &cpu.registers;
        assert_eq!(registers.de(), 0x2360);
        assert_eq!(registers.get_flag_z(), z);
        assert_eq!(registers.get_flag_h(), h);
        assert_eq!(registers.get_flag_n(), n);
        assert_eq!(registers.get_flag_cy(), cy);
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
        let (mut cpu, memory) = create_cpu(rom);
        let z = cpu.registers.get_flag_z();
        let h = cpu.registers.get_flag_h();
        let n = cpu.registers.get_flag_n();
        let cy = cpu.registers.get_flag_cy();
        for i in 0..2{
            cpu.step();
        }
        let registers = &cpu.registers;
        assert_eq!(registers.de(), 0x235E);
        assert_eq!(registers.get_flag_z(), z);
        assert_eq!(registers.get_flag_h(), h);
        assert_eq!(registers.get_flag_n(), n);
        assert_eq!(registers.get_flag_cy(), cy);
        assert_eq!(registers.pc(), 4);
    }

    #[test]
    fn rlca() {
        let rom = ROM::new(vec![
            0b00_111_110,
            0x85,           // LD A, 0x85
            0b00_000_111    // RLCA

        ]);
        let (mut cpu, memory) = create_cpu(rom);
        for i in 0..2{
            cpu.step();
        }
        let registers = &cpu.registers;
        assert_eq!(registers.a(), 0x0B);
        assert_eq!(registers.get_flag_z(), 0);
        assert_eq!(registers.get_flag_h(), 0);
        assert_eq!(registers.get_flag_n(), 0);
        assert_eq!(registers.get_flag_cy(), 1);
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
        let (mut cpu, memory) = create_cpu(rom);
        for i in 0..4{
            cpu.step();
        }
        let registers = &cpu.registers;
        assert_eq!(registers.a(), 0x2B);
        assert_eq!(registers.get_flag_z(), 0);
        assert_eq!(registers.get_flag_h(), 0);
        assert_eq!(registers.get_flag_n(), 0);
        assert_eq!(registers.get_flag_cy(), 1);
        assert_eq!(registers.pc(), 7);
    }

    #[test]
    fn rrca() {
        let rom = ROM::new(vec![
            0b00_111_110,
            0x3b,           // LD A, 0x3b
            0b00_001_111    // RRCA

        ]);
        let (mut cpu, memory) = create_cpu(rom);
        for i in 0..2{
            cpu.step();
        }
        let registers = &cpu.registers;
        assert_eq!(registers.a(), 0x9D);
        assert_eq!(registers.get_flag_z(), 0);
        assert_eq!(registers.get_flag_h(), 0);
        assert_eq!(registers.get_flag_n(), 0);
        assert_eq!(registers.get_flag_cy(), 1);
        assert_eq!(registers.pc(), 3);
    }

    #[test]
    fn rra() {
        let rom = ROM::new(vec![
            0b00_111_110,
            0x81,           // LD A, 0x81
            0b00_011_111    // RRA

        ]);
        let (mut cpu, memory) = create_cpu(rom);
        for i in 0..2{
            cpu.step();
        }
        let registers = &cpu.registers;
        assert_eq!(registers.a(), 0x40);
        assert_eq!(registers.get_flag_z(), 0);
        assert_eq!(registers.get_flag_h(), 0);
        assert_eq!(registers.get_flag_n(), 0);
        assert_eq!(registers.get_flag_cy(), 1);
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
        let (mut cpu, memory) = create_cpu(rom);
        for i in 0..2{
            cpu.step();
        }
        let registers = &cpu.registers;
        assert_eq!(registers.b(), 0x0B);
        assert_eq!(registers.get_flag_z(), 0);
        assert_eq!(registers.get_flag_h(), 0);
        assert_eq!(registers.get_flag_n(), 0);
        assert_eq!(registers.get_flag_cy(), 1);
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
        for i in 0..3{
            cpu.step();
        }
        let registers = &cpu.registers;
        assert_eq!(memory.read().unwrap().read(0x8000), 0);
        assert_eq!(registers.get_flag_z(), 1);
        assert_eq!(registers.get_flag_h(), 0);
        assert_eq!(registers.get_flag_n(), 0);
        assert_eq!(registers.get_flag_cy(), 0);
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
        let (mut cpu, memory) = create_cpu(rom);
        for i in 0..2{
            cpu.step();
        }
        let registers = &cpu.registers;
        assert_eq!(registers.l(), 0x00);
        assert_eq!(registers.get_flag_z(), 1);
        assert_eq!(registers.get_flag_h(), 0);
        assert_eq!(registers.get_flag_n(), 0);
        assert_eq!(registers.get_flag_cy(), 1);
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
        for i in 0..3{
            cpu.step();
        }
        let registers = &cpu.registers;
        assert_eq!(memory.read().unwrap().read(0x8000), 0x22);
        assert_eq!(registers.get_flag_z(), 0);
        assert_eq!(registers.get_flag_h(), 0);
        assert_eq!(registers.get_flag_n(), 0);
        assert_eq!(registers.get_flag_cy(), 0);
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
        let (mut cpu, memory) = create_cpu(rom);
        for i in 0..2{
            cpu.step();
        }
        let registers = &cpu.registers;
        assert_eq!(registers.c(), 0x80);
        assert_eq!(registers.get_flag_z(), 0);
        assert_eq!(registers.get_flag_h(), 0);
        assert_eq!(registers.get_flag_n(), 0);
        assert_eq!(registers.get_flag_cy(), 1);
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
        for i in 0..3{
            cpu.step();
        }
        let registers = &cpu.registers;
        assert_eq!(memory.read().unwrap().read(0x8000), 0);
        assert_eq!(registers.get_flag_z(), 1);
        assert_eq!(registers.get_flag_h(), 0);
        assert_eq!(registers.get_flag_n(), 0);
        assert_eq!(registers.get_flag_cy(), 0);
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
        let (mut cpu, memory) = create_cpu(rom);
        for i in 0..2{
            cpu.step();
        }
        let registers = &cpu.registers;
        assert_eq!(registers.a(), 0x00);
        assert_eq!(registers.get_flag_z(), 1);
        assert_eq!(registers.get_flag_h(), 0);
        assert_eq!(registers.get_flag_n(), 0);
        assert_eq!(registers.get_flag_cy(), 1);
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
        for i in 0..3{
            cpu.step();
        }
        let registers = &cpu.registers;
        assert_eq!(memory.read().unwrap().read(0x8000), 0x45);
        assert_eq!(registers.get_flag_z(), 0);
        assert_eq!(registers.get_flag_h(), 0);
        assert_eq!(registers.get_flag_n(), 0);
        assert_eq!(registers.get_flag_cy(), 0);
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
        let (mut cpu, memory) = create_cpu(rom);
        for i in 0..2{
            cpu.step();
        }
        let registers = &cpu.registers;
        assert_eq!(registers.d(), 0x00);
        assert_eq!(registers.get_flag_z(), 1);
        assert_eq!(registers.get_flag_h(), 0);
        assert_eq!(registers.get_flag_n(), 0);
        assert_eq!(registers.get_flag_cy(), 1);
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
        for i in 0..3{
            cpu.step();
        }
        let registers = &cpu.registers;
        assert_eq!(memory.read().unwrap().read(0x8000), 0xFE);
        assert_eq!(registers.get_flag_z(), 0);
        assert_eq!(registers.get_flag_h(), 0);
        assert_eq!(registers.get_flag_n(), 0);
        assert_eq!(registers.get_flag_cy(), 1);
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
    let (mut cpu, memory) = create_cpu(rom);
    for i in 0..2{
        cpu.step();
    }
    let registers = &cpu.registers;
    assert_eq!(registers.a(), 0xC5);
    assert_eq!(registers.get_flag_z(), 0);
    assert_eq!(registers.get_flag_h(), 0);
    assert_eq!(registers.get_flag_n(), 0);
    assert_eq!(registers.get_flag_cy(), 0);
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
        for i in 0..3{
            cpu.step();
        }
        let registers = &cpu.registers;
        assert_eq!(memory.read().unwrap().read(0x8000), 0x00);
        assert_eq!(registers.get_flag_z(), 1);
        assert_eq!(registers.get_flag_h(), 0);
        assert_eq!(registers.get_flag_n(), 0);
        assert_eq!(registers.get_flag_cy(), 1);
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
        let (mut cpu, memory) = create_cpu(rom);
        for i in 0..2{
            cpu.step();
        }
        let registers = &cpu.registers;
        assert_eq!(registers.a(), 0x00);
        assert_eq!(registers.get_flag_z(), 1);
        assert_eq!(registers.get_flag_h(), 0);
        assert_eq!(registers.get_flag_n(), 0);
        assert_eq!(registers.get_flag_cy(), 1);
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
        for i in 0..3{
            cpu.step();
        }
        let registers = &cpu.registers;
        assert_eq!(memory.read().unwrap().read(0x8000), 0x7F);
        assert_eq!(registers.get_flag_z(), 0);
        assert_eq!(registers.get_flag_h(), 0);
        assert_eq!(registers.get_flag_n(), 0);
        assert_eq!(registers.get_flag_cy(), 1);
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
        let (mut cpu, memory) = create_cpu(rom);
        for i in 0..2{
            cpu.step();
        }
        let registers = &cpu.registers;
        assert_eq!(registers.a(), 0x00);
        assert_eq!(registers.get_flag_z(), 1);
        assert_eq!(registers.get_flag_h(), 0);
        assert_eq!(registers.get_flag_n(), 0);
        assert_eq!(registers.get_flag_cy(), 0);
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
        for i in 0..3{
            cpu.step();
        }
        let registers = &cpu.registers;
        assert_eq!(memory.read().unwrap().read(0x8000), 0x0F);
        assert_eq!(registers.get_flag_z(), 0);
        assert_eq!(registers.get_flag_h(), 0);
        assert_eq!(registers.get_flag_n(), 0);
        assert_eq!(registers.get_flag_cy(), 0);
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
        let (mut cpu, memory) = create_cpu(rom);
        for i in 0..2{
            cpu.step();
        }
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
        let (mut cpu, memory) = create_cpu(rom);
        for i in 0..2{
            cpu.step();
        }
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
        let (mut cpu, memory) = create_cpu(rom);
        for i in 0..3{
            cpu.step();
        }
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
        let (mut cpu, memory) = create_cpu(rom);
        for i in 0..3{
            cpu.step();
        }
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
        let (mut cpu, memory) = create_cpu(rom);
        for i in 0..2{
            cpu.step();
        }
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
        let (mut cpu, memory) = create_cpu(rom);
        for i in 0..2{
            cpu.step();
        }
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
        for i in 0..3{
            cpu.step();
        }
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
        let (mut cpu, memory) = create_cpu(rom);
        for i in 0..2{
            cpu.step();
        }
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
        let (mut cpu, memory) = create_cpu(rom);
        for i in 0..2{
            cpu.step();
        }
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
        for i in 0..3{
            cpu.step();
        }
        let registers = &cpu.registers;
        assert_eq!(memory.read().unwrap().read(0x8000), 0xF7);
        assert_eq!(registers.pc(), 7);
    }

}