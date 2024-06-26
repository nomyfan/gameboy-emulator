pub mod bitwise;
pub mod boxed;
pub mod command;

pub fn boxed_array<T: Copy, const SIZE: usize>(val: T) -> Box<[T; SIZE]> {
    let boxed_slice = vec![val; SIZE].into_boxed_slice();
    let ptr = Box::into_raw(boxed_slice) as *mut [T; SIZE];

    unsafe { Box::from_raw(ptr) }
}

pub fn boxed_array_fn<T, F: Fn(usize) -> T, const SIZE: usize>(init_fn: F) -> Box<[T; SIZE]> {
    let mut vector = Vec::with_capacity(SIZE);
    for x in 0..SIZE {
        vector.push(init_fn(x));
    }
    let boxed_slice = vector.into_boxed_slice();
    let ptr = Box::into_raw(boxed_slice) as *mut [T; SIZE];

    unsafe { Box::from_raw(ptr) }
}

pub fn boxed_array_try_from_vec<T, const SIZE: usize>(
    vec: Vec<T>,
) -> Result<Box<[T; SIZE]>, Vec<T>> {
    if vec.len() == SIZE {
        let boxed_slice = vec.into_boxed_slice();
        let ptr = Box::into_raw(boxed_slice) as *mut [T; SIZE];

        Ok(unsafe { Box::from_raw(ptr) })
    } else {
        Err(vec)
    }
}

pub trait Memory {
    fn write(&mut self, addr: u16, value: u8);
    fn read(&self, addr: u16) -> u8;
}

pub trait Bus: Memory {
    fn step(&mut self, clocks: u8);
    fn vdma_active(&self) -> bool;
    fn step_vdma(&mut self);
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InterruptType {
    VBlank = 0b1,
    LCDStat = 0b10,
    Timer = 0b100,
    Serial = 0b1000,
    Joypad = 0b10000,
}

pub trait InterruptRequest {
    fn request(&mut self, interrupt_type: InterruptType);
    fn request_vblank(&mut self) {
        self.request(InterruptType::VBlank);
    }
    fn request_lcd_stat(&mut self) {
        self.request(InterruptType::LCDStat);
    }
    fn request_timer(&mut self) {
        self.request(InterruptType::Timer);
    }
    fn request_serial(&mut self) {
        self.request(InterruptType::Serial);
    }
    fn request_joypad(&mut self) {
        self.request(InterruptType::Joypad);
    }
}

#[derive(Debug, Default)]
pub struct Interrupt(pub u8);

impl InterruptRequest for Interrupt {
    fn request(&mut self, interrupt_type: InterruptType) {
        self.0 |= interrupt_type as u8;
    }
}

impl Interrupt {
    pub fn take(&mut self) -> u8 {
        let value = self.0;
        self.0 = 0;
        value
    }
}

pub const fn kib(k: usize) -> usize {
    k * 1024
}

pub const fn mib(m: usize) -> usize {
    kib(m) * 1024
}

pub const CPU_FREQ: u32 = 4_194_304;

pub trait Snapshot {
    type Snapshot;
    fn take_snapshot(&self) -> Self::Snapshot;
    fn restore_snapshot(&mut self, snapshot: Self::Snapshot);
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MachineModel {
    DMG,
    CGB,
}
