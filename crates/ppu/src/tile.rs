use crate::sprite::{Sprite, SpriteAttrs};
use gb_shared::pick_bits;

#[derive(Debug, Default)]
pub(crate) struct TileData {
    pub(crate) index: u8,
    pub(crate) colors: [u16; 8],
    pub(crate) sprite_attrs: Option<SpriteAttrs>,
}

impl TileData {
    /// Return color ID in range of 0..4.
    pub(crate) fn get_color_id(&self, x: u8, y: u8) -> u8 {
        assert!(x < 8 && y < 8);

        let palettes = self.colors[y as usize];
        let bit = (7 - x) as usize * 2;
        let palette = (palettes >> bit) & 0b11;

        palette as u8
    }
}

pub(crate) trait TileDataBuilder {
    fn low(&mut self, data: [u8; 8]) -> &mut Self;
    fn high(&mut self, data: [u8; 8]) -> &mut Self;
    fn build(self) -> TileData;
}

#[derive(Debug, Default)]
pub(crate) struct BackgroundTileDataBuilder {
    pub(crate) index: u8,
    low: Option<[u8; 8]>,
    high: Option<[u8; 8]>,
}

impl BackgroundTileDataBuilder {
    pub(crate) fn new(index: u8) -> Self {
        BackgroundTileDataBuilder { index, low: None, high: None }
    }
}

fn mix_colors(low: [u8; 8], high: [u8; 8]) -> [u16; 8] {
    let mut colors: [u16; 8] = Default::default();

    let mut mix = |data: [u8; 8], offset: usize| {
        for i in (0..data.len()).step_by(2) {
            let lsbs = data[i];
            let msbs = data[i + 1];

            let mut color = 0u16;
            for bit in 0..8 {
                let lsb = (lsbs & (1 << bit)) as u16 >> bit;
                let msb = (msbs & (1 << bit)) as u16 >> bit;

                let lsb = lsb << (bit * 2);
                let msb = msb << (bit * 2 + 1);

                color |= msb | lsb;
            }
            colors[i / 2 + offset] = color;
        }
    };

    mix(low, 0);
    mix(high, 4);

    colors
}

fn apply_attrs<'data, 'attrs>(data: &'data mut [u16; 8], attrs: &'attrs SpriteAttrs) {
    if attrs.y_flip() {
        for i in 0..4 {
            let i_rev = 7 - i;
            let tmp = data[i];
            data[i] = data[i_rev];
            data[i_rev] = tmp;
        }
    }
    if attrs.x_flip() {
        for i in 0..8 {
            let mut new_value = 0;
            for offset in (0..16).step_by(2) {
                let mut val = pick_bits!(data[i], offset, offset + 1);
                val >>= offset;
                val <<= 14 - offset;

                new_value |= val;
            }
            data[i] = new_value;
        }
    }
}

impl TileDataBuilder for BackgroundTileDataBuilder {
    fn low(&mut self, data: [u8; 8]) -> &mut Self {
        self.low = Some(data);
        self
    }

    fn high(&mut self, data: [u8; 8]) -> &mut Self {
        self.high = Some(data);
        self
    }

    fn build(self) -> TileData {
        let Some(low) = self.low else { panic!("low data is not set") };
        let Some(high) = self.high else { panic!("high data is not set") };

        let colors = mix_colors(low, high);
        TileData { index: self.index, colors, sprite_attrs: None }
    }
}

#[derive(Debug, Default)]
pub(crate) struct SpriteTileDataBuilder {
    sprite: Sprite,
    tile_index: u8,
    low: Option<[u8; 8]>,
    high: Option<[u8; 8]>,
}

impl SpriteTileDataBuilder {
    pub(crate) fn new(sprite: Sprite, object_size: u8) -> Self {
        // Remove the last bit.
        let tile_index =
            if object_size == 16 { sprite.tile_index & 0b1111_1110 } else { sprite.tile_index };
        SpriteTileDataBuilder { sprite, low: None, high: None, tile_index }
    }

    pub(crate) fn tile_index(&self) -> u8 {
        self.tile_index
    }
}

impl TileDataBuilder for SpriteTileDataBuilder {
    fn low(&mut self, data: [u8; 8]) -> &mut Self {
        self.low = Some(data);
        self
    }

    fn high(&mut self, data: [u8; 8]) -> &mut Self {
        self.high = Some(data);
        self
    }

    fn build(self) -> TileData {
        let Some(low) = self.low else { panic!("low data is not set") };
        let Some(high) = self.high else { panic!("high data is not set") };

        let mut colors = mix_colors(low, high);
        apply_attrs(&mut colors, &self.sprite.attrs);
        TileData { index: self.tile_index(), colors, sprite_attrs: Some(self.sprite.attrs) }
    }
}

#[cfg(test)]
mod tests {
    use super::{apply_attrs, mix_colors, TileData};
    use crate::{config::COLOR_PALETTES, sprite::SpriteAttrs};

    #[test]
    fn test_build_colors() {
        let low: [u8; 8] = [
            0b00_11_11_00,
            0b01_11_11_10,
            0b01_00_00_10,
            0b01_00_00_10,
            0b01_00_00_10,
            0b01_00_00_10,
            0b01_00_00_10,
            0b01_00_00_10,
        ];
        let high: [u8; 8] = [
            0b01_11_11_10,
            0b01_01_11_10,
            0b01_11_11_10,
            0b00_00_10_10,
            0b01_11_11_00,
            0b01_01_01_10,
            0b00_11_10_00,
            0b01_11_11_00,
        ];

        let colors = mix_colors(low, high);

        let expected: [u16; 8] = [
            0b00_10_11_11_11_11_10_00,
            0b00_11_00_00_00_00_11_00,
            0b00_11_00_00_00_00_11_00,
            0b00_11_00_00_00_00_11_00,
            0b00_11_01_11_11_11_11_00,
            0b00_01_01_01_11_01_11_00,
            0b00_11_01_11_01_11_10_00,
            0b00_10_11_11_11_10_00_00,
        ];

        assert_eq!(expected, colors);
    }

    #[test]
    fn pick_color() {
        let mut tile = TileData::default();
        tile.colors = [
            0b00_10_11_11_11_11_10_00,
            0b00_11_00_00_00_00_11_00,
            0b00_11_00_00_00_00_11_00,
            0b00_11_00_00_00_00_11_00,
            0b00_11_01_11_11_11_11_00,
            0b00_01_01_01_11_01_11_00,
            0b00_11_01_11_01_11_10_00,
            0b00_10_11_11_11_10_00_00,
        ];

        assert_eq!(tile.get_color_id(1, 0), 0b10);
        assert_eq!(tile.get_color_id(3, 4), 0b11);
        assert_eq!(tile.get_color_id(2, 1), 0b00);
        assert_eq!(tile.get_color_id(3, 5), 0b01);
    }

    #[test]
    fn flip_x() {
        let mut data = [
            0b00_10_01_11_00_10_01_11,
            0b00_00_00_00_00_00_00_00,
            0b00_00_00_00_00_00_00_00,
            0b00_00_00_00_00_00_00_00,
            0b00_00_00_00_00_00_00_00,
            0b00_00_00_00_00_00_00_00,
            0b00_00_00_00_00_00_00_00,
            0b00_00_00_00_00_00_00_00,
        ];
        apply_attrs(&mut data, &SpriteAttrs(0b0010_0000));

        let expected = [
            0b11_01_10_00_11_01_10_00,
            0b00_00_00_00_00_00_00_00,
            0b00_00_00_00_00_00_00_00,
            0b00_00_00_00_00_00_00_00,
            0b00_00_00_00_00_00_00_00,
            0b00_00_00_00_00_00_00_00,
            0b00_00_00_00_00_00_00_00,
            0b00_00_00_00_00_00_00_00,
        ];

        assert_eq!(expected, data);
    }

    #[test]
    fn flip_y() {
        let mut data = [
            0b00_10_01_11_00_10_01_11,
            0b00_00_00_00_00_00_00_00,
            0b00_00_00_00_00_00_00_00,
            0b00_00_00_00_00_00_00_00,
            0b00_00_00_00_00_00_00_00,
            0b00_00_00_00_00_00_00_00,
            0b00_00_00_00_00_00_00_00,
            0b00_00_00_00_00_00_00_00,
        ];
        apply_attrs(&mut data, &SpriteAttrs(0b0100_0000));

        let expected = [
            0b00_00_00_00_00_00_00_00,
            0b00_00_00_00_00_00_00_00,
            0b00_00_00_00_00_00_00_00,
            0b00_00_00_00_00_00_00_00,
            0b00_00_00_00_00_00_00_00,
            0b00_00_00_00_00_00_00_00,
            0b00_00_00_00_00_00_00_00,
            0b00_10_01_11_00_10_01_11,
        ];

        assert_eq!(expected, data);
    }
}
