use crate::{
    emulator::gameboy::Gameboy,
    gpu::lcd::LCDFetcher,
};
use image::{
    imageops,
    FilterType,
};
use piston_window::{
    Event,
    EventLoop,
    G2dTexture,
    Loop,
    PistonWindow,
    Texture,
    TextureSettings,
};
use std::{
    cell::RefCell,
    rc::Rc,
};
use crate::debug::vram_fetcher::VRAMFetcher;
use piston_window::WindowSettings;
use crate::debug::vram_fetcher::VramDebugger;

const BG_TILES_HOR: u32 = 20;
const BG_TILES_VER: u32 = 18;
const BG_TILE_WIDTH: u32 = 8;
const BG_TILE_HEIGHT: u32 = 8;
pub const HOR_PIXELS: u32 = BG_TILES_HOR * BG_TILE_WIDTH;
pub const VER_PIXELS: u32 = BG_TILES_VER * BG_TILE_HEIGHT;

pub struct Renderer {
    window:  PistonWindow,
    lcd:     Rc<RefCell<LCDFetcher>>,
    gameboy: Gameboy,
}

impl Renderer {
    pub fn new(
        mut window: PistonWindow,
        lcd: Rc<RefCell<LCDFetcher>>,
        gameboy: Gameboy,
        fps: u64,
    ) -> Renderer {
        window.events.set_max_fps(fps);
        window.events.set_lazy(false);
        Renderer { window, lcd, gameboy }
    }

    pub fn run(&mut self) {
        let fetcher = VRAMFetcher{};
        while let Some(e) = self.window.next() {
            if let Event::Loop(loop_event) = e {
                if let Loop::Render(render) = loop_event {
                    self.gameboy.render_step();
                    let img = self.lcd.borrow().image().clone();
                    //let img = self.gameboy.render_background_tilemap(fetcher);
                    let img = imageops::resize(
                        &img,
                        render.draw_width,
                        render.draw_height,
                        FilterType::CatmullRom,
                    );
                    let img: G2dTexture = Texture::from_image(
                        &mut self.window.factory,
                        &img,
                        &TextureSettings::new(),
                    )
                    .unwrap();
                    self.window.draw_2d(&e, |c, g| {
                        //piston_window::clear([1.0, 0.0, 0.0, 1.0], g);
                        piston_window::image(&img, c.transform, g);
                    });
                }
            }
        }
    }
}
