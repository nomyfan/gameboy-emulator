use gb_cartridge::Cartridge;

const ROMS: [&str; 14] = [
    "../../roms/01-special.gb",
    "../../roms/02-interrupts.gb",
    "../../roms/03-op sp,hl.gb",
    "../../roms/04-op r,imm.gb",
    "../../roms/05-op rp.gb",
    "../../roms/06-ld r,r.gb",
    "../../roms/07-jr,jp,call,ret,rst.gb",
    "../../roms/08-misc instrs.gb",
    "../../roms/09-op r,r.gb",
    "../../roms/10-bit ops.gb",
    "../../roms/11-op a,(hl).gb",
    "../../roms/cpu_instrs.gb",
    "../../roms/dmg-acid2.gb",
    "../../roms/mem_timing.gb",
];

#[test]
fn test_load() {
    ROMS.iter().for_each(|rom| {
        Cartridge::load(std::path::Path::new(rom)).unwrap();
    })
}
