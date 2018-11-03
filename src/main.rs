#[macro_use]
extern crate log;
extern crate simplelog;
extern crate clap;

mod cpu;
mod registers;
mod rom;
mod memory;
mod util;
mod interrupt_controller;
mod opcodes;

use std::sync::{Arc, RwLock};
use std::thread;

use memory::Memory;
use rom::ROM;
use std::fs::File;
use std::io::Read;
use cpu::CPU;
use interrupt_controller::InterruptController;

use clap::{Arg, App};


fn main() {
    let (filename, boot) = retrieve_options();
    setup_logging(&filename);
    let mut file = File::open(filename).expect("file not found");
    let rom = ROM::new(read_game(&mut file));
    let memory = Arc::new(RwLock::new(Memory::new(rom, boot)));
    let interrupt = Arc::new(RwLock::new(InterruptController::new()));
    let mut cpu = CPU::new(interrupt, memory.clone(), boot);
    handle_header(&memory.read().unwrap());
    let cpu_handle = thread::spawn(move || {
        loop{
            cpu.step();
        }
    });
    cpu_handle.join().unwrap_or(panic!("the disco"));
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

fn retrieve_options() -> (String, bool) {
    let matches = App::new("Rustboy")
        .version("0.1")
        .author("Alexander W.")
        .about("Rust Gameboy Emulator")
        .arg(Arg::with_name("game")
            .help("path of game to play")
            .required(true))
        .arg(Arg::with_name("boot")
                 .help("enable boot sequence")
                 .short("b")
                 .long("boot"))
        .get_matches();
    let game = matches.value_of("game").unwrap().to_string();
    let boot = matches.is_present("boot");
    (game, boot)
}

fn setup_logging(file_name: &str){
    let file_name = file_name.split("/").collect::<Vec<_>>().last().unwrap().to_string();
    let log_path = format!("logs/{}.log", file_name);
    //WriteLogger::init(LevelFilter::Debug, Config::default(), File::create(log_path).unwrap()).unwrap();
    //TestLogger::init(LevelFilter::Debug, Config::default())
}
