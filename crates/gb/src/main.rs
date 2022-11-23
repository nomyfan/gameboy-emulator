use log::debug;

struct WorkRam {
    /// [C000, D000)
    /// 4KiB
    ram: [u8; 0x1000],
}

impl WorkRam {
    fn new() -> Self {
        Self { ram: [0; 0x1000] }
    }
}

impl io::IO for WorkRam {
    fn write(&mut self, addr: u16, value: u8) {
        debug_assert!(addr >= 0xC000 && addr <= 0xCFFF);

        let addr = (addr as usize) - 0xC000;
        self.ram[addr] = value;
    }

    fn read(&self, addr: u16) -> u8 {
        debug_assert!(addr >= 0xC000 && addr <= 0xCFFF);

        let addr = (addr as usize) - 0xC000;
        self.ram[addr]
    }
}

struct HighRam {
    /// [FF80, FFFF)
    ram: [u8; 0x7F],
}

impl HighRam {
    fn new() -> Self {
        Self { ram: [0; 0x7F] }
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

struct Bus {
    cart: cartridge::Cartridge,
    wram: WorkRam,
    hram: HighRam,
}

impl io::IO for Bus {
    fn write(&mut self, addr: u16, value: u8) {
        debug!("bus write at 0x{:04X}, value: 0x{:02X}", addr, value);

        if addr < 0x8000 {
            // ROM data
            self.cart.write(addr, value);
        } else if addr < 0xA000 {
            // VRAM
        } else if addr < 0xC000 {
            // EXT-RAM, from cartridge
            self.cart.write(addr, value);
        } else if addr < 0xE000 {
            // WRAM
            self.wram.write(addr, value);
        } else if addr < 0xFE00 {
            // Reserved echo RAM
        } else if addr < 0xFEA0 {
            // OAM
        } else if addr < 0xFF00 {
            // Reserved
        } else if addr < 0xFF80 {
            // IO registers
        } else if addr < 0xFFFF {
            // HRAM
            self.hram.write(addr, value);
        } else {
            // addr == 0xFFFF
            // CPU IE
        }
    }

    fn read(&self, addr: u16) -> u8 {
        let value = if addr < 0x8000 {
            // ROM data
            self.cart.read(addr)
        } else if addr < 0xA000 {
            // VRAM
            // TODO
            0
        } else if addr < 0xC000 {
            // EXT-RAM, from cartridge
            self.cart.read(addr)
        } else if addr < 0xE000 {
            // WRAM
            self.wram.read(addr)
        } else if addr < 0xFE00 {
            // Reserved echo RAM
            0
        } else if addr < 0xFEA0 {
            // OAM
            // TODO
            0
        } else if addr < 0xFF00 {
            // Reserved
            0
        } else if addr < 0xFF80 {
            // IO registers
            // TODO
            0
        } else if addr < 0xFFFF {
            // HRAM
            self.hram.read(addr)
        } else {
            // addr == 0xFFFF
            // CPU IE
            // TODO
            0
        };

        debug!("bus read at 0x{:04X}, value: 0x{:04X}", addr, value);
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
    let bus = Bus { cart, wram: WorkRam::new(), hram: HighRam::new() };

    let mut cpu = cpu_sm83::Cpu::new(bus);

    // loop {
    //     cpu.execute();
    // }
    for _ in 1..20 {
        cpu.execute();
        debug!(
            "SP: 0x{:04X}, PC: 0x{:04X}, AF: 0x{:04X}, BC: 0x{:04X}, DE: 0x{:04X}, HL: 0x{:04X}",
            cpu.sp,
            cpu.pc,
            cpu.af(),
            cpu.bc(),
            cpu.de(),
            cpu.hl(),
        );
    }
}
