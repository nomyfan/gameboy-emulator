use log::debug;

struct Bus {
    cart: cartridge::Cartridge,
}

impl io::IO for Bus {
    fn write(&mut self, addr: u16, value: u8) {
        debug!("bus write address: 0x{:04X}, value: 0x{:02X}", addr, value);

        if addr < 0x8000 {
            // ROM data
        } else if addr < 0xA000 {
            // Char/Map data
        } else if addr < 0xC000 {
            // EXT-RAM
        } else if addr < 0xE000 {
            // WRAM
        } else if addr < 0xFE00 {
            // Reserved echo RAM
        } else if addr < 0xFEA0 {
            // OAM
        } else if addr < 0xFF00 {
            // Reserved
        } else if addr < 0xFF80 {
            // IO registers
        } else if addr == 0xFFFF {
            // CPU set IE
        } else {
            // HRAM
        }
    }

    fn read(&self, addr: u16) -> u8 {
        debug!("bus read 0x{:04X}", addr);

        if addr < 0x8000 {
            // ROM data
        } else if addr < 0xA000 {
            // Char/Map data
        } else if addr < 0xC000 {
            // EXT-RAM
        } else if addr < 0xE000 {
            // WRAM
        } else if addr < 0xFE00 {
            // Reserved echo RAM
        } else if addr < 0xFEA0 {
            // OAM
        } else if addr < 0xFF00 {
            // Reserved
        } else if addr < 0xFF80 {
            // IO registers
        } else if addr == 0xFFFF {
            // CPU set IE
        } else {
            // HRAM
        }

        0
    }
}

fn main() {
    env_logger::init();

    let mut rom_path = std::env::current_dir().unwrap();
    rom_path.push("roms");
    rom_path.push("06-ld r,r.gb");
    let cart = cartridge::Cartridge::load(&rom_path).unwrap();

    // Delegate all RWs.
    let bus = Bus { cart };

    let mut cpu = cpu_sm83::Cpu::new(bus);
    cpu.execute();
}
