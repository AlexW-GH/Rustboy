#![feature(duration_as_u128)]

#[macro_use]
extern crate log;
extern crate simplelog;
extern crate clap;
extern crate chrono;
extern crate piston_window;
extern crate core;

mod cpu;
mod renderer;
mod registers;
mod rom;
mod memory;
mod util;
mod interrupt_controller;
mod opcodes;
mod ppu;
mod gameboy;
mod lcd;

use std::sync::{Arc, RwLock};
use std::thread;

use memory::Memory;
use rom::ROM;
use std::fs::File;
use std::io::Read;
use cpu::CPU;
use interrupt_controller::InterruptController;

use clap::{Arg, App};
use piston_window::*;
use renderer::Renderer;
use simplelog::TestLogger;
use simplelog::LevelFilter;
use simplelog::Config;
use gameboy::Gameboy;
use lcd::LCD;

fn main() {
    let (filename, boot) = retrieve_options();
    setup_logging(&filename);
    let mut file = File::open(filename).expect("file not found");
    let rom = ROM::new(read_game(&mut file));
    let lcd = Arc::new(RwLock::new(LCD::new()));
    let cpu_lcd = lcd.clone();
    let cpu_handle = thread::spawn(move || {
        let mut gameboy = Gameboy::new(cpu_lcd, rom, boot);
        gameboy.run();
    });
    let window = create_window();
    let mut renderer = Renderer::new(window, lcd);
    //renderer.run();
    cpu_handle.join().unwrap_or(panic!("the disco"));
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
    //TestLogger::init(LevelFilter::Debug, Config::default());
}

fn create_window() -> PistonWindow{
    WindowSettings::new("Rustboy", (renderer::HOR_PIXELS*2, renderer::VER_PIXELS*2))
        .exit_on_esc(true)
        .resizable(false)
        .build()
        .unwrap_or_else(|e| { panic!("Failed to build PistonWindow: {}", e) })
}
