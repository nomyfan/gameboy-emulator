//! VRAM DMA

use gb_shared::{is_bit_set, Memory, Snapshot};

/// VRAM takes 8 M-cycles to copy 16 bytes of data in normal speed mode,
/// and 16 M-cycles in double speed mode. Older MBC(like MBC1-3) and slower
/// ROMS(TODO: investigate) are not guaranteed to support General Purpose DMA or HBlank DMA.
pub(crate) struct Hdma {
    hdma1: u8,
    hdma2: u8,
    hdma3: u8,
    hdma4: u8,
    hblank_mode: bool,
    /// Indicate whether DMA is in progress. Whatever the mode is,
    /// if `active` is false, then HDMA5 returns 0xFF.
    active: bool,
    /// Total remaining bytes to copy.
    remain: u16,
    /// While in HBlank DMA, it's possible for the program to terminate
    /// the progress early via writing a value with Bit.7 set to 0 to HDMA5.
    /// In this case, HDMA5 returns the remaining length with Bit.7 set to 1.
    terminated: bool,
    src_addr: u16,
    dst_addr: u16,
    /// (LY, Remain(Reaming bytes to copy in current scanline))
    hblank_scanline: Option<(u8, u8)>,
}

impl Hdma {
    pub(crate) fn new() -> Self {
        Self {
            hdma1: 0xFF,
            hdma2: 0xF0,
            hdma3: 0x9F,
            hdma4: 0xF0,
            hblank_mode: false,
            active: false,
            remain: 0,
            terminated: false,
            src_addr: 0,
            dst_addr: 0,
            hblank_scanline: None,
        }
    }
}

impl Hdma {
    pub(crate) fn active(&self, ly: u8, hblank: bool) -> bool {
        let mut active = self.active && !self.terminated;
        if self.hblank_mode {
            active &= hblank;
            active &= self.hblank_scanline.map_or(true, |x| {
                if x.0 == ly {
                    x.1 > 0
                } else {
                    assert_eq!(x.1, 0);
                    true
                }
            })
        }

        active
    }

    pub(crate) fn step(&mut self, ly: u8, hblank: bool) -> Option<(u16, u16)> {
        if !self.active(ly, hblank) {
            return None;
        }
        assert!(self.remain > 0);

        if self.hblank_mode {
            match self.hblank_scanline.as_mut() {
                Some(scanline) => {
                    if scanline.1 == 0 {
                        (*scanline) = (ly, 16);
                    }
                }
                None => self.hblank_scanline = Some((ly, 16)),
            }
        }

        let src_addr = self.src_addr;
        let dst_addr = self.dst_addr;

        self.src_addr += 1;
        self.dst_addr += 1;
        self.remain -= 1;
        if let Some(x) = self.hblank_scanline.as_mut() {
            x.1 -= 1;
            if x.1 == 0 {
                assert_eq!(self.remain % 16, 0);
            }
        }

        // TODO: https://gbdev.io/pandocs/CGB_Registers.html#documented-registers:~:text=hblank%20dma%20should%20not%20be%20started%20(write%20to%20ff55)%20during%20a%20hblank%20period%20(stat%20mode%200).

        // The status of this case is unknown. See https://gbdev.io/pandocs/CGB_Registers.html#documented-registers:~:text=if%20the%20transfer%E2%80%99s%20destination%20address%20overflows
        if self.dst_addr > 0x9FFF {
            self.remain = 0;
        }

        if self.remain == 0 {
            self.active = false;
            self.hblank_scanline = None;
        }

        Some((src_addr, dst_addr))
    }
}

impl Memory for Hdma {
    fn write(&mut self, addr: u16, value: u8) {
        match addr {
            0xFF51 => self.hdma1 = value,
            0xFF52 => self.hdma2 = value,
            0xFF53 => self.hdma3 = value,
            0xFF54 => self.hdma4 = value,
            0xFF55 => {
                if self.active {
                    assert!(self.hblank_mode); // Only in mode1 can the program have a chance to write to HDMA5.
                    if is_bit_set!(value, 7) {
                        self.terminated = true;
                    }
                    return;
                }

                let src_addr = {
                    let hi = self.hdma1;
                    let lo = self.hdma2 & 0xF0;

                    ((hi as u16) << 8) | (lo as u16)
                };
                assert!(src_addr <= 0xDFF0); // TODO:
                let dst_addr = {
                    let hi = (self.hdma3 & 0x1F) | 0x80;
                    let lo = self.hdma4 & 0xF0;

                    ((hi as u16) << 8) | (lo as u16)
                };

                self.src_addr = src_addr;
                self.dst_addr = dst_addr;
                self.hblank_mode = is_bit_set!(value, 7);
                self.active = true;
                self.remain = ((value & 0x7F) + 1) as u16 * 16;
                self.hblank_scanline = None;
                self.terminated = false;
            }
            _ => unreachable!("Invalid HDMA write at {:#X} {:#X}", addr, value),
        }
    }

    fn read(&self, addr: u16) -> u8 {
        match addr {
            // 0xFF51-0xFF54 is write-only.
            0xFF55 => {
                if !self.hblank_mode {
                    if self.active {
                        unreachable!("In General Purpose DMA mode, the program should have no chance to execute because the DMA copies all data once.")
                    }
                    // Completed
                    0xFF
                } else {
                    let mut len = if self.active {
                        assert_eq!(self.remain % 16, 0);
                        (self.remain / 16 - 1) as u8
                    } else {
                        0xFF
                    };
                    if self.terminated {
                        len |= 0x80;
                    }
                    len
                }
            }
            _ => unreachable!("Invalid HDMA read at {:#X}", addr),
        }
    }
}

#[derive(serde::Serialize, serde::Deserialize)]
pub(crate) struct HdmaSnapshot;

impl Snapshot for Hdma {
    type Snapshot = HdmaSnapshot;

    fn take_snapshot(&self) -> Self::Snapshot {
        todo!()
    }

    fn restore_snapshot(&mut self, snapshot: Self::Snapshot) {
        todo!()
    }
}
