use gb_shared::is_bit_set;

/// https://gbdev.io/pandocs/OAM.html
#[repr(C)]
#[derive(Debug, Clone, Copy, Default)]
pub(crate) struct Sprite {
    /// Sprite's Y position on the screen + 16.
    pub(crate) y: u8,
    /// Sprite's X position on the screen + 8.
    pub(crate) x: u8,
    pub(crate) tile_index: u8,
    /// Sprite attributes
    /// - Bit 0-2: palette number. CGB only.
    /// - Bit 3: tile VRAM bank. CGB only.
    /// - Bit 4: palette number. Non CGB only.
    /// - Bit 5: X flip(0=normal, 1=horizontally mirrored).
    /// - Bit 6: Y flip(0=normal, 1=vertically mirrored).
    /// - Bit 7: BG and Window over OBJ(0=No, 1=BG and Window colors 1-3 are drawn over the OBJ)
    pub(crate) attrs: SpriteAttrs,
}

#[repr(C)]
#[derive(Debug, Clone, Copy, Default)]
pub(crate) struct SpriteAttrs(pub(crate) u8);

impl SpriteAttrs {
    pub(crate) fn y_flip(&self) -> bool {
        is_bit_set!(self.0, 6)
    }

    pub(crate) fn x_flip(&self) -> bool {
        is_bit_set!(self.0, 5)
    }

    /// BG and Window colors 1-3 are drawn over this sprite.
    pub(crate) fn bgw_over_object(&self) -> bool {
        is_bit_set!(self.0, 7)
    }

    pub(crate) fn dmg_palette(&self) -> u8 {
        if is_bit_set!(self.0, 4) {
            1
        } else {
            0
        }
    }
}

impl From<u8> for SpriteAttrs {
    fn from(value: u8) -> Self {
        Self(value)
    }
}

#[cfg(test)]
mod tests {
    use super::Sprite;

    #[test]
    fn sprite_size() {
        assert_eq!(4, std::mem::size_of::<Sprite>())
    }
}
