use gb_cartridge::Cartridge;

const ROMS: [&str; 2] = [
    //
    "../../roms/gb-test-roms/cpu_instrs/cpu_instrs.gb",
    "../../roms/dmg-acid2.gb",
];

#[test]
fn test_load() {
    ROMS.iter().for_each(|path| {
        let rom = std::fs::read(std::path::Path::new(path)).unwrap();
        Cartridge::try_from(rom).unwrap();
    })
}
