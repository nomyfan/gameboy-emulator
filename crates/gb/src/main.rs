fn main() {
    env_logger::init();

    let mut rom_path = std::env::current_dir().unwrap();
    rom_path.push("roms");
    rom_path.push("06-ld r,r.gb");
    let cart = cartridge::Cartridge::load(&rom_path).unwrap();
    let mut cpu = cpu_sm83::Cpu::new();
    cpu.execute();
}
