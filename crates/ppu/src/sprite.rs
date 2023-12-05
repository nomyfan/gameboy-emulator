/// https://gbdev.io/pandocs/OAM.html
#[repr(C)]
#[derive(Debug, Clone, Copy, Default)]
pub(crate) struct Sprite {
    /// Sprite's Y position on the screen + 16.
    pub(crate) y: u8,
    /// Sprite's X position on the screen + 8.
    pub(crate) x: u8,
    pub(crate) tile_index: u8,
    pub(crate) attrs: u8,
}

#[cfg(test)]
mod tests {
    use super::Sprite;

    #[test]
    fn sprite_size() {
        assert_eq!(4, std::mem::size_of::<Sprite>())
    }
}
