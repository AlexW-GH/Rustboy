use std::sync::{Arc, RwLock};

use piston_window::*;
use gpu::lcd::LCD;

const BG_TILES_HOR: u32 = 20;
const BG_TILES_VER: u32 = 18;
const BG_TILE_WIDTH: u32 = 8;
const BG_TILE_HEIGHT: u32 = 8;
pub const HOR_PIXELS: u32 = BG_TILES_HOR*BG_TILE_WIDTH;
pub const VER_PIXELS: u32 = BG_TILES_VER*BG_TILE_HEIGHT;

pub struct Renderer {
    window: PistonWindow,
    lcd: Arc<RwLock<LCD>>,

    draw_info: DrawInformation,
}

struct DrawInformation{
    draw_offset_top: u32,
    draw_offset_left: u32,
    pixel_width: u32,
    pixel_height: u32,
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
    pub fn new(window: PistonWindow, lcd: Arc<RwLock<LCD>>) -> Renderer {
        let size: Size = window.size();
        let draw_info = Renderer::create_draw_info(size.width, size.height);
        Renderer { window, lcd, draw_info }
    }

    fn create_draw_info(width: u32, height: u32) -> DrawInformation{
        let draw_offset_left = (width % HOR_PIXELS) / 2;
        let draw_offset_top = (height % VER_PIXELS) / 2;
        let pixel_width = width / HOR_PIXELS;
        let pixel_height = height / VER_PIXELS;

        DrawInformation{draw_offset_left, draw_offset_top, pixel_width, pixel_height}
    }

    pub fn run(&mut self) {

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
        self.draw_info = Self::create_draw_info(width, height);
    }

    fn draw_background<G>(pixel_width: f32, pixel_height: f32, c: Context, g: &mut G) where G: Graphics{
        for i in 0 .. HOR_PIXELS {
            for j in 0 .. VER_PIXELS{
                let x = i as f32;
                let y = (j*0xF) as f32;
                let pos = x+y;
                let color = (i as f32)/HOR_PIXELS as f32;
                //rectangle([0.75, 0.75, 0.75, 1.00]/*[color, color, color, 1.0]*/,[x, y, pixel_width, pixel_height], c.transform, g);
            }
        }
    }
}