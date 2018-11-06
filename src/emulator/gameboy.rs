use std::rc::Rc;
use std::sync::Arc;
use std::sync::RwLock;
use std::cell::RefCell;
use std::cell::Ref;
use gpu::lcd::LCD;
use memory::cartridge::ROM;
use memory::memory::MemoryController;
use processor::interrupt_controller::InterruptController;
use processor::cpu::CPU;

pub struct Gameboy {
    cpu: CPU,
}

impl Gameboy {
    pub fn new(lcd: Arc<RwLock<LCD>>, rom: ROM, boot: bool) -> Gameboy{
        let memory = Rc::new(RefCell::new(MemoryController::new(rom, boot)));
        let interrupt = InterruptController::new();
        let mut cpu = CPU::new(interrupt, memory.clone(), boot);
        Self::handle_header(memory.borrow());
        Gameboy{ cpu }
    }

    pub fn run(&mut self){
        self.cpu.run();
    }

    fn handle_header(memory: Ref<MemoryController>){
        let title = Self::extract_title(&memory);
        println!("Game Title: {:?}", title);
        println!("Licensee Code: {:#06X}", memory.following_u16(0x143));
        println!("Cardridge Type: {:#04X}", memory.read(0x147));
        println!("Rom Size: {:#04X}", memory.read(0x148));
        println!("external Ram Size: {:#04X}", memory.read(0x149));
        println!("Destination Code: {:#04X}", memory.read(0x14A));
        println!("Old Licensee Code: {:#04X}", memory.read(0x14B));
        println!("Mask ROM Version number: {:#04X}", memory.read(0x14C));
    }

    fn extract_title(memory: &MemoryController) -> String {
        let mut title = Vec::new();
        for i in 0x134..0x144 {
            let char = memory.read(i);
            if char == 0 { break }
            title.push(char)
        }
        String::from_utf8(title).unwrap_or("".to_string())
    }
}

