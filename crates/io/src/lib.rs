pub trait IO {
    fn write(&mut self, addr: u16, value: u8);
    fn read(&self, addr: u16) -> u8;
}
