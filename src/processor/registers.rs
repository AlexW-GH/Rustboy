use std::fmt;
use util::bit_op;

#[derive(Clone)]
pub struct Registers{
    af: AF,
    bc: BC,
    de: DE,
    hl: HL,
    sp: u16,
    pc: u16
}

impl fmt::Debug for Registers {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{{af: {:?}, bc: {:?}, de: {:?}, hl: {:?}, sp: {:#06X}, pc: {:#06X}}}", self.af, self.bc, self.de, self.hl, self.sp, self.pc)
    }
}

#[derive(Clone, Copy)]
union AF {
    single: AFSingle,
    both: u16
}

impl fmt::Debug for AF {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        unsafe { write!(f, "{:#06X} (a: {:#04X}, f: {:#04X})", self.both, self.single.a, self.single.f) }
    }
}

impl AF {
    fn new(boot_sequence: bool) -> AF {
        AF {
            both: if boot_sequence {0} else {0x01b0}
        }
    }
}

#[derive(Clone, Copy)]
union BC {
    single: BCSingle,
    both: u16
}

impl fmt::Debug for BC {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        unsafe { write!(f, "{:#06X} (b: {:#04X}, c: {:#04X})", self.both, self.single.b, self.single.c) }
    }
}

impl BC {
    fn new(boot_sequence: bool) -> BC {
        BC {
            both: if boot_sequence {0} else {0x0013}
        }
    }
}

#[derive(Clone, Copy)]
union DE {
    single: DESingle,
    both: u16
}

impl fmt::Debug for DE {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        unsafe { write!(f, "{:#06X} (d: {:#04X}, e: {:#04X})", self.both, self.single.d, self.single.e) }
    }
}

impl DE {
    fn new(boot_sequence: bool) -> DE {
        DE {
            both: if boot_sequence {0} else {0x00d8}
        }
    }
}

#[derive(Clone, Copy)]
union HL {
    single: HLSingle,
    both: u16
}

impl fmt::Debug for HL {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        unsafe { write!(f, "{:#06X} (h: {:#04X}, l: {:#04X})", self.both, self.single.h, self.single.l) }
    }
}

impl HL {
    fn new(boot_sequence: bool) -> HL {
        HL {
            both: if boot_sequence {0} else {0x014D}
        }
    }
}

#[derive(Debug, Copy, Clone)]
struct AFSingle {
    f: u8,
    a: u8
}

#[derive(Debug, Copy, Clone)]
struct BCSingle {
    c: u8,
    b: u8
}

#[derive(Debug, Copy, Clone)]
struct DESingle {
    e: u8,
    d: u8
}

#[derive(Debug, Copy, Clone)]
struct HLSingle {
    l: u8,
    h: u8
}

impl Registers {

    pub fn new(boot_sequence: bool) -> Registers{
        Registers{
            af: AF::new(boot_sequence),
            bc: BC::new(boot_sequence),
            de: DE::new(boot_sequence),
            hl: HL::new(boot_sequence),
            sp: if boot_sequence {0x0000} else {0xFFFE},
            pc: if boot_sequence {0x0000} else {0x0100}
        }
    }

    pub fn read_r(&self, target: RegisterR) -> u8 {
        match target {
            RegisterR::A => self.a(),
            RegisterR::B => self.b(),
            RegisterR::C => self.c(),
            RegisterR::D => self.d(),
            RegisterR::E => self.e(),
            RegisterR::H => self.h(),
            RegisterR::L => self.l(),
        }
    }

    pub fn write_r(&mut self, target: RegisterR, value: u8) {
        match target {
            RegisterR::A => self.set_a(value),
            RegisterR::B => self.set_b(value),
            RegisterR::C => self.set_c(value),
            RegisterR::D => self.set_d(value),
            RegisterR::E => self.set_e(value),
            RegisterR::H => self.set_h(value),
            RegisterR::L => self.set_l(value),
        }
    }

    pub fn write_dd(&mut self, target: RegisterDD, value: u16) {
        match target {
            RegisterDD::BC => self.set_bc(value),
            RegisterDD::DE => self.set_de(value),
            RegisterDD::HL => self.set_hl(value),
            RegisterDD::SP => self.set_sp(value),
        }
    }

    pub fn read_qq(&self, target: RegisterQQ) -> u16 {
        match target {
            RegisterQQ::AF => self.af(),
            RegisterQQ::BC => self.bc(),
            RegisterQQ::DE => self.de(),
            RegisterQQ::HL => self.hl(),
        }
    }

    pub fn write_qq(&mut self, target: RegisterQQ, value: u16) {
        match target {
            RegisterQQ::AF => self.set_af(value),
            RegisterQQ::BC => self.set_bc(value),
            RegisterQQ::DE => self.set_de(value),
            RegisterQQ::HL => self.set_hl(value),
        }
    }

    pub fn read_ss(&self, target: RegisterSS) -> u16 {
        match target {
            RegisterSS::BC => self.bc(),
            RegisterSS::DE => self.de(),
            RegisterSS::HL => self.hl(),
            RegisterSS::SP => self.sp(),
        }
    }

    pub fn write_ss(&mut self, target: RegisterSS, value: u16) {
        match target {
            RegisterSS::BC => self.set_bc(value),
            RegisterSS::DE => self.set_de(value),
            RegisterSS::HL => self.set_hl(value),
            RegisterSS::SP => self.set_sp(value),
        }
    }

    pub fn a(&self) -> u8{
        unsafe { self.af.single.a }
    }

    pub fn b(&self) -> u8{
        unsafe { self.bc.single.b }
    }

    pub fn c(&self) -> u8{
        unsafe { self.bc.single.c }
    }

    pub fn d(&self) -> u8{
        unsafe { self.de.single.d }
    }

    pub fn e(&self) -> u8{
        unsafe { self.de.single.e }
    }

    pub fn f(&self) -> u8{
        unsafe { self.af.single.f }
    }

    pub fn h(&self) -> u8{
        unsafe { self.hl.single.h }
    }

    pub fn l(&self) -> u8{
        unsafe { self.hl.single.l }
    }

    pub fn af(&self) -> u16{
        unsafe { self.af.both }
    }

    pub fn bc(&self) -> u16{
        unsafe { self.bc.both }
    }

    pub fn de(&self) -> u16{
        unsafe { self.de.both }
    }

    pub fn hl(&self) -> u16{
        unsafe { self.hl.both }
    }

    pub fn sp(&self) -> u16{
        self.sp
    }

    pub fn pc(&self) -> u16{
        self.pc
    }

    pub fn set_a(&mut self, value: u8){
        unsafe { self.af.single.a = value };
    }

    pub fn set_b(&mut self, value: u8){
        unsafe { self.bc.single.b = value };
    }

    pub fn set_c(&mut self, value: u8){
        unsafe { self.bc.single.c = value };
    }

    pub fn set_d(&mut self, value: u8){
        unsafe { self.de.single.d = value };
    }

    pub fn set_e(&mut self, value: u8){
        unsafe { self.de.single.e = value };
    }

    pub fn set_f(&mut self, value: u8){
        unsafe { self.af.single.f = value };
    }

    pub fn set_h(&mut self, value: u8){
        unsafe { self.hl.single.h = value };
    }

    pub fn set_l(&mut self, value: u8){
        unsafe { self.hl.single.l = value };
    }

    pub fn set_af(&mut self, value: u16){
        self.af.both = value;
    }

    pub fn set_bc(&mut self, value: u16){
        self.bc.both = value;
    }

    pub fn set_de(&mut self, value: u16){
        self.de.both = value;
    }

    pub fn set_hl(&mut self, value: u16){
        self.hl.both = value;
    }

    pub fn set_sp(&mut self, value: u16){
        self.sp = value;
    }

    pub fn set_pc(&mut self, value: u16){
        self.pc = value;
    }

    pub fn inc_pc(&mut self, value: u16){
        self.pc = self.pc.wrapping_add(value);
    }

    pub fn check_condition(&self, condition: Condition) -> bool{
        match condition{
            Condition::C  => (self.f() >> 4) & 0b1 == 0b1,
            Condition::NC => (self.f() >> 4) & 0b1 == 0b0,
            Condition::NZ => (self.f() >> 7) & 0b1 == 0b0,
            Condition::Z  => (self.f() >> 7) & 0b1 == 0b1,
        }
    }

    pub fn flag_cy(&self) -> u8{
        (self.f() >> 4) & 1
    }
    pub fn flag_h(&self) -> u8{
        (self.f() >> 5) & 1
    }
    pub fn flag_n(&self) -> u8{
        (self.f() >> 6) & 1
    }
    pub fn flag_z(&self) -> u8{
        (self.f() >> 7) & 1
    }

    pub fn set_flags(&mut self, z: u8, n: u8, h: u8, cy: u8){
        let mut flags = self.f();
        flags = bit_op::change_bit_to(flags, 7, z);
        flags = bit_op::change_bit_to(flags, 6, n);
        flags = bit_op::change_bit_to(flags, 5, h);
        flags = bit_op::change_bit_to(flags, 4, cy);
        self.set_f(flags);
    }

    pub fn set_flags_add(&mut self, operand1: u8, operand2: u8,
                              z: FlagCalculationStatus, n: FlagCalculationStatus,
                              h: FlagCalculationStatus, cy: FlagCalculationStatus){
        let mut flags = self.f();
        flags = Registers::calculate_flag_z(FlagCalculationOperation::Add, operand1, operand2, z, flags);
        flags = Registers::calculate_flag_n(n, flags);
        flags = Registers::calculate_flag_h(FlagCalculationOperation::Add, operand1, operand2, h, flags);
        flags = Registers::calculate_flag_cy(FlagCalculationOperation::Add, operand1, operand2, cy, flags);
        self.set_f(flags);
    }

    pub fn set_flags_add_with_carry(&mut self, operand1: u8, operand2: u8,
                         z: FlagCalculationStatus, n: FlagCalculationStatus,
                         h: FlagCalculationStatus, cy: FlagCalculationStatus){
        let mut flags = self.f();
        flags = Registers::calculate_flag_z_with_carry(FlagCalculationOperation::Add, operand1, operand2, z, flags);
        flags = Registers::calculate_flag_n(n, flags);
        flags = Registers::calculate_flag_h_with_carry(FlagCalculationOperation::Add, operand1, operand2, h, flags);
        flags = Registers::calculate_flag_cy_with_carry(FlagCalculationOperation::Add, operand1, operand2, cy, flags);
        self.set_f(flags);
    }

    pub fn set_flags_sub(&mut self, operand1: u8, operand2: u8,
                              z: FlagCalculationStatus, n: FlagCalculationStatus,
                              h: FlagCalculationStatus, cy: FlagCalculationStatus){
        let mut flags = self.f();
        flags = Registers::calculate_flag_z(FlagCalculationOperation::Sub, operand1, operand2, z, flags);
        flags = Registers::calculate_flag_n(n, flags);
        flags = Registers::calculate_flag_h(FlagCalculationOperation::Sub, operand1, operand2, h, flags);
        flags = Registers::calculate_flag_cy(FlagCalculationOperation::Sub, operand1, operand2, cy, flags);
        self.set_f(flags);
    }

    pub fn set_flags_sub_with_carry(&mut self, operand1: u8, operand2: u8,
                         z: FlagCalculationStatus, n: FlagCalculationStatus,
                         h: FlagCalculationStatus, cy: FlagCalculationStatus){
        let mut flags = self.f();
        flags = Registers::calculate_flag_z_with_carry(FlagCalculationOperation::Sub, operand1, operand2, z, flags);
        flags = Registers::calculate_flag_n(n, flags);
        flags = Registers::calculate_flag_h_with_carry(FlagCalculationOperation::Sub, operand1, operand2, h, flags);
        flags = Registers::calculate_flag_cy_with_carry(FlagCalculationOperation::Sub, operand1, operand2, cy, flags);
        self.set_f(flags);
    }

    pub fn set_flags_add_u16(&mut self, operand1: u16, operand2: u16,
                         z: FlagCalculationStatus, n: FlagCalculationStatus,
                         h: FlagCalculationStatus, cy: FlagCalculationStatus){
        let mut flags = self.f();
        flags = Registers::calculate_flag_z_u16(FlagCalculationOperation::Add, operand1, operand2, z, flags);
        flags = Registers::calculate_flag_n(n, flags);
        flags = Registers::calculate_flag_h_u16(FlagCalculationOperation::Add, operand1, operand2, h, flags);
        flags = Registers::calculate_flag_cy_u16(FlagCalculationOperation::Add, operand1, operand2, cy, flags);
        self.set_f(flags);
    }

    pub fn set_flags_sub_u16(&mut self, operand1: u16, operand2: u16,
                             z: FlagCalculationStatus, n: FlagCalculationStatus,
                             h: FlagCalculationStatus, cy: FlagCalculationStatus){
        let mut flags = self.f();
        flags = Registers::calculate_flag_z_u16(FlagCalculationOperation::Sub, operand1, operand2, z, flags);
        flags = Registers::calculate_flag_n(n, flags);
        flags = Registers::calculate_flag_h_u16(FlagCalculationOperation::Sub, operand1, operand2, h, flags);
        flags = Registers::calculate_flag_cy_u16(FlagCalculationOperation::Sub, operand1, operand2, cy, flags);
        self.set_f(flags);
    }

    fn calculate_flag_z(operation: FlagCalculationOperation, operand1: u8, operand2: u8, z: FlagCalculationStatus, flags: u8) -> u8 {
        match z {
            FlagCalculationStatus::Set => bit_op::set_bit(flags, 7),
            FlagCalculationStatus::Clear => bit_op::set_bit(flags, 7),
            FlagCalculationStatus::Ignore => flags,
            FlagCalculationStatus::Calculate => {
                let result = match operation{
                    FlagCalculationOperation::Add => operand1.overflowing_add(operand2),
                    FlagCalculationOperation::Sub => operand1.overflowing_sub(operand2),
                };
                if result.0 == 0 {
                    bit_op::set_bit(flags, 7)
                } else {
                    bit_op::clear_bit(flags, 7)
                }
            },
        }
    }

    fn calculate_flag_n(n: FlagCalculationStatus, flags: u8) -> u8 {
        match n {
            FlagCalculationStatus::Set => bit_op::set_bit(flags, 6),
            FlagCalculationStatus::Clear => bit_op::clear_bit(flags, 6),
            FlagCalculationStatus::Ignore => flags,
            FlagCalculationStatus::Calculate => bit_op::set_bit(flags, 6),
        }
    }

    fn calculate_flag_h(operation: FlagCalculationOperation, operand1: u8, operand2: u8, h: FlagCalculationStatus, flags: u8) -> u8 {
        match h {
            FlagCalculationStatus::Set => bit_op::set_bit(flags, 5),
            FlagCalculationStatus::Clear => bit_op::clear_bit(flags, 5),
            FlagCalculationStatus::Ignore => flags,
            FlagCalculationStatus::Calculate => {
                match operation {
                    FlagCalculationOperation::Add => {
                        let result = ((operand1 & 0xf) + (operand2 & 0xf)) & 0x10;
                        if result == 0x10 {
                            bit_op::set_bit(flags, 5)
                        } else {
                            bit_op::clear_bit(flags, 5)
                        }
                    },
                    FlagCalculationOperation::Sub => {
                        let mut result: i16 = operand1 as i16 & 0xF;
                        result -= (operand2 & 0xF) as i16;

                        if result < 0 {
                            bit_op::set_bit(flags, 5)
                        } else {
                            bit_op::clear_bit(flags, 5)
                        }
                    }
                }

            },
        }
    }

    fn calculate_flag_cy(operation: FlagCalculationOperation, operand1: u8, operand2: u8, h: FlagCalculationStatus, flags: u8) -> u8 {
        match h {
            FlagCalculationStatus::Set => bit_op::set_bit(flags, 4),
            FlagCalculationStatus::Clear => bit_op::clear_bit(flags, 4),
            FlagCalculationStatus::Ignore => flags,
            FlagCalculationStatus::Calculate => {
                match operation{
                    FlagCalculationOperation::Add => {
                        let result = operand1.checked_add(operand2);
                        if result == None {
                            bit_op::set_bit(flags, 4)
                        } else {
                            bit_op::clear_bit(flags, 4)
                        }
                    },
                    FlagCalculationOperation::Sub => {
                        if operand1 < operand2 {
                            bit_op::set_bit(flags, 4)
                        } else {
                            bit_op::clear_bit(flags, 4)
                        }
                    }
                }

            },
        }
    }

    fn calculate_flag_z_u16(operation: FlagCalculationOperation, operand1: u16, operand2: u16, z: FlagCalculationStatus, flags: u8) -> u8 {
        match z {
            FlagCalculationStatus::Set => bit_op::set_bit(flags, 7),
            FlagCalculationStatus::Clear => bit_op::clear_bit(flags, 7),
            FlagCalculationStatus::Ignore => flags,
            FlagCalculationStatus::Calculate => {
                let result = match operation{
                    FlagCalculationOperation::Add => operand1.overflowing_add(operand2),
                    FlagCalculationOperation::Sub => operand1.overflowing_sub(operand2),
                };
                if result.0 == 0 {
                    bit_op::set_bit(flags, 7)
                } else {
                    bit_op::clear_bit(flags, 7)
                }
            },
        }
    }

    fn calculate_flag_h_u16(operation: FlagCalculationOperation, operand1: u16, operand2: u16, h: FlagCalculationStatus, flags: u8) -> u8 {
        match h {
            FlagCalculationStatus::Set => bit_op::set_bit(flags, 5),
            FlagCalculationStatus::Clear => bit_op::clear_bit(flags, 5),
            FlagCalculationStatus::Ignore => flags,
            FlagCalculationStatus::Calculate => {
                match operation {
                    FlagCalculationOperation::Add => {
                        let mut result = operand1 & 0xFFF;
                        result += operand2 & 0xFFF;
                        if result >= 0x1000 {
                            bit_op::set_bit(flags, 5)
                        } else {
                            bit_op::clear_bit(flags, 5)
                        }
                    },
                    FlagCalculationOperation::Sub => {
                        let mut result: i32 = operand1 as i32 & 0xFFF;
                        result -= operand2 as i32 & 0xFFF;

                        if result < 0 {
                            bit_op::set_bit(flags, 5)
                        } else {
                            bit_op::clear_bit(flags, 5)
                        }
                    }
                }

            },
        }
    }

    fn calculate_flag_cy_u16(operation: FlagCalculationOperation, operand1: u16, operand2: u16, h: FlagCalculationStatus, flags: u8) -> u8 {
        match h {
            FlagCalculationStatus::Set => bit_op::set_bit(flags, 4),
            FlagCalculationStatus::Clear => bit_op::clear_bit(flags, 4),
            FlagCalculationStatus::Ignore => flags,
            FlagCalculationStatus::Calculate => {
                match operation{
                    FlagCalculationOperation::Add => {
                        let result = operand1.checked_add(operand2);
                        if result == None {
                            bit_op::set_bit(flags, 4)
                        } else {
                            bit_op::clear_bit(flags, 4)
                        }
                    },
                    FlagCalculationOperation::Sub => {
                        if operand1 < operand2 {
                            bit_op::set_bit(flags, 4)
                        } else {
                            bit_op::clear_bit(flags, 4)
                        }
                    }
                }

            },
        }
    }

    fn calculate_flag_z_with_carry(operation: FlagCalculationOperation, operand1: u8, operand2: u8, z: FlagCalculationStatus, flags: u8) -> u8 {
        let flag_z = (flags >> 7) & 1;
        match z {
            FlagCalculationStatus::Set => bit_op::set_bit(flags, 7),
            FlagCalculationStatus::Clear => bit_op::set_bit(flags, 7),
            FlagCalculationStatus::Ignore => flags,
            FlagCalculationStatus::Calculate => {
                let result = match operation{
                    FlagCalculationOperation::Add => operand1.wrapping_add(operand2).wrapping_add(flag_z),
                    FlagCalculationOperation::Sub => operand1.wrapping_sub(operand2).wrapping_sub(flag_z),
                };
                if result == 0 {
                    bit_op::set_bit(flags, 7)
                } else {
                    bit_op::clear_bit(flags, 7)
                }
            },
        }
    }

    fn calculate_flag_h_with_carry(operation: FlagCalculationOperation, operand1: u8, operand2: u8, h: FlagCalculationStatus, flags: u8) -> u8 {
        let flag_h = (flags >> 5) & 1;
        match h {
            FlagCalculationStatus::Set => bit_op::set_bit(flags, 5),
            FlagCalculationStatus::Clear => bit_op::clear_bit(flags, 5),
            FlagCalculationStatus::Ignore => flags,
            FlagCalculationStatus::Calculate => {
                match operation {
                    FlagCalculationOperation::Add => {
                        let result = ((operand1 & 0xf) + (operand2 & 0xf) + flag_h) & 0x10;
                        if result == 0x10 {
                            bit_op::set_bit(flags, 5)
                        } else {
                            bit_op::clear_bit(flags, 5)
                        }
                    },
                    FlagCalculationOperation::Sub => {
                        let mut result: i16 = operand1 as i16 & 0xF;
                        result -= (operand2 & 0xF) as i16;
                        result -= flag_h as i16;

                        if result < 0 {
                            bit_op::set_bit(flags, 5)
                        } else {
                            bit_op::clear_bit(flags, 5)
                        }
                    }
                }

            },
        }
    }

    fn calculate_flag_cy_with_carry(operation: FlagCalculationOperation, operand1: u8, operand2: u8, h: FlagCalculationStatus, flags: u8) -> u8 {
        let flag_cy = (flags >> 4) & 1;
        match h {
            FlagCalculationStatus::Set => bit_op::set_bit(flags, 4),
            FlagCalculationStatus::Clear => bit_op::clear_bit(flags, 4),
            FlagCalculationStatus::Ignore => flags,
            FlagCalculationStatus::Calculate => {
                match operation{
                    FlagCalculationOperation::Add => {
                        let result = operand1.checked_add(operand2);
                        if result == None {
                            bit_op::set_bit(flags, 4)
                        } else {
                            let result = result.unwrap().checked_add(flag_cy);
                            if result == None {
                                bit_op::set_bit(flags, 4)
                            } else {
                                bit_op::clear_bit(flags, 4)
                            }
                        }
                    },
                    FlagCalculationOperation::Sub => {
                        if operand1 < operand2.wrapping_add(flag_cy) {
                            bit_op::set_bit(flags, 4)
                        } else {
                            bit_op::clear_bit(flags, 4)
                        }
                    }
                }

            },
        }
    }

}

#[derive(Debug, Copy, Clone)]
pub enum FlagCalculationStatus{
    Set,
    Clear,
    Ignore,
    Calculate
}

#[derive(Debug, Copy, Clone)]
pub enum FlagCalculationOperation{
    Add,
    Sub
}

#[derive(Debug, Copy, Clone)]
pub enum RegisterR{
    A,
    B,
    C,
    D,
    E,
    H,
    L
}

impl RegisterR {
    pub fn new(value: u8) -> RegisterR{
        match value{
            0b111 => RegisterR::A,
            0b000 => RegisterR::B,
            0b001 => RegisterR::C,
            0b010 => RegisterR::D,
            0b011 => RegisterR::E,
            0b100 => RegisterR::H,
            0b101 => RegisterR::L,
            _ => unreachable!()
        }
    }
}

#[derive(Debug, Copy, Clone)]
pub enum RegisterDD{
    BC,
    DE,
    HL,
    SP
}

impl RegisterDD {
    pub fn new(value: u8) -> RegisterDD{
        match value{
            0b00 => RegisterDD::BC,
            0b01 => RegisterDD::DE,
            0b10 => RegisterDD::HL,
            0b11 => RegisterDD::SP,
            _ => unreachable!()
        }
    }
}

#[derive(Debug, Copy, Clone)]
pub enum RegisterQQ{
    BC,
    DE,
    HL,
    AF
}

impl RegisterQQ {
    pub fn new(value: u8) -> RegisterQQ{
        match value{
            0b00 => RegisterQQ::BC,
            0b01 => RegisterQQ::DE,
            0b10 => RegisterQQ::HL,
            0b11 => RegisterQQ::AF,
            _ => unreachable!()
        }
    }
}

#[derive(Debug, Copy, Clone)]
pub enum RegisterSS{
    BC,
    DE,
    HL,
    SP
}

impl RegisterSS {
    pub fn new(value: u8) -> RegisterSS{
        match value{
            0b00 => RegisterSS::BC,
            0b01 => RegisterSS::DE,
            0b10 => RegisterSS::HL,
            0b11 => RegisterSS::SP,
            _ => unreachable!()
        }
    }
}

#[derive(Debug, Copy, Clone)]
pub enum Condition{
    NZ,
    Z,
    NC,
    C
}

impl Condition {
    pub fn new(value: u8) -> Condition{
        match value{
            0b00 => Condition::NZ,
            0b01 => Condition::Z,
            0b10 => Condition::NC,
            0b11 => Condition::C,
            _ => unreachable!()
        }
    }
}

#[cfg(test)]
mod tests {
    use processor::registers::Registers;

    #[test]
    fn everything_setup_after_initialization_with_boot_sequence() {
        let registers = Registers::new(true);
        assert_eq!(registers.af(), 0);
        assert_eq!(registers.bc(), 0);
        assert_eq!(registers.de(), 0);
        assert_eq!(registers.hl(), 0);
        assert_eq!(registers.a(), 0);
        assert_eq!(registers.b(), 0);
        assert_eq!(registers.c(), 0);
        assert_eq!(registers.d(), 0);
        assert_eq!(registers.e(), 0);
        assert_eq!(registers.f(), 0);
        assert_eq!(registers.h(), 0);
        assert_eq!(registers.l(), 0);
        assert_eq!(registers.pc(), 0x0);
        assert_eq!(registers.sp(), 0x0);
    }

    #[test]
    fn everything_setup_after_initialization() {
        let registers = Registers::new(false);
        assert_eq!(registers.af(), 0x01B0);
        assert_eq!(registers.bc(), 0x0013);
        assert_eq!(registers.de(), 0x00D8);
        assert_eq!(registers.hl(), 0x014D);
        assert_eq!(registers.a(), 0x01);
        assert_eq!(registers.b(), 0x00);
        assert_eq!(registers.c(), 0x13);
        assert_eq!(registers.d(), 0x00);
        assert_eq!(registers.e(), 0xD8);
        assert_eq!(registers.f(), 0xB0);
        assert_eq!(registers.h(), 0x01);
        assert_eq!(registers.l(), 0x4D);
        assert_eq!(registers.pc(), 0x100);
        assert_eq!(registers.sp(), 0xFFFE);
    }

    #[test]
    fn set_af_correct() {
        let mut registers = Registers::new(false);
        registers.set_af(0xABCD);
        assert_eq!(registers.af(), 0xABCD);
        assert_eq!(registers.a(), 0xAB);
        assert_eq!(registers.f(), 0xCD);
    }

    #[test]
    fn set_af_singles_correct() {
        let mut registers = Registers::new(false);
        registers.set_a(0xAB);
        registers.set_f(0xCD);
        assert_eq!(registers.af(), 0xABCD);
        assert_eq!(registers.a(), 0xAB);
        assert_eq!(registers.f(), 0xCD);
    }

    #[test]
    fn set_a_singles_correct() {
        let mut registers = Registers::new(false);
        registers.set_a(0xAB);
        assert_eq!(registers.af(), 0xABB0);
        assert_eq!(registers.a(), 0xAB);
        assert_eq!(registers.f(), 0xB0);
    }

    #[test]
    fn set_f_singles_correct() {
        let mut registers = Registers::new(false);
        registers.set_f(0xCD);
        assert_eq!(registers.af(), 0x01CD);
        assert_eq!(registers.a(), 0x01);
        assert_eq!(registers.f(), 0xCD);
    }

    #[test]
    fn set_bc_correct() {
        let mut registers = Registers::new(false);
        registers.set_bc(0xABCD);
        assert_eq!(registers.bc(), 0xABCD);
        assert_eq!(registers.b(), 0xAB);
        assert_eq!(registers.c(), 0xCD);
    }

    #[test]
    fn set_bc_singles_correct() {
        let mut registers = Registers::new(false);
        registers.set_b(0xAB);
        registers.set_c(0xCD);
        assert_eq!(registers.bc(), 0xABCD);
        assert_eq!(registers.b(), 0xAB);
        assert_eq!(registers.c(), 0xCD);
    }

    #[test]
    fn set_b_singles_correct() {
        let mut registers = Registers::new(false);
        registers.set_b(0xAB);
        assert_eq!(registers.bc(), 0xAB13);
        assert_eq!(registers.b(), 0xAB);
        assert_eq!(registers.c(), 0x13);
    }

    #[test]
    fn set_c_singles_correct() {
        let mut registers = Registers::new(false);
        registers.set_c(0xCD);
        assert_eq!(registers.bc(), 0x00CD);
        assert_eq!(registers.b(), 0x00);
        assert_eq!(registers.c(), 0xCD);
    }

    #[test]
    fn set_de_correct() {
        let mut registers = Registers::new(false);
        registers.set_de(0xABCD);
        assert_eq!(registers.de(), 0xABCD);
        assert_eq!(registers.d(), 0xAB);
        assert_eq!(registers.e(), 0xCD);
    }

    #[test]
    fn set_de_singles_correct() {
        let mut registers = Registers::new(false);
        registers.set_d(0xAB);
        registers.set_e(0xCD);
        assert_eq!(registers.de(), 0xABCD);
        assert_eq!(registers.d(), 0xAB);
        assert_eq!(registers.e(), 0xCD);
    }

    #[test]
    fn set_d_singles_correct() {
        let mut registers = Registers::new(false);
        registers.set_d(0xAB);
        assert_eq!(registers.de(), 0xABD8);
        assert_eq!(registers.d(), 0xAB);
        assert_eq!(registers.e(), 0xD8);
    }

    #[test]
    fn set_e_singles_correct() {
        let mut registers = Registers::new(false);
        registers.set_e(0xCD);
        assert_eq!(registers.de(), 0x00CD);
        assert_eq!(registers.d(), 0x00);
        assert_eq!(registers.e(), 0xCD);
    }

    #[test]
    fn set_hl_correct() {
        let mut registers = Registers::new(false);
        registers.set_hl(0xABCD);
        assert_eq!(registers.hl(), 0xABCD);
        assert_eq!(registers.h(), 0xAB);
        assert_eq!(registers.l(), 0xCD);
    }

    #[test]
    fn set_hl_singles_correct() {
        let mut registers = Registers::new(false);
        registers.set_h(0xAB);
        registers.set_l(0xCD);
        assert_eq!(registers.hl(), 0xABCD);
        assert_eq!(registers.h(), 0xAB);
        assert_eq!(registers.l(), 0xCD);
    }

    #[test]
    fn set_h_singles_correct() {
        let mut registers = Registers::new(false);
        registers.set_h(0xAB);
        assert_eq!(registers.hl(), 0xAB4D);
        assert_eq!(registers.h(), 0xAB);
        assert_eq!(registers.l(), 0x4D);
    }

    #[test]
    fn set_l_singles_correct() {
        let mut registers = Registers::new(false);
        registers.set_l(0xCD);
        assert_eq!(registers.hl(), 0x01CD);
        assert_eq!(registers.h(), 0x01);
        assert_eq!(registers.l(), 0xCD);
    }

    #[test]
    fn set_sp_and_pc() {
        let mut registers = Registers::new(false);
        registers.set_sp(0xABCD);
        registers.set_pc(0x1234);
        assert_eq!(registers.sp(), 0xABCD);
        assert_eq!(registers.pc(), 0x1234);
    }
}