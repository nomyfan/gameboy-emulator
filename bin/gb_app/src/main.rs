use gb::GameBoy;

fn main() -> anyhow::Result<()> {
    env_logger::init();

    // TODO: a callback to trigger rendering
    let gb = GameBoy::try_from_path(
        std::env::current_dir().unwrap().join("roms").join("cpu_instrs.gb"),
    )?;

    gb.play()
}
