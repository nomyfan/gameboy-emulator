mod compatibility_palettes;
mod monochrome;
mod polychrome;

pub(crate) trait Palette {
    fn background_color(&self, palette_id: u8, color_id: u8) -> u32;
    fn object_color(&self, palette_id: u8, color_id: u8) -> u32;
    fn colors(&self) -> &[[u32; 4]];
}

pub(crate) use monochrome::Monochrome;
pub(crate) use polychrome::Polychrome;
