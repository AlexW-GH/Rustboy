
use image::imageops;
use image::FilterType;
use crate::gpu::lcd::LCDFetcher;
use piston_window::G2dTexture;
use piston_window::Texture;
use piston_window::TextureSettings;
use piston_window::PistonWindow;
use piston_window::Event;
use piston_window::Loop;
use piston_window::Input;
use piston_window::Size;
use piston_window::Window;
use piston_window::EventLoop;
use crate::emulator::gameboy::Gameboy;
use std::rc::Rc;
use std::cell::RefCell;

const BG_TILES_HOR: u32 = 20;
const BG_TILES_VER: u32 = 18;
const BG_TILE_WIDTH: u32 = 8;
const BG_TILE_HEIGHT: u32 = 8;
pub const HOR_PIXELS: u32 = BG_TILES_HOR*BG_TILE_WIDTH;
pub const VER_PIXELS: u32 = BG_TILES_VER*BG_TILE_HEIGHT;

pub struct Renderer {
    window: PistonWindow,
    lcd: Rc<RefCell<LCDFetcher>>,
    window_width: u32,
    window_height: u32,
    gameboy: Gameboy
}

impl Renderer {
    pub fn new(mut window: PistonWindow, lcd: Rc<RefCell<LCDFetcher>>, gameboy: Gameboy) -> Renderer {
        let size: Size = window.size();
        window.events.set_max_fps(60);
        Renderer { window, lcd , window_width: size.width, window_height: size.height, gameboy}
    }

    pub fn run(&mut self) {
        while let Some(e) = self.window.next() {
            match e {
                Event::Loop(loop_event) => match loop_event {
                    Loop::Render(_r) => {
                        self.gameboy.render_step();
                        let img = self.lcd.borrow().image().clone();
                        let img = imageops::resize(&img, self.window_width, self.window_height, FilterType::CatmullRom);
                        let img: G2dTexture = Texture::from_image(
                            &mut self.window.factory,
                            &img,
                            &TextureSettings::new()).unwrap();
                        self.window.draw_2d(&e, |c, g| {
                            piston_window::clear([1.0, 0.0, 0.0, 1.0], g);
                            piston_window::image(&img, c.transform, g);
                        });
                    },
                    _ => ()
                },
                Event::Input(input_event) => self.handle_input(&input_event),
                _ => ()
            }
        }
    }

    fn handle_input(&mut self, input: &Input){
        match input{
            Input::Resize(w, h) => self.change_window_size(*w, *h),
            _ => ()
        }
    }

    fn change_window_size(&mut self, width: u32, height: u32){
        self.window_height = height;
        self.window_width = width;
    }
}