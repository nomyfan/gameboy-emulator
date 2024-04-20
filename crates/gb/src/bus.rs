use gb_apu::{Apu, ApuSnapshot};
use gb_cartridge::Cartridge;
use gb_ppu::{Ppu, PpuSnapshot};
use gb_shared::{command::Command, Memory, Snapshot};
use std::ops::{Deref, DerefMut};

use crate::{
    dma::{DmaSnapshot, DMA},
    hram::{HighRam, HighRamSnapshot},
    joypad::{Joypad, JoypadSnapshot},
    serial::{Serial, SerialSnapshot},
    timer::{Timer, TimerSnapshot},
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
    cart: Cartridge,
    wram: WorkRam,
    hram: HighRam,
    /// DMA state.
    dma: DMA,
    /// Serial transfer
    serial: Serial,
    joypad: Joypad,
    timer: Timer,
    pub(crate) ppu: Ppu,
    pub(crate) apu: Apu,
    ref_count: usize,
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
                    _ => {
                        log::error!("Unsupported");
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
                    0xFF40..=0xFF4B => self.ppu.read(addr),
                    _ => {
                        log::error!("Unsupported");
                        0
                    }
                }
            }
            // Exclude 0xFF46(DMA)
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
    ptr: *mut BusInner,
}

impl Deref for Bus {
    type Target = BusInner;

    fn deref(&self) -> &Self::Target {
        unsafe { self.ptr.as_ref().unwrap() }
    }
}

impl DerefMut for Bus {
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { self.ptr.as_mut().unwrap() }
    }
}

impl Bus {
    pub(crate) fn new(cart: Cartridge, sample_rate: Option<u32>) -> Self {
        Self {
            ptr: Box::into_raw(Box::new(BusInner {
                cart,
                wram: WorkRam::new(),
                hram: HighRam::new(),
                interrupt_enable: 0,
                interrupt_flag: 0xE0,
                dma: DMA::new(),
                serial: Serial::new(),
                joypad: Joypad::new(),
                timer: Timer::new(),
                ppu: Ppu::new(),
                apu: Apu::new(sample_rate),
                ref_count: 1,
            })),
        }
    }

    fn step_dma(&mut self) {
        if let Some((src, dst)) = self.dma.next_addr() {
            let value = self.read(src);
            self.ppu.write(dst, value)
        }
    }

    fn set_irq(&mut self, irq: u8) {
        self.write(0xFF0F, self.read(0xFF0F) | irq);
    }

    pub(crate) fn step_timer(&mut self) {
        self.timer.step();
    }

    pub(crate) fn handle_command(&mut self, command: Command) {
        if let Command::Joypad(joypad_command) = command {
            self.joypad.handle_command(joypad_command);
            let irq = self.joypad.take_irq();
            self.set_irq(irq);
        }
    }
}

impl gb_shared::Component for Bus {
    fn step(&mut self, cycles: u8) {
        let m_cycles = cycles / 4;
        debug_assert!(m_cycles > 0);

        for _ in 0..m_cycles {
            for _ in 0..4 {
                self.ppu.step();
                let irq = self.ppu.take_irq();
                self.set_irq(irq);

                self.step_timer();
                let irq = self.timer.take_irq();
                self.set_irq(irq);

                self.apu.step();
            }

            // It costs 160 machine cycles to transfer 160 bytes of data.
            // https://gbdev.io/pandocs/OAM_DMA_Transfer.html#ff46--dma-oam-dma-source-address--start:~:text=the%20transfer%20takes%20160%20machine%20cycles
            self.step_dma();
        }
    }
}

impl Clone for Bus {
    fn clone(&self) -> Self {
        unsafe {
            (*self.ptr).ref_count += 1;
        }
        Self { ptr: self.ptr }
    }
}

impl Drop for Bus {
    fn drop(&mut self) {
        if let Some(inner) = unsafe { self.ptr.as_mut() } {
            inner.ref_count -= 1;
            if inner.ref_count == 0 {
                unsafe {
                    // Deallocate the inner struct.
                    let _ = Box::from_raw(self.ptr);
                }
            }
        }
    }
}

impl Memory for Bus {
    fn write(&mut self, addr: u16, value: u8) {
        unsafe { (*(self.ptr)).write(addr, value) }
    }

    fn read(&self, addr: u16) -> u8 {
        unsafe { (*self.ptr).read(addr) }
    }
}

#[derive(serde::Serialize, serde::Deserialize)]
pub(crate) struct BusSnapshot {
    interrupt_enable: u8,
    interrupt_flag: u8,
    wram: WorkRamSnapshot,
    hram: HighRamSnapshot,
    dma: DmaSnapshot,
    serial: SerialSnapshot,
    joypad: JoypadSnapshot,
    timer: TimerSnapshot,
    ppu: PpuSnapshot,
    apu: ApuSnapshot,
    cart: Vec<u8>,
}

impl Snapshot for Bus {
    type Snapshot = BusSnapshot;

    fn take_snapshot(&self) -> Self::Snapshot {
        BusSnapshot {
            interrupt_enable: self.interrupt_enable,
            interrupt_flag: self.interrupt_flag,
            wram: self.wram.take_snapshot(),
            hram: self.hram.take_snapshot(),
            dma: self.dma.take_snapshot(),
            serial: self.serial.take_snapshot(),
            joypad: self.joypad.take_snapshot(),
            timer: self.timer.take_snapshot(),
            ppu: self.ppu.take_snapshot(),
            apu: self.apu.take_snapshot(),
            cart: self.cart.take_snapshot(),
        }
    }

    fn restore_snapshot(&mut self, snapshot: Self::Snapshot) {
        self.interrupt_enable = snapshot.interrupt_enable;
        self.interrupt_flag = snapshot.interrupt_flag;
        self.wram.restore_snapshot(snapshot.wram);
        self.hram.restore_snapshot(snapshot.hram);
        self.dma.restore_snapshot(snapshot.dma);
        self.serial.restore_snapshot(snapshot.serial);
        self.joypad.restore_snapshot(snapshot.joypad);
        self.timer.restore_snapshot(snapshot.timer);
        self.ppu.restore_snapshot(snapshot.ppu);
        self.apu.restore_snapshot(snapshot.apu);
        self.cart.restore_snapshot(snapshot.cart);
    }
}
