use std::sync::{Arc, RwLock};

use memory::Memory;
use piston_window::*;

const BG_TILES_HOR: u32 = 20;
const BG_TILES_VER: u32 = 18;
const BG_TILE_WIDTH: u32 = 8;
const BG_TILE_HEIGHT: u32 = 8;
pub const HOR_PIXELS: u32 = BG_TILES_HOR*BG_TILE_WIDTH;
pub const VER_PIXELS: u32 = BG_TILES_VER*BG_TILE_HEIGHT;

pub struct Renderer {
    window: PistonWindow,
    memory: Arc<RwLock<Memory>>,

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
    pub fn new(window: PistonWindow, memory: Arc<RwLock<Memory>>) -> Renderer {
        let size: Size = window.size();
        let draw_info = Renderer::create_draw_info(size.width, size.height);
        Renderer { window, memory, draw_info }
    }

    fn create_draw_info(width: u32, height: u32) -> DrawInformation{
        let draw_offset_left = (width % HOR_PIXELS) / 2;
        let draw_offset_top = (height % VER_PIXELS) / 2;
        let pixel_width = width / HOR_PIXELS;
        let pixel_height = height / VER_PIXELS;

        DrawInformation{draw_offset_left, draw_offset_top, pixel_width, pixel_height}
    }

    pub fn run(&mut self) {
        let memory = self.memory.clone();
        while let Some(e) = self.window.next() {
            match &e {
                Event::Input(input) => self.handle_input(input),
                //Event::Loop(draw_loop) => self.draw(draw_loop),
                _ => ()
            }
            let pixel_width = self.draw_info.pixel_width as f64;
            let pixel_height = self.draw_info.pixel_height as f64;
            let left_offset = self.draw_info.draw_offset_left as f64;
            let top_offset = self.draw_info.draw_offset_top as f64;
            println!("Pixel Width: {}", pixel_width);
            println!("Pixel Height: {}", pixel_height);
            self.window.draw_2d(&e, |c, g| {
                //Self::draw_background(pixel_width, pixel_height, c, g);
                clear([0.5, 0.5, 0.5, 0.0], g);
                for i in 0 .. BG_TILES_HOR {
                    for j in 0 .. BG_TILES_VER{
                        let x = i as f64 * pixel_width * BG_TILE_WIDTH as f64 + left_offset;
                        let y = j as f64 * pixel_height * BG_TILE_HEIGHT as f64 + top_offset;
                        let pos = x as u16+(y as u16*0x10);
                        let color = (i as f32*j as f32)/(HOR_PIXELS as f32*VER_PIXELS as f32);
                        let tile = {
                            let memory = memory.read().unwrap();
                            BgTile{
                                line0: ((memory.read((pos*2)+0x8000 + (256*0)) as u16) << 8) + (memory.read((pos*2)+ 0x8001)) as u16,
                                line1: ((memory.read((pos*2)+0x8000 + (256*1)) as u16) << 8) + (memory.read((pos*2)+ 0x8001)) as u16,
                                line2: ((memory.read((pos*2)+0x8000 + (256*2)) as u16) << 8) + (memory.read((pos*2)+ 0x8001)) as u16,
                                line3: ((memory.read((pos*2)+0x8000 + (256*3)) as u16) << 8) + (memory.read((pos*2)+ 0x8001)) as u16,
                                line4: ((memory.read((pos*2)+0x8000 + (256*4)) as u16) << 8) + (memory.read((pos*2)+ 0x8001)) as u16,
                                line5: ((memory.read((pos*2)+0x8000 + (256*5)) as u16) << 8) + (memory.read((pos*2)+ 0x8001)) as u16,
                                line6: ((memory.read((pos*2)+0x8000 + (256*6)) as u16) << 8) + (memory.read((pos*2)+ 0x8001)) as u16,
                                line7: ((memory.read((pos*2)+0x8000 + (256*7)) as u16) << 8) + (memory.read((pos*2)+ 0x8001)) as u16,
                            }
                        };
                        for i in 0 .. BG_TILE_HEIGHT{
                            for j in 0 .. BG_TILE_WIDTH{
                                let line = match i {
                                    0 => tile.line0,
                                    1 => tile.line1,
                                    2 => tile.line2,
                                    3 => tile.line3,
                                    4 => tile.line4,
                                    5 => tile.line5,
                                    6 => tile.line6,
                                    7 => tile.line7,
                                    _ => unreachable!()
                                };
                                println!("{:?}", tile);
                                let color = Self::select_color(line, BG_TILE_WIDTH-1-j);
                                print!("line {} ({}): ", j, line);
                                println!("{}, {}, {}, {}", x as f64 + (j as f64 * pixel_width as f64), y as f64+i as f64* pixel_height as f64, pixel_width as f64, pixel_height as f64);
                                rectangle(
                                    [color, color, color, 1.0],
                                    [
                                        x as f64 + (j as f64 * pixel_width as f64),
                                        y as f64 + (i as f64* pixel_height as f64),
                                        pixel_width as f64,
                                        pixel_height as f64],
                                    c.transform, g);
                            }
                        }

                    }
                }
            });
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