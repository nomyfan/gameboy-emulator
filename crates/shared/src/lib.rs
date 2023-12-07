pub fn boxed_array<T: Copy, const SIZE: usize>(val: T) -> Box<[T; SIZE]> {
    let boxed_slice = vec![val; SIZE].into_boxed_slice();
    let ptr = Box::into_raw(boxed_slice) as *mut [T; SIZE];

    unsafe { Box::from_raw(ptr) }
}

pub fn boxed_array_fn<T: Copy, F: Fn(usize) -> T, const SIZE: usize>(init_fn: F) -> Box<[T; SIZE]> {
    let mut vector = Vec::with_capacity(SIZE);
    for x in 0..SIZE {
        vector.push(init_fn(x));
    }
    let boxed_slice = vector.into_boxed_slice();
    let ptr = Box::into_raw(boxed_slice) as *mut [T; SIZE];

    unsafe { Box::from_raw(ptr) }
}

pub trait Memory {
    fn write(&mut self, addr: u16, value: u8);
    fn read(&self, addr: u16) -> u8;
}

#[macro_export]
macro_rules! is_bit_set {
    ($value:expr, $bit:expr) => {
        (($value) & (1 << ($bit))) != 0
    };
}

#[macro_export]
macro_rules! pick_bits {
    ($value:expr, $($offset:expr),*) => {
        {
            ($value) & (0 $(| (1 << ($offset)))*)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::pick_bits;

    #[test]
    fn pick_bits() {
        assert_eq!(0b1000_0100, pick_bits!(0b1111_1111u8, 2, 7));
    }
}
