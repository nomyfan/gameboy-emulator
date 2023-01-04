use log::debug;

struct WorkRam {
    /// [C000, E000)
    /// (4 + 4)KiB
    ram: Box<[u8]>,
}

impl WorkRam {
    fn new() -> Self {
        Self { ram: vec![0u8; 0x2000].into_boxed_slice() }
    }
}

impl io::IO for WorkRam {
    fn write(&mut self, addr: u16, value: u8) {
        debug_assert!(addr >= 0xC000 && addr <= 0xDFFF);

        let addr = (addr as usize) - 0xC000;
        self.ram[addr] = value;
    }

    fn read(&self, addr: u16) -> u8 {
        debug_assert!(addr >= 0xC000 && addr <= 0xDFFF);

        let addr = (addr as usize) - 0xC000;
        self.ram[addr]
    }
}

struct HighRam {
    /// [FF80, FFFF)
    ram: Box<[u8]>,
}

impl HighRam {
    fn new() -> Self {
        Self { ram: vec![0; 0x80].into_boxed_slice() }
    }
}

impl io::IO for HighRam {
    fn write(&mut self, addr: u16, value: u8) {
        debug_assert!(addr >= 0xFF80 && addr <= 0xFFFE);

        let addr = (addr as usize) - 0xFF80;
        self.ram[addr] = value;
    }

    fn read(&self, addr: u16) -> u8 {
        debug_assert!(addr >= 0xFF80 && addr <= 0xFFFE);

        let addr = (addr as usize) - 0xFF80;
        self.ram[addr]
    }
}

struct Interrupts {
    /// R/W. Set the bit to be 1 if interrupt
    /// is enabled
    ///
    /// - Bit 4, Joypad
    /// - Bit 3, Serial
    /// - Bit 2, Timer
    /// - Bit 1, LCD STAT
    /// - Bit 0, Vertical Blank
    enable: u8,
    /// R/W. Set the bit to be 1 if interrupt
    /// is requested.
    ///
    /// - Bit 4, Joypad
    /// - Bit 3, Serial
    /// - Bit 2, Timer
    /// - Bit 1, LCD STAT
    /// - Bit 0, Vertical Blank
    flag: u8,
}

impl Interrupts {
    fn new() -> Self {
        Self { enable: 0, flag: 0 }
    }
}

/// IO devices
struct Devices {
    interrupts: Interrupts,
}

impl Devices {
    fn new() -> Self {
        Self { interrupts: Interrupts::new() }
    }
}

impl io::IO for Devices {
    fn write(&mut self, addr: u16, value: u8) {
        match addr {
            // IF
            0xFF0F => self.interrupts.flag = value,
            // IE
            0xFFFF => self.interrupts.enable = value,
            // TODO
            _ => {}
        }
    }

    fn read(&self, addr: u16) -> u8 {
        match addr {
            // IF
            0xFF0F => self.interrupts.flag,
            // IE
            0xFFFF => self.interrupts.enable,
            // TODO
            _ => 0,
        }
    }
}

struct Bus {
    cart: cartridge::Cartridge,
    wram: WorkRam,
    hram: HighRam,
    devices: Devices,
}

impl io::IO for Bus {
    fn write(&mut self, addr: u16, value: u8) {
        debug!("bus write at {:#04X}, value: {:#02X}", addr, value);

        match addr {
            0x0000..=0x7FFF => {
                // ROM data
                self.cart.write(addr, value);
            }
            0x8000..=0x9FFF => todo!("VRAM"),
            0xA000..=0xBFFF => {
                // EXT-RAM, from cartridge
                self.cart.write(addr, value);
            }
            0xC000..=0xDFFF => {
                // WRAM
                self.wram.write(addr, value);
            }
            0xE000..=0xFDFF => unreachable!("Unusable ECHO RAM [0xE000, 0xFDFF]"),
            0xFE00..=0xFE9F => todo!("OAM"),
            0xFEA0..=0xFEFF => unreachable!("Unusable memory [0xFEA0, 0xFEFF]"),
            0xFF00..=0xFF7F | 0xFFFF => {
                self.devices.write(addr, value);
            }
            0xFF80..=0xFFFE => {
                // HRAM
                self.hram.write(addr, value);
            }
        }
    }

    fn read(&self, addr: u16) -> u8 {
        let value = match addr {
            0x0000..=0x7FFF => {
                // ROM data
                self.cart.read(addr)
            }
            0x8000..=0x9FFF => todo!("VRAM"),
            0xA000..=0xBFFF => {
                // EXT-RAM, from cartridge
                self.cart.read(addr)
            }
            0xC000..=0xDFFF => {
                // WRAM
                self.wram.read(addr)
            }
            0xE000..=0xFDFF => unreachable!("Unusable ECHO RAM [0xE000, 0xFDFF]"),
            0xFE00..=0xFE9F => todo!("OAM"),
            0xFEA0..=0xFEFF => unreachable!("Unusable memory [0xFEA0, 0xFEFF]"),
            0xFF00..=0xFF7F | 0xFFFF => self.devices.read(addr),
            0xFF80..=0xFFFE => {
                // HRAM
                self.hram.read(addr)
            }
        };

        debug!("bus read at {:#04X}, value: {:#04X}", addr, value);
        value
    }
}

fn main() {
    env_logger::init();

    let mut rom_path = std::env::current_dir().unwrap();
    rom_path.push("roms");
    rom_path.push("cpu_instrs.gb");
    let cart = cartridge::Cartridge::load(&rom_path).unwrap();

    // Delegate all RWs.
    let bus = Bus { cart, wram: WorkRam::new(), hram: HighRam::new(), devices: Devices::new() };

    let mut cpu = cpu_sm83::Cpu::new(bus);

    // loop {
    //     cpu.execute();
    // }
    for _ in 1..20 {
        cpu.handle_interrupts();
        cpu.execute();

        if cpu.stopped {
            println!("Stopping...");
            // TODO
            std::process::exit(0);
        }
    }
}
