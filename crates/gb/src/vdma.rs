//! VRAM DMA

use gb_shared::{is_bit_set, Memory, Snapshot};

/// VRAM takes 8 M-cycles to copy 16 bytes of data in normal speed mode,
/// and 16 M-cycles in double speed mode. Older MBC(like MBC1-3) and slower
/// ROMS(TODO: investigate) are not guaranteed to support General Purpose DMA(GDMA)
/// or HBlank DMA(HDMA).
#[derive(serde::Serialize, serde::Deserialize, Clone)]
pub(crate) struct Vdma {
    hdma1: u8,
    hdma2: u8,
    hdma3: u8,
    hdma4: u8,
    /// `true` for HDMA, `false` for GDMA.
    hdma: bool,
    /// Indicate whether DMA is in progress. Whatever the mode is,
    /// if `active` is false, then HDMA5 returns 0xFF.
    active: bool,
    /// Total remaining bytes to copy.
    remain: u16,
    /// While in HDMA, it's possible for the program to terminate
    /// the progress early via writing a value with Bit.7 set to 0 to HDMA5.
    /// In this case, HDMA5 returns the remaining length with Bit.7 set to 1.
    terminated: bool,
    src_addr: u16,
    dst_addr: u16,
    /// (LY, Remain(Reaming bytes to copy in current scanline))
    hblank_scanline: Option<(u8, u8)>,
}

impl Vdma {
    pub(crate) fn new() -> Self {
        Self {
            hdma1: 0xFF,
            hdma2: 0xF0,
            hdma3: 0x9F,
            hdma4: 0xF0,
            hdma: false,
            active: false,
            remain: 0,
            terminated: false,
            src_addr: 0,
            dst_addr: 0,
            hblank_scanline: None,
        }
    }
}

impl Vdma {
    pub(crate) fn active(&self, ly: u8, hblank: bool) -> bool {
        let mut active = self.active && !self.terminated;
        if self.hdma {
            active &= hblank;
            active &= self.hblank_scanline.is_none_or(|x| {
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

        if self.hdma {
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
        }

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

impl Memory for Vdma {
    fn write(&mut self, addr: u16, value: u8) {
        match addr {
            0xFF51 => self.hdma1 = value,
            0xFF52 => self.hdma2 = value,
            0xFF53 => self.hdma3 = value,
            0xFF54 => self.hdma4 = value,
            0xFF55 => {
                if self.active && !self.terminated {
                    assert!(self.hdma); // Only in HDMA can the program have a chance to write to HDMA5.
                    if !(is_bit_set!(value, 7)) {
                        self.terminated = true;
                    }
                    return;
                }

                let src_addr = {
                    let hi = self.hdma1;
                    let lo = self.hdma2 & 0xF0;

                    ((hi as u16) << 8) | (lo as u16)
                };
                assert!(src_addr <= 0xDFF0);
                let dst_addr = {
                    let hi = (self.hdma3 & 0x1F) | 0x80;
                    let lo = self.hdma4 & 0xF0;

                    ((hi as u16) << 8) | (lo as u16)
                };

                self.src_addr = src_addr;
                self.dst_addr = dst_addr;
                self.hdma = is_bit_set!(value, 7);
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
                if !self.hdma {
                    if self.active {
                        unreachable!("In GDMA, the program should have no chance to execute because the DMA copies all data once.")
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

pub(crate) type VdmaSnapshot = Vdma;

impl Snapshot for Vdma {
    type Snapshot = VdmaSnapshot;

    fn take_snapshot(&self) -> Self::Snapshot {
        self.clone()
    }

    fn restore_snapshot(&mut self, snapshot: Self::Snapshot) {
        (*self) = snapshot;
    }
}

#[cfg(test)]
mod tests {
    use gb_shared::ByteView;

    use super::*;

    const HDMA1: u16 = 0xFF51;
    const HDMA2: u16 = 0xFF52;
    const HDMA3: u16 = 0xFF53;
    const HDMA4: u16 = 0xFF54;
    const HDMA5: u16 = 0xFF55;

    struct DmaHelper {
        dma: Vdma,
    }

    impl DmaHelper {
        fn new() -> Self {
            DmaHelper { dma: Vdma::new() }
        }

        fn start(&mut self, hdma: bool, src_addr: u16, dst_addr: u16, length: u8) {
            self.dma.write(HDMA1, src_addr.msb());
            self.dma.write(HDMA2, src_addr.lsb());
            self.dma.write(HDMA3, dst_addr.msb());
            self.dma.write(HDMA4, dst_addr.lsb());
            self.dma.write(HDMA5, length | if hdma { 0x80 } else { 0 });
        }

        fn transfer(&mut self, ly: u8, hblank: bool) {
            for _ in 0..16 {
                assert!(self.dma.active(ly, hblank));
                assert!(self.dma.step(ly, hblank).is_some());
            }
        }
    }

    #[test]
    #[should_panic]
    fn src_address_range() {
        let mut helper = DmaHelper::new();
        helper.start(
            false,  //
            0xE034, // Invalid source address
            0x8005, 0x01,
        );
    }

    #[test]
    fn address_should_be_divied_by_16() {
        let mut helper = DmaHelper::new();
        helper.start(true, 0x1234, 0x8005, 0x01);
        let addr = helper.dma.step(0, true);
        assert_eq!(addr, Some((0x1230, 0x8000)));
    }

    mod hdma {
        use super::*;

        #[test]
        fn should_be_active_during_transferring_16_bytes() {
            let mut helper = DmaHelper::new();
            helper.start(true, 0x1230, 0x8000, 0x02);

            helper.transfer(0, true);
            assert!(!helper.dma.active(0, true));
        }

        #[test]
        #[should_panic]
        fn should_transfer_16_bytes_before_entering_new_scanline() {
            let mut helper = DmaHelper::new();
            helper.start(true, 0x1230, 0x8000, 0x02);

            // LY 0
            assert!(helper.dma.step(0, true).is_some());
            // LY 1
            helper.dma.step(1, true);
        }

        #[test]
        fn manual_terminate() {
            let mut helper = DmaHelper::new();
            helper.start(true, 0x1230, 0x8000, 0x02);
            helper.transfer(0, true);

            assert!(helper.dma.active(1, true));
            // Terminate
            helper.dma.write(HDMA5, 0x00);
            assert!(!helper.dma.active(1, true));
            assert_eq!(0x81, helper.dma.read(HDMA5));
        }

        #[test]
        fn should_be_able_to_start_new_transfer_after_terminated() {
            let mut helper = DmaHelper::new();
            helper.start(true, 0x1230, 0x8000, 0x02);
            helper.transfer(0, true);

            assert!(helper.dma.active(1, true));
            // Terminate
            helper.dma.write(HDMA5, 0x00);
            assert!(!helper.dma.active(1, true));
            assert_eq!(0x81, helper.dma.read(HDMA5));
            // Start new transfer
            helper.start(true, 0x1230, 0x8000, 0x02);
            helper.dma.active(0, true);
            assert_eq!(0x02, helper.dma.read(HDMA5));
        }

        #[test]
        #[should_panic]
        fn should_not_read_hdma5_during_transferring() {
            let mut helper = DmaHelper::new();
            helper.start(true, 0x1230, 0x8000, 0x02);

            helper.dma.step(0, true);
            helper.dma.read(HDMA5);
        }

        #[test]
        fn should_hdma5_return_remining_length() {
            let mut helper = DmaHelper::new();
            helper.start(true, 0x1230, 0x8000, 0x02);

            assert_eq!(0x02, helper.dma.read(HDMA5));
            helper.transfer(0, true);
            assert_eq!(0x01, helper.dma.read(HDMA5));
        }

        #[test]
        fn should_be_inactive_when_not_in_hblank() {
            let mut helper = DmaHelper::new();
            helper.start(true, 0x1230, 0x8000, 0x02);

            assert!(!helper.dma.active(0, false));
        }
    }

    mod gdma {
        use super::*;

        #[test]
        #[should_panic]
        fn gdma_should_transfer_all_data_at_once_failure() {
            let mut helper = DmaHelper::new();
            helper.start(false, 0x1230, 0x8000, 0x02);

            helper.transfer(0, false);
            helper.dma.read(HDMA5);
        }

        #[test]
        #[should_panic]
        fn gdma_should_transfer_all_data_at_once_success() {
            let mut helper = DmaHelper::new();
            helper.start(false, 0x1230, 0x8000, 0x02);

            helper.transfer(0, false);
            helper.transfer(0, false);
            helper.dma.read(HDMA5);
        }
    }
}
