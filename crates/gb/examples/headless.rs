use gb::{Cartridge, GameBoy, Manifest};
use gb_shared::CPU_FREQ;

// cargo run --example headless /path/to/rom 20
// cargo flamegraph --example headless -- /path/to/rom 20
fn main() {
    let args = std::env::args().collect::<Vec<String>>();
    let rom_path = args.get(1).unwrap();
    let mut cpu_seconds = args.get(2).unwrap().parse::<u32>().unwrap();

    let rom = std::fs::read(std::path::Path::new(rom_path)).unwrap();
    let cart = Cartridge::try_from(rom).unwrap();
    let mut gb = GameBoy::new(Manifest { cart, sample_rate: Some(44_1000) });

    while cpu_seconds > 0 {
        gb.continue_clocks((cpu_seconds.min(512)) * CPU_FREQ);
        cpu_seconds = cpu_seconds.saturating_sub(512);
    }
}
