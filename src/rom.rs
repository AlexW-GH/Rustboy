pub struct ROM {
    data: Vec<u8>
}

impl ROM {
    pub fn new(game: Vec<u8>) -> ROM{
        let mut data = vec![0; 8000];
        for (i, val) in game.iter().enumerate(){
            if i<8000 {
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
        let start_slice = if self.data.len() >= index*4000 {
            index*4000
        } else {
            panic!("Memory Range {:#06x} - {:#06x} out of bounds",index*4000, (index+1)*4000 )
        };
        let end_slice = if self.data.len() >= (index+1)*4000 {
            index+1*4000
        } else {
            self.data.len()
        };
        &self.data[start_slice .. end_slice]
    }
}