extern crate clap;

mod cpu;
mod registers;
mod rom;
mod memory;
mod util;
mod interrupt_controller;

use std::sync::{Arc, RwLock};
use std::thread;

use registers::Registers;
use registers::RegisterR;
use registers::RegisterDD;
use memory::Memory;
use rom::ROM;
use std::fs::File;
use std::io::Read;
use cpu::CPU;
use interrupt_controller::InterruptController;

use clap::{Arg, App};


fn main() {
    let filename = retrieve_path_to_game();
    let mut file = File::open(filename).expect("file not found");
    let rom = ROM::new(read_game(&mut file));
    let memory = Arc::new(RwLock::new(Memory::new(rom)));
    let interrupt = Arc::new(RwLock::new(InterruptController::new()));
    let mut cpu = CPU::new(interrupt, memory.clone());
    //handle_header(&memory);
    let cpu_handle = thread::spawn(move || {
        loop{
            cpu.step();
        }
    });
    cpu_handle.join().unwrap_or(panic!("at the disco"));

}

fn handle_header(memory: &Memory){
    let title = extract_title(memory);
    println!("Game Title: {:?}", title);
    println!("Licensee Code: {:#06X}", memory.following_u16(0x143));
    println!("Cardridge Type: {:#04X}", memory.read(0x147));
    println!("Rom Size: {:#04X}", memory.read(0x148));
    println!("external Ram Size: {:#04X}", memory.read(0x149));
    println!("Destination Code: {:#04X}", memory.read(0x14A));
    println!("Old Licensee Code: {:#04X}", memory.read(0x14B));
    println!("Mask ROM Version number: {:#04X}", memory.read(0x14C));
}

fn extract_title(memory: &Memory) -> String {
    let mut title = Vec::new();
    for i in 0x134..0x144 {
        let char = memory.read(i);
        if char == 0 { break }
        title.push(char)
    }
    String::from_utf8(title).unwrap_or("".to_string())
}

fn read_game(file: &mut File) -> Vec<u8>{
    let mut game = Vec::new();
    file.read_to_end(&mut game).expect("something went wrong reading the file");
    game
}

fn retrieve_path_to_game() -> String {
    let matches = App::new("Rustboy")
        .version("0.1")
        .author("Alexander W.")
        .about("Rust Gameboy Emulator")
        .arg(Arg::with_name("game")
            .help("path of game to play")
            .required(true))
        .get_matches();
    matches.value_of("game").unwrap().to_string()
}
