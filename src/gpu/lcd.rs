use image::ImageBuffer;
use image::Rgba;
use std::sync::Arc;
use std::sync::Mutex;

const BG_TILES_HOR: u32 = 20;
const BG_TILES_VER: u32 = 18;
const BG_TILE_WIDTH: u32 = 8;
const BG_TILE_HEIGHT: u32 = 8;
pub const HOR_PIXELS: u32 = BG_TILES_HOR*BG_TILE_WIDTH;
pub const VER_PIXELS: u32 = BG_TILES_VER*BG_TILE_HEIGHT;

pub struct LCD{
    lcd_fetcher: Arc<Mutex<LCDFetcher>>,
    image: ImageBuffer<Rgba<u8>, Vec<u8>>
}

impl LCD{
    pub fn new(lcd_fetcher: Arc<Mutex<LCDFetcher>>) -> LCD{
        let image =ImageBuffer::from_fn(HOR_PIXELS, VER_PIXELS, |_, _| {
            Rgba([255u8; 4])
        });
        LCD{image, lcd_fetcher}
    }

    pub fn set_pixel(&mut self, x: u32, y: u32, color: u8){
        //Todo: correct colors

        let pixel = match color {
            0b00 => Rgba([255u8; 4]),
            0b01 => Rgba([180u8; 4]),
            0b10 => Rgba([90u8; 4]),
            0b11 => Rgba([0u8; 4]),
            _ => panic!("That's not a color"),
        };
        self.image.put_pixel(x, y, pixel)
    }

    pub fn display(&self){
        self.lcd_fetcher.lock().unwrap().set_image(self.image.clone())
    }
}

pub struct LCDFetcher{
    image: ImageBuffer<Rgba<u8>, Vec<u8>>
}

impl LCDFetcher{
    pub fn new() -> LCDFetcher{
        let image =ImageBuffer::from_fn(HOR_PIXELS, VER_PIXELS, |_, _| {
            Rgba([255u8; 4])
        });
        LCDFetcher{image}
    }

    pub fn set_image(&mut self, image: ImageBuffer<Rgba<u8>, Vec<u8>>){
        self.image = image;
    }

    pub fn image(&self) -> &ImageBuffer<Rgba<u8>, Vec<u8>>{
        &self.image
    }
}