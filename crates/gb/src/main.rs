use gb_cartridge::Cartridge;
use gb_cpu_sm83::Cpu;
use gb_ppu::PPU;
use gb_shared::boxed_array;
use gb_shared::Memory;
use log::debug;
use std::{cell::RefCell, rc::Rc};

struct WorkRam {
    /// [C000, E000)
    /// (4 + 4)KiB
    ram: Box<[u8; 0x2000]>,
}

impl WorkRam {
    fn new() -> Self {
        Self { ram: boxed_array(0) }
    }
}

impl Memory for WorkRam {
    fn write(&mut self, addr: u16, value: u8) {
        debug_assert!((0xC000..=0xDFFF).contains(&addr));

        let addr = (addr as usize) - 0xC000;
        self.ram[addr] = value;
    }

    fn read(&self, addr: u16) -> u8 {
        debug_assert!((0xC000..=0xDFFF).contains(&addr));

        let addr = (addr as usize) - 0xC000;
        self.ram[addr]
    }
}

struct HighRam {
    /// [FF80, FFFF)
    ram: Box<[u8; 0x80]>,
}

impl HighRam {
    fn new() -> Self {
        Self { ram: boxed_array(0) }
    }
}

impl Memory for HighRam {
    fn write(&mut self, addr: u16, value: u8) {
        debug_assert!((0xFF80..=0xFFFE).contains(&addr));

        let addr = (addr as usize) - 0xFF80;
        self.ram[addr] = value;
    }

    fn read(&self, addr: u16) -> u8 {
        debug_assert!((0xFF80..=0xFFFE).contains(&addr));

        let addr = (addr as usize) - 0xFF80;
        self.ram[addr]
    }
}

struct Bus {
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
    ppu: PPU,
}

impl Memory for Bus {
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
            0xE000..=0xFDFF => unreachable!("Unusable ECHO RAM [0xE000, 0xFDFF]"),
            0xFE00..=0xFE9F => {
                // OAM
                self.ppu.write(addr, value);
            }
            0xFEA0..=0xFEFF => unreachable!("Unusable memory [0xFEA0, 0xFEFF]"),
            0xFF0F => {
                // IF
                self.interrupt_flag = value
            }
            0xFF41..=0xFF4B => {
                self.ppu.write(addr, value);
            }
            0xFF80..=0xFFFE => {
                // HRAM
                self.hram.write(addr, value);
            }
            0xFFFF => {
                // IE
                self.interrupt_enable = value;
            }
            // TODO [FF00, FF7F] IO registers.
            _ => {}
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
            0xE000..=0xFDFF => unreachable!("Unusable ECHO RAM [0xE000, 0xFDFF]"),
            0xFE00..=0xFE9F => {
                // OAM
                self.ppu.read(addr)
            }
            0xFEA0..=0xFEFF => unreachable!("Unusable memory [0xFEA0, 0xFEFF]"),
            0xFF0F => {
                // IF
                self.interrupt_flag
            }
            0xFF41..=0xFF4B => self.ppu.read(addr),
            0xFF80..=0xFFFE => {
                // HRAM
                self.hram.read(addr)
            }
            0xFFFF => {
                // IE
                self.interrupt_enable
            }
            // TODO [FF00, FF7F] IO registers.
            _ => 0,
        };

        debug!("bus read at {:#04X}, value: {:#04X}", addr, value);
        value
    }
}

impl Bus {
    fn step(&mut self, cycles: u8) {
        for _ in 0..cycles {
            for _ in 0..4 {
                self.ppu.step();
            }
        }
    }
}

struct GameBoy {
    bus: Rc<RefCell<Bus>>,
    cpu: Cpu<Bus>,
}

impl GameBoy {
    fn new(cart: Cartridge) -> Self {
        let bus = Rc::new(RefCell::new(Bus {
            cart,
            wram: WorkRam::new(),
            hram: HighRam::new(),
            interrupt_enable: 0,
            interrupt_flag: 0,
            ppu: PPU::new(),
        }));

        let cpu = Cpu::new(bus.clone());
        GameBoy { bus, cpu }
    }
}

fn main() {
    env_logger::init();

    let cart = Cartridge::load(std::env::current_dir().unwrap().join("roms").join("cpu_instrs.gb"))
        .unwrap();

    let mut gb = GameBoy::new(cart);
    for _ in 1..20 {
        debug!("{:?}", &gb.cpu);

        if gb.cpu.stopped {
            println!("Stopping...");
            // TODO
            std::process::exit(0);
        }

        if gb.cpu.interrupt_master_enable {
            gb.cpu.handle_interrupts();
        }

        let cycles = gb.cpu.step();
        gb.bus.borrow_mut().step(cycles);
    }
}
