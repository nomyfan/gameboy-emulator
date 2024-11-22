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
    };
}

#[macro_export]
macro_rules! set_bits {
    ($value:expr, $($bit:expr),+) => {
        {
            ($value) $(| (1 << ($bit)))+
        }
    };
}

#[macro_export]
macro_rules! unset_bits {
    ($value:expr, $($bit:expr),+) => {
        {
            ($value) & !(0 $(| (1 << ($bit)))+)
        }
    };
}

#[cfg(test)]
mod tests {
    use crate::{pick_bits, set_bits, unset_bits};

    #[test]
    fn pick_bits() {
        assert_eq!(0b1000_0100, pick_bits!(0b1111_1111u8, 2, 7));
    }

    #[test]
    fn set_bits() {
        assert_eq!(0b1000_0100, set_bits!(0b0000_0000u8, 2, 7));
    }

    #[test]
    fn unset_bits() {
        assert_eq!(0b0111_1011, unset_bits!(0b1111_1111u8, 2, 7));
    }
}
