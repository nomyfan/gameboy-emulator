fn main() {
    env_logger::init();

    let mut cpu = cpu_sm83::Cpu::new();
    cpu.execute();
}
