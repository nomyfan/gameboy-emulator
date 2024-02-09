use gb_cartridge::Cartridge;
use gb_ppu::PPU;
use gb_shared::{command::Command, Memory};
use log::debug;
use std::ops::{Deref, DerefMut};

use crate::{dma::DMA, hram::HighRam, joypad::Joypad, serial::Serial, timer::Timer, wram::WorkRam};

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
    ppu: PPU,

    ref_count: usize,
}

impl Memory for BusInner {
    fn write(&mut self, addr: u16, value: u8) {
        debug!("bus write at {:#04X}, value: {:#02X}", addr, value);

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
            0xE000..=0xFDFF => debug!("Unusable ECHO RAM [0xE000, 0xFDFF]"),
            0xFE00..=0xFE9F => {
                if self.dma.active() {
                    return;
                }
                // OAM
                self.ppu.write(addr, value);
            }
            0xFEA0..=0xFEFF => debug!("Unusable memory [0xFEA0, 0xFEFF]"),
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
                        // TODO: Sound
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
                        debug!("Unsupported");
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
        let value = match addr {
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
                debug!("Unusable ECHO RAM [0xE000, 0xFDFF]");
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
                debug!("Unusable memory [0xFEA0, 0xFEFF]");
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
                    0xFF10..=0xFF3F => {
                        // TODO: Sound
                        0
                    }
                    0xFF46 => self.dma.read(addr),
                    0xFF40..=0xFF4B => self.ppu.read(addr),
                    _ => {
                        debug!("Unsupported");
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
        };

        debug!("bus read at {:#04X}, value: {:#04X}", addr, value);
        value
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
    pub(crate) fn new(cart: Cartridge) -> Self {
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
                ppu: PPU::new(),
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

    pub(crate) fn set_frame_out_handle(
        &mut self,
        frame_out_handle: Option<Box<gb_shared::FrameOutHandle>>,
    ) {
        self.ppu.set_frame_out_handle(frame_out_handle);
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
