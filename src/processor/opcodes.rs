use super::{
    cpu::CPU,
    registers::{
        Condition,
        FlagCalculationStatus::*,
        FlagCalculationsBuilder,
        RegisterDD,
        RegisterQQ,
        RegisterR,
        RegisterSS,
        Registers,
    },
};
use crate::util::{
    bit_op,
    memory_op,
};

#[rustfmt::skip] const OPCODE_TABLE: [fn(u8, u16, &mut CPU) -> u8; 0x100] = [
    /*    x0       x1       x2        x3       x4        x5       x6         x7       x8        x9        xA        xB       xC        xD       xE         xF          */
    /*0x*/nop,     ld_dd_nn,ld_mbc_a, inc_ss,  inc_r,    dec_r,   ld_r_n,    rlca,    ld_mnn_sp,add_hl_ss,ld_a_mbc, dec_ss,  inc_r,    dec_r,   ld_r_n,    rrca,   /*0x*/
    /*1x*/stop,    ld_dd_nn,ld_mde_a, inc_ss,  inc_r,    dec_r,   ld_r_n,    rla,     jr_e,     add_hl_ss,ld_a_mde, dec_ss,  inc_r,    dec_r,   ld_r_n,    rra,    /*1x*/
    /*2x*/jr_cc_e, ld_dd_nn,ld_mhli_a,inc_ss,  inc_r,    dec_r,   ld_r_n,    daa,     jr_cc_e,  add_hl_ss,ld_a_mhli,dec_ss,  inc_r,    dec_r,   ld_r_n,    cpl,    /*2x*/
    /*3x*/jr_cc_e, ld_dd_nn,ld_mhld_a,inc_ss,  inc_mhl,  dec_mhl, ld_mhl_n,  scf,     jr_cc_e,  add_hl_ss,ld_a_mhld,dec_ss,  inc_r,    dec_r,   ld_r_n,    ccf,    /*3x*/
    /*4x*/ld_r_r,  ld_r_r,  ld_r_r,   ld_r_r,  ld_r_r,   ld_r_r,  ld_r_mhl,  ld_r_r,  ld_r_r,   ld_r_r,   ld_r_r,   ld_r_r,  ld_r_r,   ld_r_r,  ld_r_mhl,  ld_r_r, /*4x*/
    /*5x*/ld_r_r,  ld_r_r,  ld_r_r,   ld_r_r,  ld_r_r,   ld_r_r,  ld_r_mhl,  ld_r_r,  ld_r_r,   ld_r_r,   ld_r_r,   ld_r_r,  ld_r_r,   ld_r_r,  ld_r_mhl,  ld_r_r, /*5x*/
    /*6x*/ld_r_r,  ld_r_r,  ld_r_r,   ld_r_r,  ld_r_r,   ld_r_r,  ld_r_mhl,  ld_r_r,  ld_r_r,   ld_r_r,   ld_r_r,   ld_r_r,  ld_r_r,   ld_r_r,  ld_r_mhl,  ld_r_r, /*6x*/
    /*7x*/ld_mhl_r,ld_mhl_r,ld_mhl_r, ld_mhl_r,ld_mhl_r, ld_mhl_r,halt,      ld_mhl_r,ld_r_r,   ld_r_r,   ld_r_r,   ld_r_r,  ld_r_r,   ld_r_r,  ld_r_mhl,  ld_r_r, /*7x*/
    /*8x*/add_a_r, add_a_r, add_a_r,  add_a_r, add_a_r,  add_a_r, add_a_mhl, add_a_r, adc_a_r,  adc_a_r,  adc_a_r,  adc_a_r, adc_a_r,  adc_a_r, adc_a_mhl, adc_a_r,/*8x*/
    /*9x*/sub_a_r, sub_a_r, sub_a_r,  sub_a_r, sub_a_r,  sub_a_r, sub_a_mhl, sub_a_r, sbc_a_r,  sbc_a_r,  sbc_a_r,  sbc_a_r, sbc_a_r,  sbc_a_r, sbc_a_mhl, sbc_a_r,/*9x*/
    /*Ax*/and_r,   and_r,   and_r,    and_r,   and_r,    and_r,   and_mhl,   and_r,   xor_r,    xor_r,    xor_r,    xor_r,   xor_r,    xor_r,   xor_mhl,   xor_r,  /*Ax*/
    /*Bx*/or_r,    or_r,    or_r,     or_r,    or_r,     or_r,    or_mhl,    or_r,    cp_r,     cp_r,     cp_r,     cp_r,    cp_r,     cp_r,    cp_mhl,    cp_r,   /*Bx*/
    /*Cx*/ret_c,   pop_qq,  jp_cc_nn, jp_nn,   call_c_nn,push_qq, add_a_n,   rst_t,   ret_c,    ret,      jp_cc_nn, extended,call_c_nn,call_nn, adc_a_n,   rst_t,  /*Cx*/
    /*Dx*/ret_c,   pop_qq,  jp_cc_nn, unsupp,  call_c_nn,push_qq, sub_a_n,   rst_t,   ret_c,    reti,     jp_cc_nn, unsupp,  call_c_nn,unsupp,  sbc_a_n,   rst_t,  /*Dx*/
    /*Ex*/ld_mn_a, pop_qq,  ld_mc_a,  unsupp,  unsupp,   push_qq, and_n,     rst_t,   add_sp_e, jp_mhl,   ld_mnn_a, unsupp,  unsupp,   unsupp,  xor_n,     rst_t,  /*Ex*/
    /*Fx*/ld_a_mn, pop_qq,  ld_a_mc,  di,      unsupp,   push_qq, or_n,      rst_t,   ldhl_sp_e,ld_sp_hl, ld_a_mnn, ei,      unsupp,   unsupp,  cp_n,      rst_t   /*Fx*/
]; // x0       x1       x2        x3       x4       x5        x6         x7
   // x8        x9        xA        xB       xC        xD       xE         xF
#[rustfmt::skip] const OPCODE_EXT_TABLE: [fn(u8, u16, &mut CPU) -> u8; 0x100] = [
    /*    x0       x1       x2        x3       x4        x5       x6         x7       x8        x9        xA        xB       xC        xD       xE         xF          */
    /*0x*/rlc_r,    rlc_r,  rlc_r,    rlc_r,   rlc_r,    rlc_r,   rlc_mhl,   rlc_r,   rrc_r,    rrc_r,    rrc_r,    rrc_r,   rrc_r,    rrc_r,   rrc_mhl,   rrc_r,  /*0x*/
    /*1x*/rl_r,    rl_r,    rl_r,     rl_r,    rl_r,     rl_r,    rl_mhl,    rl_r,    rr_r,     rr_r,     rr_r,     rr_r,    rr_r,     rr_r,    rr_mhl,    rr_r,   /*1x*/
    /*2x*/sla_r,   sla_r,   sla_r,    sla_r,   sla_r,    sla_r,   sla_mhl,   sla_r,   sra_r,    sra_r,    sra_r,    sra_r,   sra_r,    sra_r,   sra_mhl,   sra_r,  /*2x*/
    /*3x*/swap_r,  swap_r,  swap_r,   swap_r,  swap_r,   swap_r,  swap_mhl,  swap_r,  srl_r,    srl_r,    srl_r,    srl_r,   srl_r,    srl_r,   srl_mhl,   srl_r,  /*3x*/
    /*4x*/bit_b_r, bit_b_r, bit_b_r,  bit_b_r, bit_b_r,  bit_b_r, bit_b_mhl, bit_b_r, bit_b_r,  bit_b_r,  bit_b_r,  bit_b_r, bit_b_r,  bit_b_r, bit_b_mhl, bit_b_r,/*4x*/
    /*5x*/bit_b_r, bit_b_r, bit_b_r,  bit_b_r, bit_b_r,  bit_b_r, bit_b_mhl, bit_b_r, bit_b_r,  bit_b_r,  bit_b_r,  bit_b_r, bit_b_r,  bit_b_r, bit_b_mhl, bit_b_r,/*5x*/
    /*6x*/bit_b_r, bit_b_r, bit_b_r,  bit_b_r, bit_b_r,  bit_b_r, bit_b_mhl, bit_b_r, bit_b_r,  bit_b_r,  bit_b_r,  bit_b_r, bit_b_r,  bit_b_r, bit_b_mhl, bit_b_r,/*6x*/
    /*7x*/bit_b_r, bit_b_r, bit_b_r,  bit_b_r, bit_b_r,  bit_b_r, bit_b_mhl, bit_b_r, bit_b_r,  bit_b_r,  bit_b_r,  bit_b_r, bit_b_r,  bit_b_r, bit_b_mhl, bit_b_r,/*7x*/
    /*8x*/res_b_r, res_b_r, res_b_r,  res_b_r, res_b_r,  res_b_r, res_b_mhl, res_b_r, res_b_r,  res_b_r,  res_b_r,  res_b_r, res_b_r,  res_b_r, res_b_mhl, res_b_r,/*8x*/
    /*9x*/res_b_r, res_b_r, res_b_r,  res_b_r, res_b_r,  res_b_r, res_b_mhl, res_b_r, res_b_r,  res_b_r,  res_b_r,  res_b_r, res_b_r,  res_b_r, res_b_mhl, res_b_r,/*9x*/
    /*Ax*/res_b_r, res_b_r, res_b_r,  res_b_r, res_b_r,  res_b_r, res_b_mhl, res_b_r, res_b_r,  res_b_r,  res_b_r,  res_b_r, res_b_r,  res_b_r, res_b_mhl, res_b_r,/*Ax*/
    /*Bx*/res_b_r, res_b_r, res_b_r,  res_b_r, res_b_r,  res_b_r, res_b_mhl, res_b_r, res_b_r,  res_b_r,  res_b_r,  res_b_r, res_b_r,  res_b_r, res_b_mhl, res_b_r,/*Bx*/
    /*Cx*/set_b_r, set_b_r, set_b_r,  set_b_r, set_b_r,  push_qq, set_b_mhl, set_b_r, set_b_r,  set_b_r,  set_b_r,  set_b_r, set_b_r,  set_b_r, set_b_mhl, set_b_r,/*Cx*/
    /*Dx*/set_b_r, set_b_r, set_b_r,  set_b_r, set_b_r,  push_qq, set_b_mhl, set_b_r, set_b_r,  set_b_r,  set_b_r,  set_b_r, set_b_r,  set_b_r, set_b_mhl, set_b_r,/*Dx*/
    /*Ex*/set_b_r, set_b_r, set_b_r,  set_b_r, set_b_r,  push_qq, set_b_mhl, set_b_r, set_b_r,  set_b_r,  set_b_r,  set_b_r, set_b_r,  set_b_r, set_b_mhl, set_b_r,/*Ex*/
    /*Fx*/set_b_r, set_b_r, set_b_r,  set_b_r, set_b_r,  push_qq, set_b_mhl, set_b_r, set_b_r,  set_b_r,  set_b_r,  set_b_r, set_b_r,  set_b_r, set_b_mhl, set_b_r /*Fx*/
]; // x0       x1       x2        x3       x4        x5       x6         x7
   // x8        x9        xA        xB       xC        xD       xE         xF


pub fn execute(opcode: u8, pc: u16, cpu: &mut CPU) -> u8 {
    OPCODE_TABLE[opcode as usize](opcode, pc, cpu)
}

fn extended(_: u8, pc: u16, cpu: &mut CPU) -> u8 {
    let extended_opcode = memory_op::read_memory_following_u8(cpu, pc);
    OPCODE_EXT_TABLE[extended_opcode as usize](extended_opcode, pc, cpu)
}

fn unsupp(opcode: u8, pc: u16, _: &mut CPU) -> u8 {
    debug!("{:#06X}: {:#04X} | not supported)", pc, opcode);
    panic!("Opcode {:#04x} not supported", opcode);
}

// -------------------------------------------- //
// 8-Bit Transfer and Input/Output Instructions //
// -------------------------------------------- //

/// LD      r, r'
/// 01 rrr rrr'
fn ld_r_r(opcode: u8, pc: u16, cpu: &mut CPU) -> u8 {
    let target = RegisterR::new((opcode >> 3) & 0b111);
    let source = RegisterR::new(opcode & 0b111);
    let value = cpu.registers.read_r(source);
    debug!("{:#06X}: {:#04X} | LD   {:?}, {:?}({:?})", pc, opcode, target, source, value);
    cpu.registers.write_r(target, value);
    cpu.registers.inc_pc(1);
    4
}

/// LD      r, n
/// 00 rrr 110
/// nnnnnnnn
fn ld_r_n(opcode: u8, _: u16, cpu: &mut CPU) -> u8 {
    let pc = cpu.registers.pc();
    let target = RegisterR::new((opcode >> 3) & 0b111);
    let value = memory_op::read_memory_following_u8(cpu, pc);
    debug!("{:#06X}: {:#04X} | LD   {:?}, {:?}", pc, opcode, target, value);
    cpu.registers.write_r(target, value);
    cpu.registers.inc_pc(2);
    8
}

/// LD      r, (HL)
/// 01 rrr 110
fn ld_r_mhl(opcode: u8, pc: u16, cpu: &mut CPU) -> u8 {
    let target = RegisterR::new((opcode >> 3) & 0b111);
    let address = cpu.registers.hl();
    let value = memory_op::read_memory(cpu, address);
    debug!(
        "{:#06X}: {:#04X} | LD   {:?}, {:?}[{:#06X}]({:?})",
        pc,
        opcode,
        target,
        RegisterSS::HL,
        address,
        value
    );
    cpu.registers.write_r(target, value);
    cpu.registers.inc_pc(1);
    8
}

/// LD      (HL), r
/// 01 110 rrr
fn ld_mhl_r(opcode: u8, pc: u16, cpu: &mut CPU) -> u8 {
    let source = RegisterR::new(opcode & 0b111);
    let address = cpu.registers.hl();
    let value = cpu.registers.read_r(source);
    debug!(
        "{:#06X}: {:#04X} | LD   {:?}[{:#06X}], {:?}",
        pc,
        opcode,
        RegisterSS::HL,
        address,
        value
    );
    memory_op::write_memory(cpu, address, value);
    cpu.registers.inc_pc(1);
    8
}

/// LD      (HL), n
/// 00 110 110
/// nnnnnnnn
fn ld_mhl_n(opcode: u8, _: u16, cpu: &mut CPU) -> u8 {
    let pc = cpu.registers.pc();
    let address = cpu.registers.hl();
    let value = memory_op::read_memory_following_u8(cpu, pc);
    debug!(
        "{:#06X}: {:#04X} | LD   {:?}[{:#06X}], {:?}",
        pc,
        opcode,
        RegisterSS::HL,
        address,
        value
    );
    memory_op::write_memory(cpu, address, value);
    cpu.registers.inc_pc(2);
    12
}

/// LD      A, (BC)
/// 00 001 010
fn ld_a_mbc(opcode: u8, pc: u16, cpu: &mut CPU) -> u8 {
    let address = cpu.registers.bc();
    let value = memory_op::read_memory(cpu, address);
    debug!(
        "{:#06X}: {:#04X} | LD   {:?}, {:?}[{:#06X}]({:?})",
        pc,
        opcode,
        RegisterR::A,
        RegisterSS::BC,
        address,
        value
    );
    cpu.registers.set_a(value);
    cpu.registers.inc_pc(1);
    8
}

/// LD      A, (DE)
/// 00 011 010
fn ld_a_mde(opcode: u8, pc: u16, cpu: &mut CPU) -> u8 {
    let address = cpu.registers.de();
    let value = memory_op::read_memory(cpu, address);
    debug!(
        "{:#06X}: {:#04X} | LD   {:?}, {:?}[{:#06X}]({:?})",
        pc,
        opcode,
        RegisterR::A,
        RegisterSS::DE,
        address,
        value
    );
    cpu.registers.set_a(value);
    cpu.registers.inc_pc(1);
    8
}

/// LD      A, (C)
/// 11 110 010
fn ld_a_mc(opcode: u8, pc: u16, cpu: &mut CPU) -> u8 {
    let address = 0xFF00 + u16::from(cpu.registers.c());
    let value = memory_op::read_memory(cpu, address);
    debug!(
        "{:#06X}: {:#04X} | LD   {:?}, {:?}[{:#06X}]({:?})",
        pc,
        opcode,
        RegisterR::A,
        RegisterR::C,
        address,
        value
    );
    cpu.registers.set_a(value);
    cpu.registers.inc_pc(1);
    8
}

/// LD      (C), A
/// 11 100 010
fn ld_mc_a(opcode: u8, pc: u16, cpu: &mut CPU) -> u8 {
    let address = 0xFF00 + u16::from(cpu.registers.c());
    let value = cpu.registers.a();
    debug!(
        "{:#06X}: {:#04X} | LD   {:?}[{:#06X}], {:?}",
        pc,
        opcode,
        RegisterR::C,
        address,
        RegisterR::A
    );
    memory_op::write_memory(cpu, address, value);
    cpu.registers.inc_pc(1);
    8
}

/// LD      A, (n)
/// 11 110 000
/// nnnnnnnn
fn ld_a_mn(opcode: u8, _: u16, cpu: &mut CPU) -> u8 {
    let pc = cpu.registers.pc();
    let address = 0xff00 + u16::from(memory_op::read_memory(cpu, pc + 1));
    let value = memory_op::read_memory(cpu, address);
    debug!(
        "{:#06X}: {:#04X} | LD   {:?}, [{:#06x}]({:?})",
        pc,
        opcode,
        RegisterR::A,
        address,
        value
    );
    cpu.registers.set_a(value);
    cpu.registers.inc_pc(2);
    12
}

/// LD      (n), A
/// 11 100 000
/// nnnnnnnn
fn ld_mn_a(opcode: u8, _: u16, cpu: &mut CPU) -> u8 {
    let pc = cpu.registers.pc();
    let value = cpu.registers.a();
    let address = 0xff00 + u16::from(memory_op::read_memory_following_u8(cpu, pc));

    memory_op::write_memory(cpu, address, value);
    debug!(
        "{:#06X}: {:#04X} | LD   [{:#06X}], {:?}({:?})",
        pc,
        opcode,
        address,
        RegisterR::A,
        value
    );

    cpu.registers.inc_pc(2);
    12
}

/// LD      A, (nn)
/// 11 111 010
/// nnnnnnnn
/// nnnnnnnn
fn ld_a_mnn(opcode: u8, _: u16, cpu: &mut CPU) -> u8 {
    let pc = cpu.registers.pc();
    let address = memory_op::read_memory_following_u16(cpu, pc);
    let value = memory_op::read_memory(cpu, address);
    debug!(
        "{:#06X}: {:#04X} | LD   {:?}, [{:#06x}]({:?})",
        pc,
        opcode,
        RegisterR::A,
        address,
        value
    );

    cpu.registers.set_a(value);
    cpu.registers.inc_pc(3);
    16
}

/// LD      (nn), A
/// 11 101 010
/// nnnnnnnn
/// nnnnnnnn
fn ld_mnn_a(opcode: u8, _: u16, cpu: &mut CPU) -> u8 {
    let pc = cpu.registers.pc();
    let value = cpu.registers.a();
    let address = memory_op::read_memory_following_u16(cpu, pc);
    debug!(
        "{:#06X}: {:#04X} | LD   [{:#06X}], {:?}({:?})",
        pc,
        opcode,
        address,
        RegisterR::A,
        value
    );
    memory_op::write_memory(cpu, address, value);

    cpu.registers.inc_pc(3);
    16
}

/// LD      A, (HLI)
/// 00 101 010
fn ld_a_mhli(opcode: u8, pc: u16, cpu: &mut CPU) -> u8 {
    let address = cpu.registers.hl();
    let value = memory_op::read_memory(cpu, address);
    debug!(
        "{:#06X}: {:#04X} | LD   {:?}, {:?}+[{:#06x}]({:?})",
        pc,
        opcode,
        RegisterR::A,
        RegisterDD::HL,
        address,
        value
    );
    cpu.registers.set_a(value);
    cpu.registers.set_hl(address.wrapping_add(1));
    cpu.registers.inc_pc(1);
    8
}

/// LD      A, (HLD)
/// 00 111 010
fn ld_a_mhld(opcode: u8, pc: u16, cpu: &mut CPU) -> u8 {
    let address = cpu.registers.hl();
    let value = memory_op::read_memory(cpu, address);
    debug!(
        "{:#06X}: {:#04X} | LD   {:?}, {:?}-[{:#06x}]({:?})",
        pc,
        opcode,
        RegisterR::A,
        RegisterDD::HL,
        address,
        value
    );

    cpu.registers.set_a(value);
    cpu.registers.set_hl(address.wrapping_sub(1));
    cpu.registers.inc_pc(1);
    8
}

/// LD      (BC), A
/// 00 010 010
fn ld_mbc_a(opcode: u8, _: u16, cpu: &mut CPU) -> u8 {
    let pc = cpu.registers.pc();
    let value = cpu.registers.a();
    let address = cpu.registers.bc();
    debug!(
        "{:#06X}: {:#04X} | LD   [{:#06X}], {:?}({:?})",
        pc,
        opcode,
        address,
        RegisterR::A,
        value
    );
    memory_op::write_memory(cpu, address, value);
    cpu.registers.inc_pc(1);
    8
}

/// LD      (DE), A
/// 00 010 010
fn ld_mde_a(opcode: u8, _: u16, cpu: &mut CPU) -> u8 {
    let pc = cpu.registers.pc();
    let value = cpu.registers.a();
    let address = cpu.registers.de();
    debug!(
        "{:#06X}: {:#04X} | LD   [{:#06X}], {:?}({:?})",
        pc,
        opcode,
        address,
        RegisterR::A,
        value
    );
    memory_op::write_memory(cpu, address, value);
    cpu.registers.inc_pc(1);
    8
}

/// LD      (HLI), A
/// 00 100 010
fn ld_mhli_a(opcode: u8, pc: u16, cpu: &mut CPU) -> u8 {
    let value = cpu.registers.a();
    let address = cpu.registers.hl();
    debug!(
        "{:#06X}: {:#04X} | LD   {:?}+[{:#06X}], {:?}({:?})",
        pc,
        opcode,
        RegisterQQ::HL,
        address,
        RegisterR::A,
        value
    );
    memory_op::write_memory(cpu, address, value);
    cpu.registers.set_hl(address + 1);
    cpu.registers.inc_pc(1);
    8
}

/// LD      (HLD), A
/// 00 110 010
fn ld_mhld_a(opcode: u8, pc: u16, cpu: &mut CPU) -> u8 {
    let value = cpu.registers.a();
    let address = cpu.registers.hl();
    debug!(
        "{:#06X}: {:#04X} | LD   {:?}-[{:#06X}], {:?}({:?})",
        pc,
        opcode,
        RegisterQQ::HL,
        address,
        RegisterR::A,
        value
    );
    memory_op::write_memory(cpu, address, value);
    cpu.registers.set_hl(address - 1);
    cpu.registers.inc_pc(1);
    8
}

// ---------------------------- //
// 16-Bit Transfer Instructions //
// ---------------------------- //

/// LD      dd, nn
/// 00 dd0 001
/// nnnnnnnn
/// nnnnnnnn
fn ld_dd_nn(opcode: u8, _: u16, cpu: &mut CPU) -> u8 {
    let pc = cpu.registers.pc();
    let target = RegisterDD::new((opcode >> 4) & 0b11);
    let value = memory_op::read_memory_following_u16(cpu, pc);
    debug!("{:#06X}: {:#04X} | LD   {:?}, {:?}", pc, opcode, target, value);
    cpu.registers.write_dd(target, value);
    cpu.registers.inc_pc(3);
    12
}

/// LD      sp, hl
/// 11 111 001
fn ld_sp_hl(opcode: u8, pc: u16, cpu: &mut CPU) -> u8 {
    let value = cpu.registers.hl();
    debug!(
        "{:#06X}: {:#04X} | LD   {:?}, {:?}({:?})",
        pc,
        opcode,
        RegisterDD::SP,
        RegisterDD::HL,
        value
    );
    cpu.registers.set_sp(value);
    cpu.registers.inc_pc(1);
    8
}

/// PUSH    qq
/// 11 qq0 101
fn push_qq(opcode: u8, pc: u16, cpu: &mut CPU) -> u8 {
    let register = RegisterQQ::new((opcode >> 4) & 0b11);
    let value = cpu.registers.read_qq(register);
    let sp = cpu.registers.sp();
    debug!("{:#06X}: {:#04X} | PUSH {:?}({:?})", pc, opcode, register, value);

    memory_op::push_u16_stack(cpu, value, sp);
    cpu.registers.set_sp(sp - 2);
    cpu.registers.inc_pc(1);
    16
}

/// POP    qq
/// 11 qq0 001
fn pop_qq(opcode: u8, pc: u16, cpu: &mut CPU) -> u8 {
    let register = RegisterQQ::new((opcode >> 4) & 0b11);
    let sp = cpu.registers.sp();
    let value = memory_op::pop_u16_stack(cpu, sp);
    debug!("{:#06X}: {:#04X} | POP  {:?}({:?})", pc, opcode, register, value);

    cpu.registers.write_qq(register, value);
    cpu.registers.set_sp(sp + 2);
    cpu.registers.inc_pc(1);
    12
}

/// LDHL    SP, e
/// 11 111 00
/// eeeeeeee
fn ldhl_sp_e(opcode: u8, _: u16, cpu: &mut CPU) -> u8 {
    let pc = cpu.registers.pc();
    let sp = cpu.registers.sp();
    let value = memory_op::read_memory_following_u8(cpu, pc);
    debug!("{:#06X}: {:#04X} | LDHL {:?}, {:?}", pc, opcode, RegisterDD::SP, value);
    let calculations = FlagCalculationsBuilder::new()
        .zero(Clear)
        .substraction(Clear)
        .halfcarry(Calculate)
        .carry(Calculate)
        .build();
    cpu.registers.set_flags_add((sp & 0xFF) as u8, value, 0, calculations);
    let result = add_signed(sp, value);
    cpu.registers.set_hl(result);
    cpu.registers.inc_pc(2);
    12
}

/// LD      (nn), SP
/// 00 001 000
/// nnnnnnnn
/// nnnnnnnn
fn ld_mnn_sp(opcode: u8, _: u16, cpu: &mut CPU) -> u8 {
    let pc = cpu.registers.pc();
    let address = memory_op::read_memory_following_u16(cpu, pc);
    let value = cpu.registers.sp();
    debug!(
        "{:#06X}: {:#04X} | LD   {:#06x}, {:?}({:?})",
        pc,
        opcode,
        address,
        RegisterDD::SP,
        value
    );
    {
        memory_op::write_memory(cpu, address, (value & 0xFF) as u8);
        memory_op::write_memory(cpu, address + 1, ((value >> 8) & 0xFF) as u8);
    }
    cpu.registers.inc_pc(3);
    20
}

// --------------------------------------------------- //
// 8-Bit Arithmetic and Logical Operation Instructions //
// --------------------------------------------------- //

/// ADD     A, r
/// 10 000 rrr
fn add_a_r(opcode: u8, pc: u16, cpu: &mut CPU) -> u8 {
    let register = RegisterR::new(opcode & 0b111);
    let val_a = cpu.registers.a();
    let val_r = cpu.registers.read_r(register);
    debug!(
        "{:#06X}: {:#04X} | ADD  {:?}({:?}), {:?}({:?})",
        pc,
        opcode,
        RegisterR::A,
        val_a,
        register,
        val_r
    );
    let result = val_a.wrapping_add(val_r);
    let calculations = FlagCalculationsBuilder::new()
        .zero(Calculate)
        .substraction(Clear)
        .halfcarry(Calculate)
        .carry(Calculate)
        .build();
    cpu.registers.set_flags_add(val_a, val_r, 0, calculations);
    cpu.registers.set_a(result);
    cpu.registers.inc_pc(1);
    4
}

/// ADD     A, n
/// 11 000 110
/// nnnnnnnn
fn add_a_n(opcode: u8, _: u16, cpu: &mut CPU) -> u8 {
    let pc = cpu.registers.pc();
    let val_a = cpu.registers.a();
    let val_n = memory_op::read_memory_following_u8(cpu, pc);
    debug!("{:#06X}: {:#04X} | ADD  {:?}({:?}), ({:?})", pc, opcode, RegisterR::A, val_a, val_n);
    let result = val_a.wrapping_add(val_n);
    let calculations = FlagCalculationsBuilder::new()
        .zero(Calculate)
        .substraction(Clear)
        .halfcarry(Calculate)
        .carry(Calculate)
        .build();
    cpu.registers.set_flags_add(val_a, val_n, 0, calculations);
    cpu.registers.set_a(result);
    cpu.registers.inc_pc(2);
    8
}

/// ADD     A, (HL)
/// 10 000 110
fn add_a_mhl(opcode: u8, pc: u16, cpu: &mut CPU) -> u8 {
    let hl = cpu.registers.hl();
    let val_a = cpu.registers.a();
    let val_hl = memory_op::read_memory(cpu, hl);
    debug!(
        "{:#06X}: {:#04X} | ADD  {:?}({:?}), {:?}{:#06x}({:?})",
        pc,
        opcode,
        RegisterR::A,
        val_a,
        RegisterDD::HL,
        hl,
        val_hl
    );
    let result = val_a.wrapping_add(val_hl);
    let calculations = FlagCalculationsBuilder::new()
        .zero(Calculate)
        .substraction(Clear)
        .halfcarry(Calculate)
        .carry(Calculate)
        .build();
    cpu.registers.set_flags_add(val_a, val_hl, 0, calculations);
    cpu.registers.set_a(result);
    cpu.registers.inc_pc(1);
    8
}

/// ADC     A, r
/// 10 001 rrr
fn adc_a_r(opcode: u8, pc: u16, cpu: &mut CPU) -> u8 {
    let register = RegisterR::new(opcode & 0b111);
    let val_a = cpu.registers.a();
    let val_r = cpu.registers.read_r(register);
    let cy_flag = cpu.registers.flag_cy();
    debug!(
        "{:#06X}: {:#04X} | ADC  {:?}({:?}), {:?}({:?})",
        pc,
        opcode,
        RegisterR::A,
        val_a,
        register,
        val_r
    );
    let result = val_a.wrapping_add(val_r).wrapping_add(cy_flag);
    let calculations = FlagCalculationsBuilder::new()
        .zero(Calculate)
        .substraction(Clear)
        .halfcarry(Calculate)
        .carry(Calculate)
        .build();
    cpu.registers.set_flags_add(val_a, val_r, cy_flag, calculations);
    cpu.registers.set_a(result);
    cpu.registers.inc_pc(1);
    4
}

/// ADC     A, n
/// 11 001 110
/// nnnnnnnn
fn adc_a_n(opcode: u8, _: u16, cpu: &mut CPU) -> u8 {
    let pc = cpu.registers.pc();
    let val_a = cpu.registers.a();
    let val_n = memory_op::read_memory_following_u8(cpu, pc);
    let cy_flag = cpu.registers.flag_cy();
    debug!("{:#06X}: {:#04X} | ADC  {:?}({:?}), ({:?})", pc, opcode, RegisterR::A, val_a, val_n);
    let result = val_a.wrapping_add(val_n).wrapping_add(cy_flag);
    let calculations = FlagCalculationsBuilder::new()
        .zero(Calculate)
        .substraction(Clear)
        .halfcarry(Calculate)
        .carry(Calculate)
        .build();
    cpu.registers.set_flags_add(val_a, val_n, cy_flag, calculations);
    cpu.registers.set_a(result);
    cpu.registers.inc_pc(2);
    8
}

/// ADC     A, (HL)
/// 10 001 110
fn adc_a_mhl(opcode: u8, pc: u16, cpu: &mut CPU) -> u8 {
    let hl = cpu.registers.hl();
    let val_a = cpu.registers.a();
    let val_hl = memory_op::read_memory(cpu, hl);
    let cy_flag = cpu.registers.flag_cy();
    debug!(
        "{:#06X}: {:#04X} | ADC  {:?}({:?}), {:?}{:#06x}({:?})",
        pc,
        opcode,
        RegisterR::A,
        val_a,
        RegisterDD::HL,
        hl,
        val_hl
    );
    let result = val_a.wrapping_add(val_hl).wrapping_add(cy_flag);
    let calculations = FlagCalculationsBuilder::new()
        .zero(Calculate)
        .substraction(Clear)
        .halfcarry(Calculate)
        .carry(Calculate)
        .build();
    cpu.registers.set_flags_add(val_a, val_hl, cy_flag, calculations);
    cpu.registers.set_a(result);
    cpu.registers.inc_pc(1);
    8
}

/// SUB     A, r
/// 10 010 rrr
fn sub_a_r(opcode: u8, pc: u16, cpu: &mut CPU) -> u8 {
    let register = RegisterR::new(opcode & 0b111);
    let val_a = cpu.registers.a();
    let val_r = cpu.registers.read_r(register);
    debug!(
        "{:#06X}: {:#04X} | SUB  {:?}({:?}), {:?}({:?})",
        pc,
        opcode,
        RegisterR::A,
        val_a,
        register,
        val_r
    );
    let result = val_a.wrapping_sub(val_r);
    let calculations = FlagCalculationsBuilder::new()
        .zero(Calculate)
        .substraction(Set)
        .halfcarry(Calculate)
        .carry(Calculate)
        .build();
    cpu.registers.set_flags_sub(val_a, val_r, 0, calculations);
    cpu.registers.set_a(result);
    cpu.registers.inc_pc(1);
    4
}

/// SUB     A, n
/// 11 010 110
/// nnnnnnnn
fn sub_a_n(opcode: u8, _: u16, cpu: &mut CPU) -> u8 {
    let pc = cpu.registers.pc();
    let val_a = cpu.registers.a();
    let val_n = memory_op::read_memory_following_u8(cpu, pc);
    debug!("{:#06X}: {:#04X} | SUB  {:?}({:?}), ({:?})", pc, opcode, RegisterR::A, val_a, val_n);
    let result = val_a.wrapping_sub(val_n);
    let calculations = FlagCalculationsBuilder::new()
        .zero(Calculate)
        .substraction(Set)
        .halfcarry(Calculate)
        .carry(Calculate)
        .build();
    cpu.registers.set_flags_sub(val_a, val_n, 0, calculations);
    cpu.registers.set_a(result);
    cpu.registers.inc_pc(2);
    8
}

/// SUB     A, (HL)
/// 10 010 110
fn sub_a_mhl(opcode: u8, pc: u16, cpu: &mut CPU) -> u8 {
    let hl = cpu.registers.hl();
    let val_a = cpu.registers.a();
    let val_hl = memory_op::read_memory(cpu, hl);
    debug!(
        "{:#06X}: {:#04X} | SUB  {:?}({:?}), {:?}{:#06x}({:?})",
        pc,
        opcode,
        RegisterR::A,
        val_a,
        RegisterDD::HL,
        hl,
        val_hl
    );
    let result = val_a.wrapping_sub(val_hl);
    let calculations = FlagCalculationsBuilder::new()
        .zero(Calculate)
        .substraction(Set)
        .halfcarry(Calculate)
        .carry(Calculate)
        .build();
    cpu.registers.set_flags_sub(val_a, val_hl, 0, calculations);
    cpu.registers.set_a(result);
    cpu.registers.inc_pc(1);
    8
}

/// SBC     A, r
/// 10 010 rrr
fn sbc_a_r(opcode: u8, pc: u16, cpu: &mut CPU) -> u8 {
    let register = RegisterR::new(opcode & 0b111);
    let val_a = cpu.registers.a();
    let val_r = cpu.registers.read_r(register);
    let cy_flag = cpu.registers.flag_cy();
    debug!(
        "{:#06X}: {:#04X} | SBC  {:?}({:?}), {:?}({:?})",
        pc,
        opcode,
        RegisterR::A,
        val_a,
        register,
        val_r
    );
    let result = val_a.wrapping_sub(val_r).wrapping_sub(cy_flag);
    let calculations = FlagCalculationsBuilder::new()
        .zero(Calculate)
        .substraction(Set)
        .halfcarry(Calculate)
        .carry(Calculate)
        .build();
    cpu.registers.set_flags_sub(val_a, val_r, cy_flag, calculations);
    cpu.registers.set_a(result);
    cpu.registers.inc_pc(1);
    4
}

/// SBC     A, n
/// 11 010 110
/// nnnnnnnn
fn sbc_a_n(opcode: u8, _: u16, cpu: &mut CPU) -> u8 {
    let pc = cpu.registers.pc();
    let val_a = cpu.registers.a();
    let val_n = memory_op::read_memory_following_u8(cpu, pc);
    let cy_flag = cpu.registers.flag_cy();
    debug!("{:#06X}: {:#04X} | SBC  {:?}({:?}), ({:?})", pc, opcode, RegisterR::A, val_a, val_n);
    let result = val_a.wrapping_sub(val_n).wrapping_sub(cy_flag);
    let calculations = FlagCalculationsBuilder::new()
        .zero(Calculate)
        .substraction(Set)
        .halfcarry(Calculate)
        .carry(Calculate)
        .build();
    cpu.registers.set_flags_sub(val_a, val_n, cy_flag, calculations);
    cpu.registers.set_a(result);
    cpu.registers.inc_pc(2);
    8
}

/// SBC     A, (HL)
/// 10 010 110
fn sbc_a_mhl(opcode: u8, pc: u16, cpu: &mut CPU) -> u8 {
    let hl = cpu.registers.hl();
    let val_a = cpu.registers.a();
    let val_hl = memory_op::read_memory(cpu, hl);
    let cy_flag = cpu.registers.flag_cy();
    debug!(
        "{:#06X}: {:#04X} | SBC  {:?}({:?}), {:?}{:#06x}({:?})",
        pc,
        opcode,
        RegisterR::A,
        val_a,
        RegisterDD::HL,
        hl,
        val_hl
    );
    let result = val_a.wrapping_sub(val_hl).wrapping_sub(cy_flag);
    let calculations = FlagCalculationsBuilder::new()
        .zero(Calculate)
        .substraction(Set)
        .halfcarry(Calculate)
        .carry(Calculate)
        .build();
    cpu.registers.set_flags_sub(val_a, val_hl, cy_flag, calculations);
    cpu.registers.set_a(result);
    cpu.registers.inc_pc(1);
    8
}

/// AND     r
/// 10 100 rrr
fn and_r(opcode: u8, pc: u16, cpu: &mut CPU) -> u8 {
    let register = RegisterR::new(opcode & 0b111);
    let value = cpu.registers.read_r(register);
    let reg_a_value = cpu.registers.a();
    debug!(
        "{:#06X}: {:#04X} | AND  {:?}({:?}), {:?}({:?})",
        pc,
        opcode,
        RegisterR::A,
        reg_a_value,
        register,
        value
    );
    let result = reg_a_value & value;
    cpu.registers.set_flags(if result == 0 { 1 } else { 0 }, 0, 1, 0);
    cpu.registers.set_a(result);
    cpu.registers.inc_pc(1);
    4
}

/// AND     n
/// 11 100 110
/// nnnnnnnn
fn and_n(opcode: u8, _: u16, cpu: &mut CPU) -> u8 {
    let pc = cpu.registers.pc();
    let value = memory_op::read_memory_following_u8(cpu, pc);
    let reg_a_value = cpu.registers.a();
    debug!(
        "{:#06X}: {:#04X} | AND  {:?}({:?}), {:?}",
        pc,
        opcode,
        RegisterR::A,
        reg_a_value,
        value
    );
    let result = reg_a_value & value;
    cpu.registers.set_flags(if result == 0 { 1 } else { 0 }, 0, 1, 0);
    cpu.registers.set_a(result);
    cpu.registers.inc_pc(2);
    8
}

/// AND     (HL)
/// 10 100 110
fn and_mhl(opcode: u8, pc: u16, cpu: &mut CPU) -> u8 {
    let address = cpu.registers.hl();
    let value = memory_op::read_memory(cpu, address);
    let reg_a_value = cpu.registers.a();
    debug!(
        "{:#06X}: {:#04X} | AND  {:?}({:?}), {:?}[{:?}]({:?})",
        pc,
        opcode,
        RegisterR::A,
        reg_a_value,
        RegisterDD::HL,
        address,
        value
    );
    let result = reg_a_value & value;
    cpu.registers.set_flags(if result == 0 { 1 } else { 0 }, 0, 1, 0);
    cpu.registers.set_a(result);
    cpu.registers.inc_pc(1);
    8
}

/// OR      r
/// 10 110 rrr
fn or_r(opcode: u8, pc: u16, cpu: &mut CPU) -> u8 {
    let register = RegisterR::new(opcode & 0b111);
    let value = cpu.registers.read_r(register);
    let reg_a_value = cpu.registers.a();
    debug!(
        "{:#06X}: {:#04X} | OR   {:?}({:?}), {:?}({:?})",
        pc,
        opcode,
        RegisterR::A,
        reg_a_value,
        register,
        value
    );
    let result = reg_a_value | value;
    cpu.registers.set_flags(if result == 0 { 1 } else { 0 }, 0, 0, 0);
    cpu.registers.set_a(result);
    cpu.registers.inc_pc(1);
    4
}

/// OR      n
/// 11 110 110
/// nnnnnnnn
fn or_n(opcode: u8, _: u16, cpu: &mut CPU) -> u8 {
    let pc = cpu.registers.pc();
    let value = memory_op::read_memory_following_u8(cpu, pc);
    let reg_a_value = cpu.registers.a();
    debug!(
        "{:#06X}: {:#04X} | OR   {:?}({:?}), {:?}",
        pc,
        opcode,
        RegisterR::A,
        reg_a_value,
        value
    );
    let result = reg_a_value | value;
    cpu.registers.set_flags(if result == 0 { 1 } else { 0 }, 0, 0, 0);
    cpu.registers.set_a(result);
    cpu.registers.inc_pc(2);
    8
}

/// OR      (HL)
/// 10 110 110
fn or_mhl(opcode: u8, pc: u16, cpu: &mut CPU) -> u8 {
    let address = cpu.registers.hl();
    let value = memory_op::read_memory(cpu, address);
    let reg_a_value = cpu.registers.a();
    debug!(
        "{:#06X}: {:#04X} | OR   {:?}({:?}), {:?}[{:?}]({:?})",
        pc,
        opcode,
        RegisterR::A,
        reg_a_value,
        RegisterDD::HL,
        address,
        value
    );
    let result = reg_a_value | value;
    cpu.registers.set_flags(if result == 0 { 1 } else { 0 }, 0, 0, 0);
    cpu.registers.set_a(result);
    cpu.registers.inc_pc(1);
    8
}

/// XOR     r
/// 10 101 rrr
fn xor_r(opcode: u8, pc: u16, cpu: &mut CPU) -> u8 {
    let register = RegisterR::new(opcode & 0b111);
    let value = cpu.registers.read_r(register);
    let reg_a_value = cpu.registers.a();
    debug!("{:#06X}: {:#04X} | XOR  {:?}({:?}), A({:?})", pc, opcode, register, value, reg_a_value);
    let result = reg_a_value ^ value;
    cpu.registers.set_flags(if result == 0 { 1 } else { 0 }, 0, 0, 0);
    cpu.registers.set_a(result);
    cpu.registers.inc_pc(1);
    4
}

/// XOR     n
/// 11 101 110
/// nnnnnnnn
fn xor_n(opcode: u8, _: u16, cpu: &mut CPU) -> u8 {
    let pc = cpu.registers.pc();
    let value = memory_op::read_memory_following_u8(cpu, pc);
    let reg_a_value = cpu.registers.a();
    debug!(
        "{:#06X}: {:#04X} | XOR  {:?}({:?}), {:?}",
        pc,
        opcode,
        RegisterR::A,
        reg_a_value,
        value
    );
    let result = reg_a_value ^ value;
    cpu.registers.set_flags(if result == 0 { 1 } else { 0 }, 0, 0, 0);
    cpu.registers.set_a(result);
    cpu.registers.inc_pc(2);
    8
}

/// XOR     (HL)
/// 10 101 110
fn xor_mhl(opcode: u8, pc: u16, cpu: &mut CPU) -> u8 {
    let address = cpu.registers.hl();
    let value = memory_op::read_memory(cpu, address);
    let reg_a_value = cpu.registers.a();
    debug!(
        "{:#06X}: {:#04X} | XOR  {:?}({:?}), {:?}[{:?}]({:?})",
        pc,
        opcode,
        RegisterR::A,
        reg_a_value,
        RegisterDD::HL,
        address,
        value
    );
    let result = reg_a_value ^ value;
    cpu.registers.set_flags(if result == 0 { 1 } else { 0 }, 0, 0, 0);
    cpu.registers.set_a(result);
    cpu.registers.inc_pc(1);
    8
}

/// CP      r
/// 10 111 rrr
fn cp_r(opcode: u8, pc: u16, cpu: &mut CPU) -> u8 {
    let register = RegisterR::new(opcode & 0b111);
    let val_a = cpu.registers.a();
    let val_r = cpu.registers.read_r(register);
    debug!(
        "{:#06X}: {:#04X} | CP   {:?}({:?}), {:?}({:?})",
        pc,
        opcode,
        RegisterR::A,
        val_a,
        register,
        val_r
    );
    let calculations = FlagCalculationsBuilder::new()
        .zero(Calculate)
        .substraction(Set)
        .halfcarry(Calculate)
        .carry(Calculate)
        .build();
    cpu.registers.set_flags_sub(val_a, val_r, 0, calculations);
    cpu.registers.inc_pc(1);
    4
}

/// CP      n
/// 11 111 110
/// nnnnnnnn
fn cp_n(opcode: u8, _: u16, cpu: &mut CPU) -> u8 {
    let pc = cpu.registers.pc();
    let val_a = cpu.registers.a();
    let val_n = memory_op::read_memory_following_u8(cpu, pc);
    debug!("{:#06X}: {:#04X} | CP   {:?}({:?}), ({:?})", pc, opcode, RegisterR::A, val_a, val_n);
    let calculations = FlagCalculationsBuilder::new()
        .zero(Calculate)
        .substraction(Set)
        .halfcarry(Calculate)
        .carry(Calculate)
        .build();
    cpu.registers.set_flags_sub(val_a, val_n, 0, calculations);
    cpu.registers.inc_pc(2);
    8
}

/// CP      (HL)
/// 10 111 110
fn cp_mhl(opcode: u8, pc: u16, cpu: &mut CPU) -> u8 {
    let hl = cpu.registers.hl();
    let val_a = cpu.registers.a();
    let val_hl = memory_op::read_memory(cpu, hl);
    debug!(
        "{:#06X}: {:#04X} | SUB  {:?}({:?}), {:?}{:#06x}({:?})",
        pc,
        opcode,
        RegisterR::A,
        val_a,
        RegisterDD::HL,
        hl,
        val_hl
    );
    let calculations = FlagCalculationsBuilder::new()
        .zero(Calculate)
        .substraction(Set)
        .halfcarry(Calculate)
        .carry(Calculate)
        .build();
    cpu.registers.set_flags_sub(val_a, val_hl, 0, calculations);
    cpu.registers.inc_pc(1);
    8
}

/// INC     r
/// 00 rrr 100
fn inc_r(opcode: u8, pc: u16, cpu: &mut CPU) -> u8 {
    let register = RegisterR::new((opcode >> 3) & 0b111);
    let value = cpu.registers.read_r(register);
    debug!("{:#06X}: {:#04X} | INC  {:?}({:?})", pc, opcode, register, value);
    let calculations = FlagCalculationsBuilder::new()
        .zero(Calculate)
        .substraction(Clear)
        .halfcarry(Calculate)
        .build();
    cpu.registers.set_flags_add(value, 1, 0, calculations);
    cpu.registers.write_r(register, value.wrapping_add(1));
    cpu.registers.inc_pc(1);
    4
}

/// INC     (HL)
/// 00 110 100
fn inc_mhl(opcode: u8, pc: u16, cpu: &mut CPU) -> u8 {
    let address = cpu.registers.hl();
    let value = memory_op::read_memory(cpu, address);
    memory_op::write_memory(cpu, address, value.wrapping_add(1));
    debug!("{:#06X}: {:#04X} | INC  {:?}{:#06x}({:?})", pc, opcode, RegisterDD::HL, address, value);
    let calculations = FlagCalculationsBuilder::new()
        .zero(Calculate)
        .substraction(Clear)
        .halfcarry(Calculate)
        .build();
    cpu.registers.set_flags_add(value, 1, 0, calculations);
    cpu.registers.inc_pc(1);
    12
}

/// DEC     r
/// 00 rrr 101
fn dec_r(opcode: u8, pc: u16, cpu: &mut CPU) -> u8 {
    let register = RegisterR::new((opcode >> 3) & 0b111);
    let value = cpu.registers.read_r(register);
    debug!("{:#06X}: {:#04X} | DEC  {:?}({:?})", pc, opcode, register, value);
    let calculations = FlagCalculationsBuilder::new()
        .zero(Calculate)
        .substraction(Set)
        .halfcarry(Calculate)
        .build();
    cpu.registers.set_flags_sub(value, 1, 0, calculations);
    cpu.registers.write_r(register, value.wrapping_sub(1));
    cpu.registers.inc_pc(1);
    4
}

/// DEC     (HL)
/// 00 110 101
fn dec_mhl(opcode: u8, pc: u16, cpu: &mut CPU) -> u8 {
    let address = cpu.registers.hl();
    let value = memory_op::read_memory(cpu, address);
    memory_op::write_memory(cpu, address, value.wrapping_sub(1));
    debug!(
        "{:#06X}: {:#04X} | DEC  {:?}[{:#06x}]({:?})",
        pc,
        opcode,
        RegisterDD::HL,
        address,
        value
    );
    let calculations = FlagCalculationsBuilder::new()
        .zero(Calculate)
        .substraction(Set)
        .halfcarry(Calculate)
        .build();
    cpu.registers.set_flags_sub(value, 1, 0, calculations);
    cpu.registers.inc_pc(1);
    12
}

// ---------------------------------------- //
// 16-Bit Arithmetic Operation Instructions //
// ---------------------------------------- //

/// ADD     HL, ss
/// 00 ss1 001
fn add_hl_ss(opcode: u8, pc: u16, cpu: &mut CPU) -> u8 {
    let register = RegisterSS::new((opcode >> 4) & 0b111);
    let value = cpu.registers.read_ss(register);
    let reg_hl_value = cpu.registers.hl();
    debug!(
        "{:#06X}: {:#04X} | ADD  {:?}({:?}), {:?}({:?})",
        pc,
        opcode,
        RegisterSS::HL,
        reg_hl_value,
        register,
        value
    );
    let result = reg_hl_value.wrapping_add(value);
    let calculations = FlagCalculationsBuilder::new()
        .substraction(Clear)
        .halfcarry(Calculate)
        .carry(Calculate)
        .build();
    cpu.registers.set_flags_add_u16(reg_hl_value, value as u16, 0, calculations);
    cpu.registers.set_hl(result);
    cpu.registers.inc_pc(1);
    8
}

/// ADD     SP, e
/// 11 101 000
/// eeeeeeee
fn add_sp_e(opcode: u8, _: u16, cpu: &mut CPU) -> u8 {
    let pc = cpu.registers.pc();
    let val_sp = cpu.registers.sp();
    let val_n = memory_op::read_memory_following_u8(cpu, pc);
    debug!("{:#06X}: {:#04X} | ADD  {:?}({:?}), ({:?})", pc, opcode, RegisterSS::SP, val_sp, val_n);
    let result = add_signed(val_sp, val_n);
    debug!("Result: {:#06X}", result);
    let calculations = FlagCalculationsBuilder::new()
        .zero(Clear)
        .substraction(Clear)
        .halfcarry(Calculate)
        .carry(Calculate)
        .build();
    cpu.registers.set_flags_add((val_sp & 0xFF) as u8, val_n, 0, calculations);
    cpu.registers.set_sp(result);
    cpu.registers.inc_pc(2);
    16
}

/// INC     ss
/// 00 ss0 011
fn inc_ss(opcode: u8, pc: u16, cpu: &mut CPU) -> u8 {
    let register = RegisterSS::new((opcode >> 4) & 0b11);
    let value = cpu.registers.read_ss(register);
    debug!("{:#06X}: {:#04X} | INC  {:?}({:?})", pc, opcode, register, value);
    let calculations = FlagCalculationsBuilder::new().build();
    cpu.registers.set_flags_add_u16(value, 1, 0, calculations);
    cpu.registers.write_ss(register, value.wrapping_add(1));
    cpu.registers.inc_pc(1);
    8
}

/// DEC     ss
/// 00 ss1 011
fn dec_ss(opcode: u8, pc: u16, cpu: &mut CPU) -> u8 {
    let register = RegisterSS::new((opcode >> 4) & 0b11);
    let value = cpu.registers.read_ss(register);
    debug!("{:#06X}: {:#04X} | DEC  {:?}({:?})", pc, opcode, register, value);
    let calculations = FlagCalculationsBuilder::new().build();
    cpu.registers.set_flags_sub_u16(value, 1, 0, calculations);
    cpu.registers.write_ss(register, value.wrapping_sub(1));
    cpu.registers.inc_pc(1);
    8
}

// ------------------------- //
// Rotate Shift Instructions //
// ------------------------- //

/// RLCA
/// 00 000 111
fn rlca(opcode: u8, pc: u16, cpu: &mut CPU) -> u8 {
    rlc_r_internal(opcode, pc, RegisterR::A, false, &mut cpu.registers);
    cpu.registers.inc_pc(1);
    4
}

/// RLA
/// 00 010 111
fn rla(opcode: u8, pc: u16, cpu: &mut CPU) -> u8 {
    rl_r_internal(opcode, pc, RegisterR::A, false, &mut cpu.registers);
    cpu.registers.inc_pc(1);
    4
}

/// RRCA
/// 00 001 111
fn rrca(opcode: u8, pc: u16, cpu: &mut CPU) -> u8 {
    rrc_r_internal(opcode, pc, RegisterR::A, false, &mut cpu.registers);
    cpu.registers.inc_pc(1);
    4
}

/// RRA
/// 00 011 111
fn rra(opcode: u8, pc: u16, cpu: &mut CPU) -> u8 {
    rr_r_internal(opcode, pc, RegisterR::A, false, &mut cpu.registers);
    cpu.registers.inc_pc(1);
    4
}

/// RLC     r
/// 11 001 011
/// 00 000 rrr
fn rlc_r(ext_opcode: u8, pc: u16, cpu: &mut CPU) -> u8 {
    let register = RegisterR::new(ext_opcode & 0b111);
    rlc_r_internal(ext_opcode, pc, register, true, &mut cpu.registers);
    cpu.registers.inc_pc(2);
    8
}

/// RLC     (HL)
/// 11 001 011
/// 00 000 110
fn rlc_mhl(ext_opcode: u8, pc: u16, cpu: &mut CPU) -> u8 {
    let address = cpu.registers.hl();
    let value = memory_op::read_memory(cpu, address);
    debug!(
        "{:#06X}: {:#04X} | RLC   {:?}[{:#06x}]({:#010b})",
        pc,
        ext_opcode,
        RegisterDD::HL,
        address,
        value
    );
    let rotated = rlc_m(value, true, &mut cpu.registers);
    memory_op::write_memory(cpu, address, rotated);
    cpu.registers.inc_pc(2);
    16
}

/// RL      r
/// 11 001 011
/// 00 010 rrr
fn rl_r(ext_opcode: u8, pc: u16, cpu: &mut CPU) -> u8 {
    let register = RegisterR::new(ext_opcode & 0b111);
    rl_r_internal(ext_opcode, pc, register, true, &mut cpu.registers);
    cpu.registers.inc_pc(2);
    8
}

/// RL      (HL)
/// 11 001 011
/// 00 010 110
fn rl_mhl(ext_opcode: u8, pc: u16, cpu: &mut CPU) -> u8 {
    let address = cpu.registers.hl();
    let value = memory_op::read_memory(cpu, address);
    debug!(
        "{:#06X}: {:#04X} | RL   {:?}[{:#06x}]({:#010b})",
        pc,
        ext_opcode,
        RegisterDD::HL,
        address,
        value
    );
    let rotated = rl_m(value, true, &mut cpu.registers);
    memory_op::write_memory(cpu, address, rotated);
    cpu.registers.inc_pc(2);
    16
}

/// RRC     r
/// 11 001 011
/// 00 001 rrr
fn rrc_r(ext_opcode: u8, pc: u16, cpu: &mut CPU) -> u8 {
    let register = RegisterR::new(ext_opcode & 0b111);
    rrc_r_internal(ext_opcode, pc, register, true, &mut cpu.registers);
    cpu.registers.inc_pc(2);
    8
}

/// RRC     (HL)
/// 11 001 011
/// 00 001 110
fn rrc_mhl(ext_opcode: u8, pc: u16, cpu: &mut CPU) -> u8 {
    let address = cpu.registers.hl();
    let value = memory_op::read_memory(cpu, address);
    debug!(
        "{:#06X}: {:#04X} | RRC   {:?}[{:#06x}]({:#010b})",
        pc,
        ext_opcode,
        RegisterDD::HL,
        address,
        value
    );
    let rotated = rrc_m(value, true, &mut cpu.registers);
    memory_op::write_memory(cpu, address, rotated);
    cpu.registers.inc_pc(2);
    16
}

/// RR      r
/// 11 001 011
/// 00 011 rrr
fn rr_r(ext_opcode: u8, pc: u16, cpu: &mut CPU) -> u8 {
    let register = RegisterR::new(ext_opcode & 0b111);
    rr_r_internal(ext_opcode, pc, register, true, &mut cpu.registers);
    cpu.registers.inc_pc(2);
    8
}

/// RR      (HL)
/// 11 001 011
/// 00 011 110
fn rr_mhl(ext_opcode: u8, pc: u16, cpu: &mut CPU) -> u8 {
    let address = cpu.registers.hl();
    let value = memory_op::read_memory(cpu, address);
    debug!(
        "{:#06X}: {:#04X} | RR   {:?}[{:#06x}]({:#010b})",
        pc,
        ext_opcode,
        RegisterDD::HL,
        address,
        value
    );
    let rotated = rr_m(value, true, &mut cpu.registers);
    memory_op::write_memory(cpu, address, rotated);
    cpu.registers.inc_pc(2);
    16
}

fn rlc_r_internal(
    opcode: u8,
    pc: u16,
    register: RegisterR,
    calc_zero: bool,
    registers: &mut Registers,
) {
    let value = registers.read_r(register);
    debug!("{:#06X}: {:#04X} | RLC   {:?}({:#010b})", pc, opcode, register, value);
    let rotated = rlc_m(value, calc_zero, registers);
    registers.write_r(register, rotated);
}

fn rl_r_internal(
    opcode: u8,
    pc: u16,
    register: RegisterR,
    calc_zero: bool,
    registers: &mut Registers,
) {
    let value = registers.read_r(register);
    debug!("{:#06X}: {:#04X} | RL   {:?}({:#010b})", pc, opcode, register, value);
    let rotated = rl_m(value, calc_zero, registers);
    registers.write_r(register, rotated);
}

fn rrc_r_internal(
    opcode: u8,
    pc: u16,
    register: RegisterR,
    calc_zero: bool,
    registers: &mut Registers,
) {
    let value = registers.read_r(register);
    debug!("{:#06X}: {:#04X} | RRC   {:?}({:#010b})", pc, opcode, register, value);
    let rotated = rrc_m(value, calc_zero, registers);
    registers.write_r(register, rotated);
}

fn rr_r_internal(
    opcode: u8,
    pc: u16,
    register: RegisterR,
    calc_zero: bool,
    registers: &mut Registers,
) {
    let value = registers.read_r(register);
    debug!("{:#06X}: {:#04X} | RR   {:?}({:#010b})", pc, opcode, register, value);
    let rotated = rr_m(value, calc_zero, registers);
    registers.write_r(register, rotated);
}

fn rlc_m(value: u8, calc_zero: bool, registers: &mut Registers) -> u8 {
    let mut flags = registers.f();
    let bit7 = (value >> 7) & 1;
    let rotated = (value << 1) | bit7;
    flags = calc_flags_for_shift_and_rotate(flags, bit7, rotated, calc_zero);

    registers.set_f(flags);
    rotated
}

fn rl_m(value: u8, calc_zero: bool, registers: &mut Registers) -> u8 {
    let mut flags = registers.f();
    let bit7 = (value >> 7) & 1;
    let cy = registers.flag_cy();
    let rotated = (value << 1) | cy;
    flags = calc_flags_for_shift_and_rotate(flags, bit7, rotated, calc_zero);

    registers.set_f(flags);
    rotated
}

fn rrc_m(value: u8, calc_zero: bool, registers: &mut Registers) -> u8 {
    let mut flags = registers.f();
    let bit0 = (value) & 1;
    let rotated = (value >> 1) | (bit0 << 7);
    flags = calc_flags_for_shift_and_rotate(flags, bit0, rotated, calc_zero);

    registers.set_f(flags);
    rotated
}

fn rr_m(value: u8, calc_zero: bool, registers: &mut Registers) -> u8 {
    let mut flags = registers.f();
    let bit0 = value & 1;
    let cy = registers.flag_cy();
    let rotated = (value >> 1) | (cy << 7);
    flags = calc_flags_for_shift_and_rotate(flags, bit0, rotated, calc_zero);

    registers.set_f(flags);
    rotated
}

fn calc_flags_for_shift_and_rotate(
    mut flags: u8,
    bit_value: u8,
    calculated_result: u8,
    calc_zero: bool,
) -> u8 {
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
fn sla_r(ext_opcode: u8, pc: u16, cpu: &mut CPU) -> u8 {
    let register = RegisterR::new(ext_opcode & 0b111);
    let value = cpu.registers.read_r(register);
    debug!("{:#06X}: {:#04X} | SLA   {:?}({:#010b})", pc, ext_opcode, register, value);
    let bit7 = (value >> 7) & 1;
    let result = value << 1;
    let flags = calc_flags_for_shift_and_rotate(cpu.registers.f(), bit7, result, true);
    cpu.registers.set_f(flags);
    cpu.registers.write_r(register, result);
    cpu.registers.inc_pc(2);
    8
}

/// SLA     (HL)
/// 11 001 011
/// 00 100 110
fn sla_mhl(ext_opcode: u8, pc: u16, cpu: &mut CPU) -> u8 {
    let address = cpu.registers.hl();
    let value = memory_op::read_memory(cpu, address);
    debug!(
        "{:#06X}: {:#04X} | SLA   {:?}[{:#06x}]({:#010b})",
        pc,
        ext_opcode,
        RegisterDD::HL,
        address,
        value
    );
    let bit7 = (value >> 7) & 1;
    let result = value << 1;
    let flags = calc_flags_for_shift_and_rotate(cpu.registers.f(), bit7, result, true);
    cpu.registers.set_f(flags);
    memory_op::write_memory(cpu, address, result);
    cpu.registers.inc_pc(2);
    16
}

/// SRA     r
/// 11 001 011
/// 00 100 rrr
fn sra_r(ext_opcode: u8, pc: u16, cpu: &mut CPU) -> u8 {
    let register = RegisterR::new(ext_opcode & 0b111);
    let value = cpu.registers.read_r(register);
    debug!("{:#06X}: {:#04X} | SRA   {:?}({:#010b})", pc, ext_opcode, register, value);
    let bit0 = value & 1;
    let bit7 = (value >> 7) & 1;
    let result = (value >> 1) | (bit7 << 7);
    let flags = calc_flags_for_shift_and_rotate(cpu.registers.f(), bit0, result, true);
    cpu.registers.set_f(flags);
    cpu.registers.write_r(register, result);
    cpu.registers.inc_pc(2);
    8
}

/// SRA     (HL)
/// 11 001 011
/// 00 100 110
fn sra_mhl(ext_opcode: u8, pc: u16, cpu: &mut CPU) -> u8 {
    let address = cpu.registers.hl();
    let value = memory_op::read_memory(cpu, address);
    debug!(
        "{:#06X}: {:#04X} | SRA   {:?}[{:#06x}]({:#010b})",
        pc,
        ext_opcode,
        RegisterDD::HL,
        address,
        value
    );
    let bit0 = value & 1;
    let bit7 = (value >> 7) & 1;
    let result = (value >> 1) | (bit7 << 7);
    let flags = calc_flags_for_shift_and_rotate(cpu.registers.f(), bit0, result, true);
    cpu.registers.set_f(flags);
    memory_op::write_memory(cpu, address, result);
    cpu.registers.inc_pc(2);
    16
}

/// SRL     r
/// 11 001 011
/// 00 111 rrr
fn srl_r(ext_opcode: u8, pc: u16, cpu: &mut CPU) -> u8 {
    let register = RegisterR::new(ext_opcode & 0b111);
    let value = cpu.registers.read_r(register);
    debug!("{:#06X}: {:#04X} | SRL   {:?}({:#010b})", pc, ext_opcode, register, value);
    let bit0 = value & 1;
    let result = value >> 1;
    let flags = calc_flags_for_shift_and_rotate(cpu.registers.f(), bit0, result, true);
    cpu.registers.set_f(flags);
    cpu.registers.write_r(register, result);
    cpu.registers.inc_pc(2);
    8
}

/// SRL     (HL)
/// 11 001 011
/// 00 111 110
fn srl_mhl(ext_opcode: u8, pc: u16, cpu: &mut CPU) -> u8 {
    let address = cpu.registers.hl();
    let value = memory_op::read_memory(cpu, address);
    debug!(
        "{:#06X}: {:#04X} | SRL   {:?}[{:#06x}]({:#010b})",
        pc,
        ext_opcode,
        RegisterDD::HL,
        address,
        value
    );
    let bit0 = value & 1;
    let result = value >> 1;
    let flags = calc_flags_for_shift_and_rotate(cpu.registers.f(), bit0, result, true);
    cpu.registers.set_f(flags);
    memory_op::write_memory(cpu, address, result);
    cpu.registers.inc_pc(2);
    16
}

/// SWAP    r
/// 11 001 011
/// 00 110 rrr
fn swap_r(ext_opcode: u8, pc: u16, cpu: &mut CPU) -> u8 {
    let register = RegisterR::new(ext_opcode & 0b111);
    let value = cpu.registers.read_r(register);
    debug!("{:#06X}: {:#04X} | SWAP  {:?}({:#010b})", pc, ext_opcode, register, value);
    let result = ((value & 0b1111) << 4) | (value >> 4) & 0b1111;
    cpu.registers.set_flags(if result == 0 { 1 } else { 0 }, 0, 0, 0);
    cpu.registers.write_r(register, result);
    cpu.registers.inc_pc(2);
    8
}

/// SWAP    (HL)
/// 11 001 011
/// 00 110 110
fn swap_mhl(ext_opcode: u8, pc: u16, cpu: &mut CPU) -> u8 {
    let address = cpu.registers.hl();
    let value = memory_op::read_memory(cpu, address);
    debug!(
        "{:#06X}: {:#04X} | SWAP  {:?}[{:#06x}]({:#010b})",
        pc,
        ext_opcode,
        RegisterDD::HL,
        address,
        value
    );
    let result = ((value & 0b1111) << 4) | (value >> 4) & 0b1111;
    cpu.registers.set_flags(if result == 0 { 1 } else { 0 }, 0, 0, 0);
    memory_op::write_memory(cpu, address, result);
    cpu.registers.inc_pc(2);
    16
}

// -------------- //
// Bit Operations //
// -------------- //

/// BIT     b, r
/// 11 001 011
/// 01 bbb rrr
fn bit_b_r(ext_opcode: u8, pc: u16, cpu: &mut CPU) -> u8 {
    let register = RegisterR::new(ext_opcode & 0b111);
    let value = cpu.registers.read_r(register);
    let bit = (ext_opcode >> 3) & 0b111;
    debug!("{:#06X}: {:#04X} | BIT  {:?}, {:?}({:#010b})", pc, ext_opcode, bit, register, value);

    let bit_value = if ((value >> bit) & 0b1) == 0 { 1 } else { 0 };
    let mut flags = cpu.registers.f();
    flags = bit_op::set_bit(flags, 5);
    flags = bit_op::clear_bit(flags, 6);
    flags = bit_op::change_bit_to(flags, 7, bit_value);
    cpu.registers.set_f(flags);
    cpu.registers.inc_pc(2);
    8
}

/// BIT     b, (HL)
/// 11 001 011
/// 01 bbb 110
fn bit_b_mhl(ext_opcode: u8, pc: u16, cpu: &mut CPU) -> u8 {
    let address = cpu.registers.hl();
    let value = memory_op::read_memory(cpu, address);
    let bit = (ext_opcode >> 3) & 0b111;
    debug!(
        "{:#06X}: {:#04X} | BIT  {:?}, [{:#06x}]({:#010b})",
        pc, ext_opcode, bit, address, value
    );

    let bit_value = if ((value >> bit) & 0b1) == 0 { 1 } else { 0 };
    let mut flags = cpu.registers.f();
    flags = bit_op::set_bit(flags, 5);
    flags = bit_op::clear_bit(flags, 6);
    flags = bit_op::change_bit_to(flags, 7, bit_value);
    cpu.registers.set_f(flags);
    cpu.registers.inc_pc(2);
    12
}

/// SET     b, r
/// 11 001 011
/// 11 bbb rrr
fn set_b_r(ext_opcode: u8, pc: u16, cpu: &mut CPU) -> u8 {
    let register = RegisterR::new(ext_opcode & 0b111);
    let value = cpu.registers.read_r(register);
    let bit = (ext_opcode >> 3) & 0b111;
    debug!("{:#06X}: {:#04X} | SET  {:?}, {:?}({:#010b})", pc, ext_opcode, bit, register, value);

    let result = bit_op::set_bit(value, bit);
    cpu.registers.write_r(register, result);
    cpu.registers.inc_pc(2);
    8
}

/// SET     b, (HL)
/// 11 001 011
/// 11 bbb 110
fn set_b_mhl(ext_opcode: u8, pc: u16, cpu: &mut CPU) -> u8 {
    let address = cpu.registers.hl();
    let value = memory_op::read_memory(cpu, address);
    let bit = (ext_opcode >> 3) & 0b111;
    debug!(
        "{:#06X}: {:#04X} | SET  {:?}, [{:#06x}]({:#010b})",
        pc, ext_opcode, bit, address, value
    );

    let result = bit_op::set_bit(value, bit);
    memory_op::write_memory(cpu, address, result);
    cpu.registers.inc_pc(2);
    16
}

/// RES     b, r
/// 11 001 011
/// 10 bbb rrr
fn res_b_r(ext_opcode: u8, pc: u16, cpu: &mut CPU) -> u8 {
    let register = RegisterR::new(ext_opcode & 0b111);
    let value = cpu.registers.read_r(register);
    let bit = (ext_opcode >> 3) & 0b111;
    debug!("{:#06X}: {:#04X} | RES  {:?}, {:?}({:#010b})", pc, ext_opcode, bit, register, value);

    let result = bit_op::clear_bit(value, bit);
    cpu.registers.write_r(register, result);
    cpu.registers.inc_pc(2);
    8
}

/// RES     b, (HL)
/// 11 001 011
/// 10 bbb 110
fn res_b_mhl(ext_opcode: u8, pc: u16, cpu: &mut CPU) -> u8 {
    let address = cpu.registers.hl();
    let value = memory_op::read_memory(cpu, address);
    let bit = (ext_opcode >> 3) & 0b111;
    debug!(
        "{:#06X}: {:#04X} | RES  {:?}, [{:#06x}]({:#010b})",
        pc, ext_opcode, bit, address, value
    );

    let result = bit_op::clear_bit(value, bit);
    memory_op::write_memory(cpu, address, result);
    cpu.registers.inc_pc(2);
    16
}

// --------------- //
// Jump Operations //
// --------------- //

/// JP      nn
/// 11 000 011
/// nnnnnnnn
/// nnnnnnnn
fn jp_nn(opcode: u8, pc: u16, cpu: &mut CPU) -> u8 {
    let address = memory_op::read_memory_following_u16(cpu, pc);
    debug!("{:#06X}: {:#04X} | JP   {:#06X}", pc, opcode, address);
    cpu.registers.set_pc(address);
    16
}

/// JP      cc, nn
/// 11 0cc 011
/// nnnnnnnn
/// nnnnnnnn
fn jp_cc_nn(opcode: u8, pc: u16, cpu: &mut CPU) -> u8 {
    let condition = Condition::new((opcode >> 3) & 0b11);
    let address = memory_op::read_memory_following_u16(cpu, pc);
    if cpu.registers.check_condition(condition) {
        debug!("{:#06X}: {:#04X} | JP   {:?}, {:#06X} ||| (jp)", pc, opcode, condition, address);
        cpu.registers.set_pc(address);
        16
    } else {
        debug!("{:#06X}: {:#04X} | JP   {:?}, {:#06X}  ||| (skip)", pc, opcode, condition, address);
        cpu.registers.inc_pc(3);
        12
    }
}

/// JR      e
/// 00 011 000
/// eeeeeeee
fn jr_e(opcode: u8, pc: u16, cpu: &mut CPU) -> u8 {
    let value = memory_op::read_memory_following_u8(cpu, pc);
    debug!("{:#06X}: {:#04X} | JR   {:?}", pc, opcode, value as i8);
    let pc = add_signed(pc, value);
    cpu.registers.set_pc(pc.wrapping_add(2));
    12
}

/// JR      cc, e
/// 00 1cc 000
/// eeeeeeee
fn jr_cc_e(opcode: u8, pc: u16, cpu: &mut CPU) -> u8 {
    let condition = Condition::new((opcode >> 3) & 0b11);
    let value = memory_op::read_memory_following_u8(cpu, pc);
    if cpu.registers.check_condition(condition) {
        debug!("{:#06X}: {:#04X} | JR   {:?}, {:?} ||| (jp)", pc, opcode, condition, value as i8);
        let pc = add_signed(pc, value);
        cpu.registers.set_pc(pc.wrapping_add(2));
        12
    } else {
        debug!("{:#06X}: {:#04X} | JR   {:?}, {:?} ||| (skip)", pc, opcode, condition, value as i8);
        cpu.registers.inc_pc(2);
        8
    }
}

/// JP      (HL)
/// 11 101 001
fn jp_mhl(opcode: u8, pc: u16, cpu: &mut CPU) -> u8 {
    let address = cpu.registers.hl();
    debug!("{:#06X}: {:#04X} | JP   {:?}({:#06X})", pc, opcode, RegisterDD::HL, address);
    cpu.registers.set_pc(address);
    4
}

// ---------------------------- //
// Call and Return Instructions //
// ---------------------------- //

/// CALL    nn
/// 11 001 101
/// nnnnnnnn
/// nnnnnnnn
fn call_nn(opcode: u8, pc: u16, cpu: &mut CPU) -> u8 {
    let address = memory_op::read_memory_following_u16(cpu, pc);
    let mut sp = cpu.registers.sp();
    debug!("{:#06X}: {:#04X} | CALL {:#06x}", pc, opcode, address);
    memory_op::push_u16_stack(cpu, pc.wrapping_add(3), sp);
    sp -= 2;
    cpu.registers.set_sp(sp);
    cpu.registers.set_pc(address);
    24
}

/// CALL    cc, nn
/// 11 0cc 100
/// nnnnnnnn
/// nnnnnnnn
fn call_c_nn(opcode: u8, pc: u16, cpu: &mut CPU) -> u8 {
    let condition = Condition::new((opcode >> 3) & 0b11);
    let address = memory_op::read_memory_following_u16(cpu, pc);
    if cpu.registers.check_condition(condition) {
        debug!("{:#06X}: {:#04X} | CALL {:?}, {:#06x} ||| (jp)", pc, opcode, condition, address);
        let mut sp = cpu.registers.sp();
        memory_op::push_u16_stack(cpu, pc.wrapping_add(3), sp);
        sp -= 2;
        cpu.registers.set_sp(sp);
        cpu.registers.set_pc(address);
        24
    } else {
        debug!("{:#06X}: {:#04X} | CALL {:#06x} ||| (skip)", pc, opcode, address);
        cpu.registers.inc_pc(3);
        12
    }
}

/// RET
/// 11 001 001
fn ret(opcode: u8, _: u16, cpu: &mut CPU) -> u8 {
    let mut sp = cpu.registers.sp();
    let pc = memory_op::pop_u16_stack(cpu, sp);
    debug!("{:#06X}: {:#04X} | RET  [{:#06x}]", pc, opcode, pc);
    sp += 2;
    cpu.registers.set_sp(sp);
    cpu.registers.set_pc(pc);
    16
}

/// RET
/// 11 001 001
fn reti(opcode: u8, _: u16, cpu: &mut CPU) -> u8 {
    let mut sp = cpu.registers.sp();
    let pc = memory_op::pop_u16_stack(cpu, sp);
    debug!("{:#06X}: {:#04X} | RETI [{:#06x}]", pc, opcode, pc);
    sp += 2;
    cpu.registers.set_sp(sp);
    cpu.registers.set_pc(pc);
    cpu.interrupt.master_enable = true;
    16
}

/// RET     cc
/// 11 0cc 000
fn ret_c(opcode: u8, pc: u16, cpu: &mut CPU) -> u8 {
    let mut sp = cpu.registers.sp();
    let condition = Condition::new((opcode >> 3) & 0b11);
    if cpu.registers.check_condition(condition) {
        let pc = memory_op::pop_u16_stack(cpu, sp);
        debug!("{:#06X}: {:#04X} | RET {:?}, [{:#06x}] ||| (ret)", pc, opcode, condition, pc);
        sp = sp.wrapping_add(2);
        cpu.registers.set_sp(sp);
        cpu.registers.set_pc(pc);
        20
    } else {
        debug!("{:#06X}: {:#04X} | RET {:?}, [{:#06x}] ||| (skip)", pc, opcode, condition, pc);
        cpu.registers.inc_pc(1);
        8
    }
}

/// RST     t
/// 11 ttt 111
fn rst_t(opcode: u8, pc: u16, cpu: &mut CPU) -> u8 {
    let operand = (opcode >> 3) & 0b111;
    let address = match operand {
        0 => 0x0000,
        1 => 0x0008,
        2 => 0x0010,
        3 => 0x0018,
        4 => 0x0020,
        5 => 0x0028,
        6 => 0x0030,
        7 => 0x0038,
        _ => panic!("unsupported operand for RST: {}", operand),
    };
    let mut sp = cpu.registers.sp();
    debug!("{:#06X}: {:#04X} | RST {:#06x}", pc, opcode, address);
    memory_op::push_u16_stack(cpu, pc.wrapping_add(1), sp);
    sp = sp.wrapping_sub(2);
    cpu.registers.set_sp(sp);
    cpu.registers.set_pc(address);
    16
}


// ------------------------------------------------------------------ //
// General-Purpose Arithmetic Operations and CPU Control Instructions //
// ------------------------------------------------------------------ //

/// DAA
/// 00 100 111
fn daa(opcode: u8, pc: u16, cpu: &mut CPU) -> u8 {
    let flag_n = cpu.registers.flag_n();
    let mut flag_cy = cpu.registers.flag_cy();
    let mut flag_h = cpu.registers.flag_h();
    let mut register_a = cpu.registers.a();
    debug!("{:#06X}: {:#04X} | DAA ({:#010b})", pc, opcode, register_a);
    if flag_n == 0 {
        if flag_cy == 1 || register_a > 0x99 {
            register_a = register_a.wrapping_add(0x60);
            flag_cy = 1;
        }
        if flag_h == 1 || (register_a & 0x0f) > 0x09 {
            register_a = register_a.wrapping_add(0x6);
        }
    } else {
        if flag_cy == 1 {
            register_a = register_a.wrapping_sub(0x60);
        }
        if flag_h == 1 {
            register_a = register_a.wrapping_sub(0x6);
        }
    }
    let flag_z = if register_a == 0 { 1 } else { 0 };
    flag_h = 0;

    cpu.registers.set_flags(flag_z, flag_n, flag_h, flag_cy);
    cpu.registers.set_a(register_a);
    cpu.registers.inc_pc(1);
    4
}

/// CPL
/// 00 101 111
fn cpl(opcode: u8, pc: u16, cpu: &mut CPU) -> u8 {
    let register = RegisterR::A;
    let value = cpu.registers.read_r(register);
    debug!("{:#06X}: {:#04X} | CPL {:?}({:#010b})", pc, opcode, register, value);
    let result = !value;
    let mut flags = cpu.registers.f();
    flags = bit_op::set_bit(flags, 5);
    flags = bit_op::set_bit(flags, 6);
    cpu.registers.set_f(flags);
    cpu.registers.write_r(register, result);
    cpu.registers.inc_pc(1);
    4
}

/// SCF
/// 00 110 111
fn scf(opcode: u8, pc: u16, cpu: &mut CPU) -> u8 {
    let mut flags = cpu.registers.f();
    debug!("{:#06X}: {:#04X} | SCF", pc, opcode);
    flags = bit_op::set_bit(flags, 4);
    flags = bit_op::clear_bit(flags, 5);
    flags = bit_op::clear_bit(flags, 6);
    cpu.registers.set_f(flags);
    cpu.registers.inc_pc(1);
    4
}

/// CCF
/// 00 111 111
fn ccf(opcode: u8, pc: u16, cpu: &mut CPU) -> u8 {
    let mut flags = cpu.registers.f();
    debug!("{:#06X}: {:#04X} | SCF", pc, opcode);

    flags = bit_op::toggle_bit(flags, 4);
    flags = bit_op::clear_bit(flags, 5);
    flags = bit_op::clear_bit(flags, 6);
    cpu.registers.set_f(flags);
    cpu.registers.inc_pc(1);
    4
}

/// NOP
/// 00 000 000
fn nop(opcode: u8, pc: u16, cpu: &mut CPU) -> u8 {
    debug!("{:#06X}: {:#04X} | NOP", pc, opcode);
    cpu.registers.inc_pc(1);
    4
}

/// HALT
/// 01 110 110
fn halt(opcode: u8, pc: u16, cpu: &mut CPU) -> u8 {
    debug!("{:#06X}: {:#04X} | HALT", pc, opcode);
    //unimplemented!();
    cpu.registers.inc_pc(1);
    4
}

/// STOP
/// 00 010 000
/// 00 000 000
fn stop(opcode: u8, pc: u16, _: &mut CPU) -> u8 {
    debug!("{:#06X}: {:#04X} | STOP", pc, opcode);
    unimplemented!();
    // 4
}

/// EI
/// 11 111 011
fn ei(opcode: u8, pc: u16, cpu: &mut CPU) -> u8 {
    debug!("{:#06X}: {:#04X} | EI", pc, opcode);
    cpu.interrupt.master_enable = true;
    cpu.registers.inc_pc(1);
    4
}

/// DI
/// 11 110 011
fn di(opcode: u8, pc: u16, cpu: &mut CPU) -> u8 {
    debug!("{:#06X}: {:#04X} | DI", pc, opcode);
    cpu.interrupt.master_enable = false;
    cpu.registers.inc_pc(1);
    4
}

// ---------------- //
// Helper Functions //
// ---------------- //

fn add_signed(a: u16, b: u8) -> u16 {
    ((a as i16).wrapping_add(i16::from(b as i8))) as u16
}
