use crate::sprite::Sprite;

#[derive(Debug, Default)]
pub(crate) struct TileData {
    pub(crate) index: u8,
    pub(crate) low: [u8; 16],
    pub(crate) high: [u8; 16],
}

pub(crate) trait TileDataBuilder {
    fn low(&mut self, data: [u8; 16]) -> &mut Self;
    fn high(&mut self, data: [u8; 16]) -> &mut Self;
    fn build(self) -> TileData;
}

#[derive(Debug, Default)]
pub(crate) struct BackgroundTileDataBuilder {
    pub(crate) index: u8,
    low: Option<[u8; 16]>,
    high: Option<[u8; 16]>,
}

impl BackgroundTileDataBuilder {
    pub(crate) fn new(index: u8) -> Self {
        BackgroundTileDataBuilder { index, low: None, high: None }
    }
}

impl TileDataBuilder for BackgroundTileDataBuilder {
    fn low(&mut self, data: [u8; 16]) -> &mut Self {
        self.low = Some(data);
        self
    }

    fn high(&mut self, data: [u8; 16]) -> &mut Self {
        self.high = Some(data);
        self
    }

    fn build(self) -> TileData {
        let Some(low) = self.low else { panic!("low data is not set") };
        let Some(high) = self.high else { panic!("high data is not set") };
        TileData { index: self.index, low, high }
    }
}

#[derive(Debug, Default)]
pub(crate) struct SpriteTileDataBuilder {
    sprite: Sprite,
    tile_index: u8,
    low: Option<[u8; 16]>,
    high: Option<[u8; 16]>,
}

impl SpriteTileDataBuilder {
    pub(crate) fn new(sprite: Sprite, object_size: u8) -> Self {
        let tile_index =
            if object_size == 16 { sprite.tile_index & 0b1111_1110 } else { sprite.tile_index };
        SpriteTileDataBuilder { sprite, low: None, high: None, tile_index }
    }

    pub(crate) fn tile_index(&self) -> u8 {
        self.tile_index
    }
}

impl TileDataBuilder for SpriteTileDataBuilder {
    fn low(&mut self, data: [u8; 16]) -> &mut Self {
        self.low = Some(data);
        self
    }

    fn high(&mut self, data: [u8; 16]) -> &mut Self {
        self.high = Some(data);
        self
    }

    fn build(self) -> TileData {
        let Some(low) = self.low else { panic!("low data is not set") };
        let Some(high) = self.high else { panic!("high data is not set") };
        // TODO: apply attributes
        TileData { index: self.tile_index(), low, high }
    }
}
