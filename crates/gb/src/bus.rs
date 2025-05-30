use gb_apu::{Apu, ApuSnapshot};
use gb_cartridge::Cartridge;
use gb_ppu::{Ppu, PpuSnapshot};
use gb_shared::{command::Command, Memory, Snapshot};
use std::ops::{Deref, DerefMut};

use crate::{
    dma::{DmaSnapshot, DMA},
    hram::{HighRam, HighRamSnapshot},
    joypad::Joypad,
    misc_ram::{MiscRam, MiscRamSnapshot},
    serial::{Serial, SerialSnapshot},
    timer::{Timer, TimerSnapshot},
    vdma::{Vdma, VdmaSnapshot},
    wram::{WorkRam, WorkRamSnapshot},
};

pub(crate) struct BusInner {
    /// R/W. Set the bit to be 1 if the corresponding
    /// interrupt is enabled. Lower bits have higher
    /// priorities.
    ///
    /// - Bit 4, Joypad
    /// - Bit 3, Serial
    /// - Bit 2, Timer
    /// - Bit 1, LCD STAT
    /// - Bit 0, Vertical Blank
    interrupt_enable: u8,
    /// R/W. Set the bit to be 1 if the corresponding
    /// interrupt is requested. Lower bits have higher
    /// priorities.
    ///
    /// - Bit 4, Joypad
    /// - Bit 3, Serial
    /// - Bit 2, Timer
    /// - Bit 1, LCD STAT
    /// - Bit 0, Vertical Blank
    interrupt_flag: u8,
    pub(crate) cart: Cartridge,
    wram: WorkRam,
    hram: HighRam,
    dma: DMA,
    vdma: Vdma,
    mram: MiscRam,
    /// Serial transfer
    serial: Serial,
    joypad: Joypad,
    timer: Timer,
    clocks: u8,
    pub(crate) ppu: Ppu,
    pub(crate) apu: Apu,
}

impl Memory for BusInner {
    fn write(&mut self, addr: u16, value: u8) {
        match addr {
            0x0000..=0x7FFF => {
                // ROM data
                self.cart.write(addr, value);
            }
            0x8000..=0x9FFF => {
                // VRAM
                self.ppu.write(addr, value);
            }
            0xA000..=0xBFFF => {
                // EXT-RAM, from cartridge
                self.cart.write(addr, value);
            }
            0xC000..=0xDFFF => {
                // WRAM
                self.wram.write(addr, value);
            }
            0xE000..=0xFDFF => log::warn!("Unusable ECHO RAM [0xE000, 0xFDFF]"),
            0xFE00..=0xFE9F => {
                if self.dma.active() {
                    return;
                }
                // OAM
                self.ppu.write(addr, value);
            }
            0xFEA0..=0xFEFF => log::warn!("Unusable memory [0xFEA0, 0xFEFF]"),
            0xFF00..=0xFF7F => {
                match addr {
                    0xFF00 => self.joypad.write(addr, value),
                    0xFF01..=0xFF02 => self.serial.write(addr, value),
                    0xFF04..=0xFF07 => self.timer.write(addr, value),
                    0xFF0F => {
                        // IF
                        self.interrupt_flag = 0xE0 | value
                    }
                    0xFF10..=0xFF3F => {
                        self.apu.write(addr, value);
                    }
                    0xFF46 => {
                        // DMA
                        self.dma.write(addr, value);
                    }
                    // Exclude 0xFF46(DMA)
                    0xFF40..=0xFF4B => {
                        self.ppu.write(addr, value);
                    }
                    0xFF4D => {
                        log::error!("Switch speed is not supported yet, {}", value);
                    }
                    // VRAM bank(VBK)
                    0xFF4F => self.ppu.write(addr, value),
                    0xFF51..=0xFF54 => self.vdma.write(addr, value),
                    0xFF55 => {
                        // @see https://gbdev.io/pandocs/CGB_Registers.html#documented-registers:~:text=hblank%20dma%20should%20not%20be%20started%20(write%20to%20ff55)%20during%20a%20hblank%20period
                        if self.ppu.lcd_mode().hblank() {
                            log::error!("HBlank DMA should not be started during HBlank period.");
                            return;
                        }
                        self.vdma.write(addr, value);
                    }
                    0xFF56 => {
                        log::warn!("RP is not supported yet");
                    }
                    // BCPS, BCPD, OCPS, OCPD
                    0xFF68..=0xFF6B => self.ppu.write(addr, value),
                    // OPRI
                    0xFF6C => self.ppu.write(addr, value),
                    // WRAM bank(SVBK)
                    0xFF70 => self.wram.write(addr, value),
                    _ => {
                        log::error!("Unsupported bus write {:#X} {:#X}", addr, value);
                    }
                }
            }
            0xFF80..=0xFFFE => {
                // HRAM
                self.hram.write(addr, value);
            }
            0xFFFF => {
                // IE
                self.interrupt_enable = value;
            }
        }
    }

    fn read(&self, addr: u16) -> u8 {
        match addr {
            0x0000..=0x7FFF => {
                // ROM data
                self.cart.read(addr)
            }
            0x8000..=0x9FFF => {
                // VRAM
                self.ppu.read(addr)
            }
            0xA000..=0xBFFF => {
                // EXT-RAM, from cartridge
                self.cart.read(addr)
            }
            0xC000..=0xDFFF => {
                // WRAM
                self.wram.read(addr)
            }
            0xE000..=0xFDFF => {
                log::warn!("Unusable ECHO RAM [0xE000, 0xFDFF]");
                0
            }
            0xFE00..=0xFE9F => {
                if self.dma.active() {
                    return 0xFF;
                }

                // OAM
                self.ppu.read(addr)
            }
            0xFEA0..=0xFEFF => {
                log::warn!("Unusable memory [0xFEA0, 0xFEFF]");
                0
            }
            0xFF00..=0xFF7F => {
                match addr {
                    0xFF00 => self.joypad.read(addr),
                    0xFF01..=0xFF02 => self.serial.read(addr),
                    0xFF04..=0xFF07 => self.timer.read(addr),
                    0xFF0F => {
                        // IF
                        self.interrupt_flag
                    }
                    0xFF10..=0xFF3F => self.apu.read(addr),
                    0xFF46 => self.dma.read(addr),
                    // Exclude 0xFF46(DMA)
                    0xFF40..=0xFF4B => self.ppu.read(addr),
                    // TODO: Switch speed.
                    0xFF4D => 0x00,
                    // VRAM bank(VBK)
                    0xFF4F => self.ppu.read(addr),
                    0xFF51..=0xFF55 => self.vdma.read(addr),
                    0xFF56 => {
                        log::warn!("RP is not supported yet");
                        0
                    }
                    // BCPS, BCPD, OCPS, OCPD
                    0xFF68..=0xFF6B => self.ppu.read(addr),
                    0xFF6C => self.ppu.read(addr),
                    // WRAM bank(SVBK)
                    0xFF70 => self.wram.read(addr),
                    _ => {
                        log::error!("Unsupported bus read {:#X}", addr);
                        0
                    }
                }
            }
            0xFF80..=0xFFFE => {
                // HRAM
                self.hram.read(addr)
            }
            0xFFFF => {
                // IE
                self.interrupt_enable
            }
        }
    }
}

pub(crate) struct Bus {
    inner: std::rc::Rc<std::cell::UnsafeCell<BusInner>>,
}

impl Deref for Bus {
    type Target = BusInner;

    fn deref(&self) -> &Self::Target {
        unsafe { &*self.inner.get() }
    }
}

impl DerefMut for Bus {
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { &mut *self.inner.get() }
    }
}

impl Bus {
    pub(crate) fn new(cart: Cartridge, sample_rate: Option<u32>) -> Self {
        let machine_model = cart.machine_model();
        let compatibility_palette_id = cart.compatibility_palette_id();
        Self {
            inner: std::rc::Rc::new(std::cell::UnsafeCell::new(BusInner {
                cart,
                wram: WorkRam::new(machine_model),
                hram: HighRam::new(),
                interrupt_enable: 0,
                interrupt_flag: 0xE0,
                dma: DMA::new(),
                serial: Serial::new(),
                joypad: Joypad::new(),
                timer: Timer::new(),
                ppu: Ppu::new(machine_model, compatibility_palette_id),
                apu: Apu::new(sample_rate),
                vdma: Vdma::new(),
                mram: MiscRam::new(machine_model),
                clocks: 0,
            })),
        }
    }

    fn step_dma(&mut self) {
        if let Some((src, dst)) = self.dma.next_addr() {
            let value = self.read(src);
            self.ppu.write(dst, value)
        }
    }

    fn request_interrupt(&mut self, irq: u8) {
        self.write(0xFF0F, self.read(0xFF0F) | irq);
    }

    pub(crate) fn step_timer(&mut self) {
        self.timer.step();
    }

    pub(crate) fn exec_command(&mut self, command: Command) {
        let Command::MutateJoypadButtons(keys) = command;
        self.joypad.mutate_buttons(keys);
        let irq = self.joypad.take_irq();
        self.request_interrupt(irq);
    }
}

impl gb_shared::Bus for Bus {
    fn step(&mut self, clocks: u8) {
        let clocks = self.clocks + clocks;
        let m_cycles = clocks / 4;
        self.clocks = clocks % 4;
        debug_assert!(m_cycles > 0);

        for _ in 0..m_cycles {
            for _ in 0..4 {
                self.ppu.step();
                let irq = self.ppu.take_irq();
                self.request_interrupt(irq);

                self.step_timer();
                let irq = self.timer.take_irq();
                self.request_interrupt(irq);

                self.apu.step();
            }

            // It costs 160 machine cycles to transfer 160 bytes of data.
            // https://gbdev.io/pandocs/OAM_DMA_Transfer.html#ff46--dma-oam-dma-source-address--start:~:text=the%20transfer%20takes%20160%20machine%20cycles
            self.step_dma();
        }
    }

    fn vdma_active(&self) -> bool {
        let ly = self.ppu.ly();
        let hblank = self.ppu.lcd_mode().hblank();
        self.vdma.active(ly, hblank)
    }

    fn step_vdma(&mut self) {
        let ly = self.ppu.ly();
        let hblank = self.ppu.lcd_mode().hblank();
        if !self.vdma.active(ly, hblank) {
            return;
        }

        if let Some((src_addr, dst_addr)) = self.vdma.step(ly, hblank) {
            let value = self.read(src_addr);
            self.ppu.write(dst_addr, value);
        }
    }
}

impl Clone for Bus {
    fn clone(&self) -> Self {
        Self { inner: self.inner.clone() }
    }
}

impl Memory for Bus {
    #[inline]
    fn write(&mut self, addr: u16, value: u8) {
        self.deref_mut().write(addr, value);
    }

    #[inline]
    fn read(&self, addr: u16) -> u8 {
        self.deref().read(addr)
    }
}

#[derive(serde::Serialize, serde::Deserialize)]
pub(crate) struct BusSnapshot {
    interrupt_enable: u8,
    interrupt_flag: u8,
    cart: Vec<u8>,
    wram: WorkRamSnapshot,
    hram: HighRamSnapshot,
    dma: DmaSnapshot,
    vdma: VdmaSnapshot,
    mram: MiscRamSnapshot,
    serial: SerialSnapshot,
    timer: TimerSnapshot,
    clocks: u8,
    ppu: PpuSnapshot,
    apu: ApuSnapshot,
}

impl Snapshot for Bus {
    type Snapshot = BusSnapshot;

    fn take_snapshot(&self) -> Self::Snapshot {
        BusSnapshot {
            interrupt_enable: self.interrupt_enable,
            interrupt_flag: self.interrupt_flag,
            cart: self.cart.take_snapshot(),
            wram: self.wram.take_snapshot(),
            hram: self.hram.take_snapshot(),
            dma: self.dma.take_snapshot(),
            vdma: self.vdma.take_snapshot(),
            mram: self.mram.take_snapshot(),
            serial: self.serial.take_snapshot(),
            timer: self.timer.take_snapshot(),
            clocks: self.clocks,
            ppu: self.ppu.take_snapshot(),
            apu: self.apu.take_snapshot(),
        }
    }

    fn restore_snapshot(&mut self, snapshot: Self::Snapshot) {
        self.interrupt_enable = snapshot.interrupt_enable;
        self.interrupt_flag = snapshot.interrupt_flag;
        self.cart.restore_snapshot(snapshot.cart);
        self.wram.restore_snapshot(snapshot.wram);
        self.hram.restore_snapshot(snapshot.hram);
        self.dma.restore_snapshot(snapshot.dma);
        self.vdma.restore_snapshot(snapshot.vdma);
        self.mram.restore_snapshot(snapshot.mram);
        self.serial.restore_snapshot(snapshot.serial);
        self.timer.restore_snapshot(snapshot.timer);
        self.clocks = snapshot.clocks;
        self.ppu.restore_snapshot(snapshot.ppu);
        self.apu.restore_snapshot(snapshot.apu);
    }
}
