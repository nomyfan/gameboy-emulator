use gb_shared::is_bit_set;

/// https://gbdev.io/pandocs/OAM.html
#[repr(C)]
#[derive(Debug, Clone, Copy, Default)]
pub(crate) struct Object {
    /// Object's Y position on the screen + 16.
    pub(crate) y: u8,
    /// Object's X position on the screen + 8.
    pub(crate) x: u8,
    pub(crate) tile_index: u8,
    /// Object attributes
    /// - Bit 0-2: palette number. CGB only.
    /// - Bit 3: tile VRAM bank. CGB only.
    /// - Bit 4: palette number. Non CGB only.
    /// - Bit 5: X flip(0=normal, 1=horizontally mirrored).
    /// - Bit 6: Y flip(0=normal, 1=vertically mirrored).
    /// - Bit 7: BG and Window over OBJ(0=No, 1=BG and Window colors 1-3 are drawn over the OBJ)
    pub(crate) attrs: ObjectAttrs,
}

#[derive(Debug, Clone, Copy, Default)]
pub(crate) struct ObjectSnapshot {
    pub(crate) object: Object,
    pub(crate) size: u8,
}

#[repr(C)]
#[derive(Debug, Clone, Copy, Default)]
pub(crate) struct ObjectAttrs(pub(crate) u8);

impl ObjectAttrs {
    pub(crate) fn y_flip(&self) -> bool {
        is_bit_set!(self.0, 6)
    }

    pub(crate) fn x_flip(&self) -> bool {
        is_bit_set!(self.0, 5)
    }

    /// BG and Window colors 1-3 are drawn over this object.
    pub(crate) fn bgw_over_object(&self) -> bool {
        is_bit_set!(self.0, 7)
    }

    pub(crate) fn dmg_palette(&self) -> u8 {
        is_bit_set!(self.0, 4) as u8
    }
}

impl From<u8> for ObjectAttrs {
    fn from(value: u8) -> Self {
        Self(value)
    }
}

#[cfg(test)]
mod tests {
    use super::Object;

    #[test]
    fn object_size() {
        assert_eq!(4, std::mem::size_of::<Object>())
    }
}
