use std::sync::{Arc, RwLock};

use piston_window::*;
use gpu::lcd::LCD;
use image::imageops;
use image::FilterType;
use gpu::lcd::LCDFetcher;
use std::sync::Mutex;

const BG_TILES_HOR: u32 = 20;
const BG_TILES_VER: u32 = 18;
const BG_TILE_WIDTH: u32 = 8;
const BG_TILE_HEIGHT: u32 = 8;
pub const HOR_PIXELS: u32 = BG_TILES_HOR*BG_TILE_WIDTH;
pub const VER_PIXELS: u32 = BG_TILES_VER*BG_TILE_HEIGHT;

pub struct Renderer {
    window: PistonWindow,
    lcd: Arc<Mutex<LCDFetcher>>,
    window_width: u32,
    window_height: u32,
}

#[derive(Debug)]
struct BgTile{
    line0: u16,
    line1: u16,
    line2: u16,
    line3: u16,
    line4: u16,
    line5: u16,
    line6: u16,
    line7: u16
}

impl Renderer {
    pub fn new(window: PistonWindow, lcd: Arc<Mutex<LCDFetcher>>) -> Renderer {
        let size: Size = window.size();
        Renderer { window, lcd , window_width: size.width, window_height: size.height}
    }

    pub fn run(&mut self) {
        while let Some(e) = self.window.next() {
            match e {
                Event::Loop(loop_event) => match loop_event {
                    Loop::Render(r) => {
                        let img = imageops::resize(self.lcd.lock().unwrap().image(), self.window_width, self.window_height, FilterType::Lanczos3);
                        let img: G2dTexture = Texture::from_image(
                            &mut self.window.factory,
                            &img,
                            &TextureSettings::new()).unwrap();
                        self.window.draw_2d(&e, |c, g| {
                            clear([1.0, 0.0, 0.0, 1.0], g);
                            image(&img, c.transform, g);
                        });
                    },
                    _ => ()
                },
                _ => ()
            }
        }
    }

    fn select_color(line: u16, bits: u32) -> f32{
        let color_bits = line >> (bits*2) & 0b11;
        match color_bits{
            0b00 => 1.0,
            0b01 => 0.66,
            0b10 => 0.33,
            0b11 => 0.0,
            _ => unreachable!()
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