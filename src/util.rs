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
