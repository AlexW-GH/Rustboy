const ROM_BANK_SIZE: usize = 0x4000;

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

    pub fn data(&self, address: u16) -> u8{
        *self.data.get(address as usize).unwrap()
    }
    pub fn bank(&self, index: usize) -> &[u8]{
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