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
    _low: Option<[u8; 16]>,
    _high: Option<[u8; 16]>,
}

impl BackgroundTileDataBuilder {
    pub(crate) fn new(index: u8) -> Self {
        BackgroundTileDataBuilder { index, _low: None, _high: None }
    }
}

impl TileDataBuilder for BackgroundTileDataBuilder {
    fn low(&mut self, data: [u8; 16]) -> &mut Self {
        self._low = Some(data);
        self
    }

    fn high(&mut self, data: [u8; 16]) -> &mut Self {
        self._high = Some(data);
        self
    }

    fn build(self) -> TileData {
        let Some(low) = self._low else { panic!("low data is not set") };
        let Some(high) = self._high else { panic!("high data is not set") };
        TileData { index: self.index, low, high }
    }
}

#[derive(Debug, Default)]
pub(crate) struct SpriteTileDataBuilder {
    _sprite: Sprite,
    _low: Option<[u8; 16]>,
    _high: Option<[u8; 16]>,
}

impl SpriteTileDataBuilder {
    pub(crate) fn new(sprite: Sprite) -> Self {
        SpriteTileDataBuilder { _sprite: sprite, _low: None, _high: None }
    }

    pub(crate) fn tile_index(&self) -> u8 {
        self._sprite.tile_index
    }
}

impl TileDataBuilder for SpriteTileDataBuilder {
    fn low(&mut self, data: [u8; 16]) -> &mut Self {
        self._low = Some(data);
        self
    }

    fn high(&mut self, data: [u8; 16]) -> &mut Self {
        self._high = Some(data);
        self
    }

    fn build(self) -> TileData {
        let Some(low) = self._low else { panic!("low data is not set") };
        let Some(high) = self._high else { panic!("high data is not set") };
        // TODO: apply attributes
        TileData { index: self._sprite.tile_index, low, high }
    }
}
