use gb_shared::is_bit_set;

#[repr(u8)]
#[derive(Debug, PartialEq, Eq)]
pub(crate) enum LCDMode {
    /// OAM is inaccessible(except DMA) during this period.
    OamScan = 2,
    /// VRAM is inaccessible during this period.
    RenderPixel = 3,
    HBlank = 0,
    /// Everything is accessible during this period.
    VBlank = 1,
}

impl From<u8> for LCDMode {
    fn from(value: u8) -> Self {
        let value = value & 0b11;
        unsafe { std::mem::transmute::<[u8; 1], Self>([value]) }
    }
}

pub(crate) struct LCD {
    /// LCD control, at 0xFF40.
    /// - Bit 0: BG and Window enable/priority, 0=off, 1=on.
    /// - Bit 1: OBJ enable, 0=off, 1=on.
    /// - Bit 2: OBJ size, 0=8x8, 1=8x16.
    /// - Bit 3: BG tile map area, 0=0x9800-0x9BFF, 1=0x9C00-0x9FFF.
    /// - Bit 4: BG and Window tile data area(VRAM), 0=0x8800-0x97FF, 1=0x8000-0x8FFF.
    /// - Bit 5: Window enable, 0=off, 1=on
    /// - Bit 6: Window tile map area, 0=0x9800-0x9BFF, 1=0x9C00-0x9FFF.
    /// - Bit 7: LCD and PPU enable 0=off, 1=on.
    pub(crate) lcdc: u8,
    /// LCD status, at 0xFF41.
    /// - Bit 6 - LYC=LY STAT Interrupt source         (1=Enable) (Read/Write)
    /// - Bit 5 - Mode 2 OAM STAT Interrupt source     (1=Enable) (Read/Write)
    /// - Bit 4 - Mode 1 VBlank STAT Interrupt source  (1=Enable) (Read/Write)
    /// - Bit 3 - Mode 0 HBlank STAT Interrupt source  (1=Enable) (Read/Write)
    /// - Bit 2 - LYC=LY Flag                          (0=Different, 1=Equal) (Read Only)
    /// - Bit 1-0 - Mode Flag                          (Mode 0-3, see below) (Read Only)
    ///           0: HBlank
    ///           1: VBlank
    ///           2: Searching OAM
    ///           3: Transferring Data to LCD Controller
    pub(crate) stat: u8,
    /// Read only, LCD Y coordinate, at 0xFF44, representing current scanline.
    ///
    /// The value is in range \[0, 153].
    /// When it's in range \[144, 153], it's in VBlank period.
    pub(crate) ly: u8,
    /// LCD Y compare, at 0xFF45.
    /// When LYC == LY, LYC=LY flag is set, and (if enabled) a STAT interrupt is requested.
    pub(crate) lyc: u8,
    /// Window Y position, at 0xFF4A.
    pub(crate) wy: u8,
    /// Window X position plus 7, at 0xFF4B.
    pub(crate) wx: u8,
    /// Scroll(viewport) Y position, at 0xFF42.
    pub(crate) scy: u8,
    /// Scroll(viewport) X position, at 0xFF43.
    pub(crate) scx: u8,
}

impl Default for LCD {
    fn default() -> Self {
        Self { lcdc: 0b10010001, stat: 0b10, ly: 0, lyc: 0, wy: 0, wx: 0, scy: 0, scx: 0 }
    }
}

impl LCD {
    #[inline]
    pub(crate) fn object_size(&self) -> u8 {
        if is_bit_set!(self.lcdc, 2) {
            16
        } else {
            8
        }
    }

    pub(crate) fn is_bgw_enabled(&self) -> bool {
        is_bit_set!(self.lcdc, 0)
    }

    pub(crate) fn is_obj_enabled(&self) -> bool {
        is_bit_set!(self.lcdc, 1)
    }

    pub(crate) fn is_window_enabled(&self) -> bool {
        is_bit_set!(self.lcdc, 5)
    }
}
