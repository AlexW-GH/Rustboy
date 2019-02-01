pub mod bit_op{
    pub fn set_bit(number: u8, bit: u8) -> u8{
        if bit > 7 {
            panic!("invalid bit (>7)");
        }
        number | (0b1 << bit)
    }

    pub fn clear_bit(number: u8, bit: u8) -> u8{
        if bit > 7 {
            panic!("invalid bit (>7)");
        }
        number & !(0b1 << bit)
    }

    pub fn toggle_bit(number: u8, bit: u8) -> u8{
        if bit > 7 {
            panic!("invalid bit (>7)");
        }
        number ^ 0b1 << bit
    }

    pub fn change_bit_to(number: u8, bit: u8, value: u8) -> u8{
        if bit > 7 {
            panic!("invalid bit (>7)");
        }
        if value > 1 {
            panic!("bit can only be set to 0 or 1 {}", value);
        }
        number & !(1 << bit) | (value << bit)
    }
}

pub mod memory_op{
    use crate::mem::memory::MapsMemory;

    pub fn write_memory(memory: &mut dyn MapsMemory, address: u16, value: u8) {
        memory.write(address, value).unwrap()
    }

    pub fn read_memory(memory: &dyn MapsMemory, address: u16) -> u8 {
        memory.read(address).unwrap()
    }

    pub fn read_memory_following_u8(memory: &dyn MapsMemory, address: u16) -> u8 {
        memory.read(address +1).unwrap()
    }

    pub fn read_memory_following_u16(memory: &dyn MapsMemory, address: u16) -> u16 {
        (u16::from(memory.read(address + 2).unwrap()) << 8 ) + u16::from(memory.read(address +1).unwrap())
    }

    pub fn push_u16_stack(memory: &mut dyn MapsMemory, value: u16, sp: u16){
        memory.write(sp-1, ((value>>8) & 0xFF) as u8 ).unwrap();
        memory.write(sp-2, (value & 0xFF) as u8 ).unwrap();
    }

    pub fn pop_u16_stack(memory: &mut dyn MapsMemory, sp: u16) -> u16{
        let val_lo = memory.read(sp).unwrap();
        let val_hi = memory.read(sp + 1).unwrap();
        (u16::from(val_hi) << 8) + u16::from(val_lo)
    }
}
