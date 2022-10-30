fn main() {
    let cart = cartridge::Cartridge::load(std::path::Path::new("./roms/01.gb")).unwrap();
    println!("1 + 1 = {}", cpu_sm83::add(1, 1));
}
