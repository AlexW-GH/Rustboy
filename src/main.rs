#[macro_use]
extern crate log;

mod processor;
mod emulator;
mod gpu;
mod memory;
mod util;

use std::sync::Arc;
use std::thread;

use std::fs::File;
use std::io::Read;

use clap::{Arg, App};
use piston_window::*;
use crate::memory::cartridge::Cartridge;
use crate::emulator::gameboy::Gameboy;
use crate::emulator::renderer::Renderer;
use crate::gpu::lcd::LCDFetcher;
use std::sync::Mutex;
use std::rc::Rc;
use std::cell::RefCell;
use simplelog::TestLogger;
use simplelog::LevelFilter;
use simplelog::Config;


fn main() {
    let (filename, boot) = retrieve_options();
    setup_logging(&filename);
    let mut rom = File::open(filename).expect("file not found");
    let cartridge = Cartridge::new(read_game(&mut rom));
    let boot_rom = if boot {
        match File::open("assets/boot.gb") {
            Ok(mut boot_file) => Option::Some(read_game(&mut boot_file)),
            Err(_) => Option::None
        }
    } else { Option::None };
    let lcd = Rc::new(RefCell::new(LCDFetcher::new()));
    let lcd_fetcher = lcd.clone();
    let mut gameboy = Gameboy::new(lcd_fetcher, cartridge, boot_rom);

    let window = create_window();
    let mut renderer = Renderer::new(window, lcd, gameboy);
    renderer.run();
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
    let _log_path = format!("logs/{}.log", file_name);
    //WriteLogger::init(LevelFilter::Debug, Config::default(), File::create(log_path).unwrap()).unwrap();
    TestLogger::init(LevelFilter::Debug, Config::default());
}

fn create_window() -> PistonWindow{
    WindowSettings::new("Rustboy", (emulator::renderer::HOR_PIXELS*3, emulator::renderer::VER_PIXELS*3))
        .exit_on_esc(true)
        .opengl(OpenGL::V3_2)
        .resizable(true)
        .vsync(false)
        .build()
        .unwrap_or_else(|e| { panic!("Failed to build PistonWindow: {}", e) })

}
