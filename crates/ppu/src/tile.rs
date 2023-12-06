use crate::config::COLOR_PALETTES;
use crate::sprite::Sprite;

#[derive(Debug, Default)]
pub(crate) struct TileData {
    pub(crate) index: u8,
    pub(crate) colors: [u16; 8],
    pub(crate) sprite: Option<Sprite>,
}

impl TileData {
    pub(crate) fn pick_color(&self, x: u8, y: u8) -> u32 {
        assert!(x < 8 && y < 8);

        let palettes = self.colors[y as usize];
        let bit = (7 - x) as usize * 2;
        let palette = (palettes >> bit) & 0b11;
        COLOR_PALETTES[palette as usize]
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
    row: u8,
    low: Option<[u8; 8]>,
    high: Option<[u8; 8]>,
}

impl BackgroundTileDataBuilder {
    pub(crate) fn new(index: u8, row: u8) -> Self {
        BackgroundTileDataBuilder { index, row, low: None, high: None }
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
        TileData { index: self.index, colors, sprite: None }
    }
}

#[derive(Debug, Default)]
pub(crate) struct SpriteTileDataBuilder {
    sprite: Sprite,
    tile_index: u8,
    row: u8,
    low: Option<[u8; 8]>,
    high: Option<[u8; 8]>,
}

impl SpriteTileDataBuilder {
    pub(crate) fn new(sprite: Sprite, object_size: u8, row: u8) -> Self {
        // Remove the last bit.
        let tile_index =
            if object_size == 16 { sprite.tile_index & 0b1111_1110 } else { sprite.tile_index };
        SpriteTileDataBuilder { sprite, low: None, high: None, row, tile_index }
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

        let colors = mix_colors(low, high);
        TileData { index: self.tile_index(), colors, sprite: Some(self.sprite) }
    }
}

#[cfg(test)]
mod tests {
    use super::{mix_colors, TileData};
    use crate::config::COLOR_PALETTES;

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

        assert_eq!(tile.pick_color(1, 0), COLOR_PALETTES[0b10]);
        assert_eq!(tile.pick_color(3, 4), COLOR_PALETTES[0b11]);
        assert_eq!(tile.pick_color(2, 1), COLOR_PALETTES[0b00]);
        assert_eq!(tile.pick_color(3, 5), COLOR_PALETTES[0b01]);
    }
}
