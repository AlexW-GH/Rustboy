const LCDC_REGISTER: u16 = 0xFF40;
const PIXEL_WIDTH: u32 = 128;
const PIXEL_HEIGHT: u32 = 128;

use image::{
    ImageBuffer,
    Rgba,
};
use crate::mem::memory::MapsMemory;

pub trait VramDebugger{
    fn render_all_background_tiles(&self, fetcher: VRAMFetcher) -> ImageBuffer<Rgba<u8>, Vec<u8>>;
    fn render_background_tilemap(&self, fetcher: VRAMFetcher) -> ImageBuffer<Rgba<u8>, Vec<u8>>;
}

#[derive(Clone, Copy)]
pub struct VRAMFetcher{

}


impl VRAMFetcher{
    pub(crate) fn render_all_background_tiles(&self, memory: &MapsMemory) -> ImageBuffer<Rgba<u8>, Vec<u8>>{
        let image = ImageBuffer::from_fn(64, 384, |_, _| Rgba([255u8; 4]));

        image
    }

    pub(crate) fn render_background_tilemap(&self, memory: &MapsMemory) -> ImageBuffer<Rgba<u8>, Vec<u8>>{
        let mut image = ImageBuffer::from_fn(PIXEL_WIDTH, PIXEL_HEIGHT, |_, _| Rgba([255u8; 4]));

        let lcd_control_register = memory.read(LCDC_REGISTER).unwrap();
        let bg_tiles_address: u16 =
            if (lcd_control_register >> 4) & 1 != 0 { 0x8800 } else { 0x8000 };
        let mut counter = 0;
        for sprite_y in 0u16 .. 16{
            for sprite_x in 0u16 .. 16 {
                println!("Sprite: {}", (sprite_x + (sprite_y*16)));
                for pixel_y in 0u16 .. 8{
                    let address = bg_tiles_address + u16::from((pixel_y) * 0x2) + ((sprite_x + (sprite_y*16)) * 0x10);
                    let data0 = memory.read(address).unwrap();
                    let data1 = memory.read(address+1).unwrap();

                    let color_coded = Self::combine_pixels(data0, data1);
                    let pixels = Self::create_pixels(color_coded);
                    println!("     {:#06x}", address);
                    image.put_pixel((sprite_x*8+0) as u32, (sprite_y*8+pixel_y) as u32, pixels.0);
                    image.put_pixel((sprite_x*8+1) as u32, (sprite_y*8+pixel_y) as u32, pixels.1);
                    image.put_pixel((sprite_x*8+2) as u32, (sprite_y*8+pixel_y) as u32, pixels.2);
                    image.put_pixel((sprite_x*8+3) as u32, (sprite_y*8+pixel_y) as u32, pixels.3);
                    image.put_pixel((sprite_x*8+4) as u32, (sprite_y*8+pixel_y) as u32, pixels.4);
                    image.put_pixel((sprite_x*8+5) as u32, (sprite_y*8+pixel_y) as u32, pixels.5);
                    image.put_pixel((sprite_x*8+6) as u32, (sprite_y*8+pixel_y) as u32, pixels.6);
                    image.put_pixel((sprite_x*8+7) as u32, (sprite_y*8+pixel_y) as u32, pixels.7);
                }
            }
        }
        image
    }

    fn combine_pixels(data0: u8, data1: u8) -> u16 {
        let mut result: u16 = 0;
        result |= u16::from(((data1 >> 7) & 1) << 1 | ((data0 >> 7) & 1)) << 14;
        result |= u16::from(((data1 >> 6) & 1) << 1 | ((data0 >> 6) & 1)) << 12;
        result |= u16::from(((data1 >> 5) & 1) << 1 | ((data0 >> 5) & 1)) << 10;
        result |= u16::from(((data1 >> 4) & 1) << 1 | ((data0 >> 4) & 1)) << 8;
        result |= u16::from(((data1 >> 3) & 1) << 1 | ((data0 >> 3) & 1)) << 6;
        result |= u16::from(((data1 >> 2) & 1) << 1 | ((data0 >> 2) & 1)) << 4;
        result |= u16::from(((data1 >> 1) & 1) << 1 | ((data0 >> 1) & 1)) << 2;
        result |= u16::from(((data1) & 1) << 1 | ((data0) & 1));
        // println!("Combined Pixels: {:#018b}", result);
        result
    }

    fn create_pixels(color_code: u16) -> (Rgba<u8>, Rgba<u8>, Rgba<u8>, Rgba<u8>, Rgba<u8>, Rgba<u8>, Rgba<u8>, Rgba<u8>){
        (
            Self::decode_pixel((color_code >> 14) & 0b11),
            Self::decode_pixel((color_code >> 12) & 0b11),
            Self::decode_pixel((color_code >> 10) & 0b11),
            Self::decode_pixel((color_code >> 8 ) & 0b11),
            Self::decode_pixel((color_code >> 6 ) & 0b11),
            Self::decode_pixel((color_code >> 4 ) & 0b11),
            Self::decode_pixel((color_code >> 2 ) & 0b11),
            Self::decode_pixel((color_code      ) & 0b11)
        )
    }

    fn decode_pixel(color_code: u16) -> Rgba<u8>{
        match color_code {
            0b00 => Rgba([255u8, 255u8, 255u8, 255u8]),
            0b01 => Rgba([180u8, 180u8, 180u8, 255u8]),
            0b10 => Rgba([90u8, 90u8, 90u8, 255u8]),
            0b11 => Rgba([0u8, 0u8, 0u8, 255u8]),
            _ => panic!("That's not a color"),
        }
    }
}