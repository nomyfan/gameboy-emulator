use gb_cartridge::Cartridge;

const ROMS: [&str; 2] = [
    //
    "../../roms/gb-test-roms/cpu_instrs/cpu_instrs.gb",
    "../../roms/dmg-acid2.gb",
];

#[test]
fn test_load() {
    ROMS.iter().for_each(|rom| {
        Cartridge::load(std::path::Path::new(rom)).unwrap();
    })
}
