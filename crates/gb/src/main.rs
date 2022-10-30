fn main() {
    env_logger::init();
    println!("1 + 1 = {}", cpu_sm83::add(1, 1));
}
