fn main() {
    env_logger::init();

    let mut cpu = cpu_sm83::Cpu::new();
    let instr = cpu_sm83::instruction::LD {};
    cpu.exec_instr(instr);
}
