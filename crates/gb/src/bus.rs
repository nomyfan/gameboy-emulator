use gb_cartridge::Cartridge;
use gb_ppu::PPU;
use gb_shared::{InterruptRequest, Memory};
use log::debug;

use crate::{dma::DMA, hram::HighRam, joypad::Joypad, serial::Serial, timer::Timer, wram::WorkRam};

struct BusInner {
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

    ppu_ptr: *const PPU<Bus>,
    timer_ptr: *const Timer<Bus>,

    ref_count: usize,
}

impl BusInner {
    #[inline]
    fn ppu_mut(&mut self) -> &mut PPU<Bus> {
        unsafe { &mut *(self.ppu_ptr as *mut PPU<Bus>) }
    }

    #[inline]
    fn ppu(&self) -> &PPU<Bus> {
        unsafe { &*self.ppu_ptr }
    }

    #[inline]
    fn timer_mut(&mut self) -> &mut Timer<Bus> {
        unsafe { &mut *(self.timer_ptr as *mut Timer<Bus>) }
    }

    #[inline]
    fn timer(&self) -> &Timer<Bus> {
        unsafe { &*self.timer_ptr }
    }
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
                self.ppu_mut().write(addr, value);
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
                if !self.dma.active() {
                    // OAM
                    self.ppu_mut().write(addr, value);
                }
            }
            0xFEA0..=0xFEFF => debug!("Unusable memory [0xFEA0, 0xFEFF]"),
            0xFF00..=0xFF7F => {
                match addr {
                    0xFF00 => self.joypad.write(addr, value),
                    0xFF01..=0xFF02 => self.serial.write(addr, value),
                    0xFF04..=0xFF07 => self.timer_mut().write(addr, value),
                    0xFF0F => {
                        // IF
                        self.interrupt_flag = value
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
                        self.ppu_mut().write(addr, value);
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
                self.ppu().read(addr)
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
                self.ppu().read(addr)
            }
            0xFEA0..=0xFEFF => {
                debug!("Unusable memory [0xFEA0, 0xFEFF]");
                0
            }
            0xFF00..=0xFF7F => {
                match addr {
                    0xFF00 => self.joypad.read(addr),
                    0xFF01..=0xFF02 => self.serial.read(addr),
                    0xFF04..=0xFF07 => self.timer().read(addr),
                    0xFF0F => {
                        // IF
                        self.interrupt_flag
                    }
                    0xFF10..=0xFF3F => {
                        // TODO: Sound
                        0
                    }
                    0xFF46 => self.dma.read(addr),
                    0xFF40..=0xFF4B => self.ppu().read(addr),
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

impl Bus {
    #[inline]
    fn inner_mut(&mut self) -> &mut BusInner {
        unsafe { self.ptr.as_mut().expect("TODO:") }
    }

    pub(crate) fn new(cart: Cartridge) -> Self {
        Self {
            ptr: Box::into_raw(Box::new(BusInner {
                cart,
                wram: WorkRam::new(),
                hram: HighRam::new(),
                interrupt_enable: 0,
                interrupt_flag: 0,
                dma: DMA::new(),
                serial: Serial::new(),
                joypad: Joypad::new(),
                ppu_ptr: std::ptr::null(),
                timer_ptr: std::ptr::null(),
                ref_count: 1,
            })),
        }
    }

    pub(crate) fn set_ppu(&mut self, ppu: *const PPU<Bus>) {
        unsafe {
            (*self.ptr).ppu_ptr = ppu;
        }
    }

    pub(crate) fn set_timer(&mut self, timer: *const Timer<Bus>) {
        unsafe {
            (*self.ptr).timer_ptr = timer;
        }
    }

    fn step_dma(&mut self) {
        if let Some((src, dst)) = self.inner_mut().dma.next_addr() {
            let value = self.read(src);
            self.write(dst, value);
        }
    }

    pub(crate) fn step(&mut self) {
        self.step_dma();
        self.inner_mut().timer_mut().step();
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

impl InterruptRequest for Bus {
    fn request(&mut self, interrupt_type: gb_shared::InterruptType) {
        self.write(0xFF0F, self.read(0xFF0F) | interrupt_type as u8);
    }
}
