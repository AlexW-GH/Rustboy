use image::ImageBuffer;
use image::Rgba;
use piston_window::image::Image;

const BG_TILES_HOR: u32 = 20;
const BG_TILES_VER: u32 = 18;
const BG_TILE_WIDTH: u32 = 8;
const BG_TILE_HEIGHT: u32 = 8;
pub const HOR_PIXELS: u32 = BG_TILES_HOR*BG_TILE_WIDTH;
pub const VER_PIXELS: u32 = BG_TILES_VER*BG_TILE_HEIGHT;

pub struct LCD{
    image: ImageBuffer<Rgba<u8>, Vec<u8>>
}

impl LCD{
    pub fn new() -> LCD{
        let image =ImageBuffer::from_fn(HOR_PIXELS, VER_PIXELS, |x, y| {
            Rgba([200u8; 4])
        });
        LCD{image}
    }

    pub fn image(&self) -> &ImageBuffer<Rgba<u8>, Vec<u8>>{
        &self.image
    }
}