use crate::sprite::Sprite;

#[derive(Debug, Default)]
pub(crate) struct TileData {
    pub(crate) index: u8,
    /// The first four lines.
    pub(crate) low: [u8; 8],
    /// The last four lines.
    pub(crate) high: [u8; 8],
    pub(crate) sprite: Option<Sprite>,
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
        TileData { index: self.index, low, high, sprite: None }
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

        TileData { index: self.tile_index(), low, high, sprite: Some(self.sprite) }
    }
}
