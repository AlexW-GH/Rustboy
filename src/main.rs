#[macro_use]
extern crate log;

mod emulator;
mod gpu;
mod mem;
mod processor;
mod util;

use std::{
    fs::File,
    io::Read,
};

use crate::{
    emulator::{
        gameboy::Gameboy,
        renderer::Renderer,
    },
    gpu::lcd::LCDFetcher,
    mem::cartridge::Cartridge,
};
use clap::{
    App,
    Arg,
};
use piston_window::*;
use simplelog::{
    Config,
    LevelFilter,
    TestLogger,
};
use std::{
    cell::RefCell,
    rc::Rc,
};


fn main() {
    let (filename, boot, speed) = retrieve_options();
    setup_logging(&filename);
    let mut rom = File::open(filename).expect("file not found");
    let cartridge = Cartridge::new(read_game(&mut rom));
    let boot_rom = if boot {
        match File::open("assets/boot.gb") {
            Ok(mut boot_file) => Option::Some(read_game(&mut boot_file)),
            Err(_) => Option::None,
        }
    } else {
        Option::None
    };
    let lcd = Rc::new(RefCell::new(LCDFetcher::new()));
    let lcd_fetcher = lcd.clone();
    let gameboy = Gameboy::new(lcd_fetcher, cartridge, boot_rom);

    let window = create_window();
    let fps = (60.0 * speed).floor() as u64;
    println!("FPS: {}", fps);
    let mut renderer = Renderer::new(window, lcd, gameboy, fps);
    renderer.run();
}

fn read_game(file: &mut File) -> Vec<u8> {
    let mut game = Vec::new();
    file.read_to_end(&mut game).expect("something went wrong reading the file");
    game
}

fn retrieve_options() -> (String, bool, f64) {
    let matches = App::new("Rustboy")
        .version("0.1")
        .author("Alexander W.")
        .about("Rust Gameboy Emulator")
        .arg(Arg::with_name("game").help("path of game to play").required(true))
        .arg(Arg::with_name("boot").help("enable boot sequence").short("b").long("boot"))
        .arg(
            Arg::with_name("speed")
                .help("Adjust emulator speed")
                .default_value("1.0")
                .number_of_values(1)
                .short("s")
                .long("speed"),
        )
        .get_matches();
    let game = matches.value_of("game").unwrap().to_string();
    let boot = matches.is_present("boot");
    let speed = matches
        .value_of("speed")
        .map(|val| val.parse::<f64>().unwrap_or_else(|_| 1.0))
        .unwrap_or_else(|| 1.0);
    (game, boot, speed)
}

fn setup_logging(file_name: &str) {
    let file_name = file_name.split('/').collect::<Vec<_>>().last().unwrap().to_string();
    let _log_path = format!("logs/{}.log", file_name);
    // WriteLogger::init(LevelFilter::Debug, Config::default(),
    // File::create(log_path).unwrap()).unwrap(); TestLogger::init(LevelFilter::
    // Debug, Config::default());
}

fn create_window() -> PistonWindow {
    WindowSettings::new(
        "Rustboy",
        (emulator::renderer::HOR_PIXELS * 3, emulator::renderer::VER_PIXELS * 3),
    )
    .exit_on_esc(true)
    .resizable(true)
    .vsync(false)
    .build()
    .unwrap_or_else(|e| panic!("Failed to build PistonWindow: {}", e))
}
