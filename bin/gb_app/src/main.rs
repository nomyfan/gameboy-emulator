use gb::GameBoy;

fn main() -> anyhow::Result<()> {
    env_logger::init();

    let rom_path = std::env::args().nth(1).unwrap();
    let rom_path = std::path::Path::new(&rom_path);

    // TODO: a callback to trigger rendering
    let gb = GameBoy::try_from_path(rom_path)?;

    gb.play()
}
