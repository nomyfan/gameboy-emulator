///! OAM bug, https://gbdev.io/pandocs/OAM_Corruption_Bug.html

/// ```
/// -----------------------------------------------------------
/// 1st (X,Y) | 2nd(Index,Attrs) | 3rd(X,Y) | 4th(Index, Attrs)
/// -----------------------------------------------------------
/// ```
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Partition {
    First = 0,
    Second = 1,
    Third = 2,
    Fourth = 3,
}

impl From<u8> for Partition {
    fn from(value: u8) -> Self {
        match value & 0b11 {
            0 => Partition::First,
            1 => Partition::Second,
            2 => Partition::Third,
            3 => Partition::Fourth,
            _ => unreachable!(),
        }
    }
}

fn read_word(oam: &[u8; 160], row_index: u8, partition: Partition) -> u16 {
    let base_addr = row_index as usize * 8; // Two object each row, each object is 4 bytes.
    let offset = partition as u8 * 2; // One word is in size of 16 bits.

    let low = oam[base_addr + offset as usize] as u16;
    let high = oam[base_addr + offset as usize + 1] as u16;

    (high << 8) | low
}

fn write_word(oam: &mut [u8; 160], row_index: u8, partition: Partition, value: u16) {
    let base_addr = row_index as usize * 8; // Two object each row, each object is 4 bytes.
    let offset = partition as u8 * 2; // One word is in size of 16 bits.

    let low = value as u8;
    let high = (value >> 8) as u8;

    oam[base_addr + offset as usize] = low;
    oam[base_addr + offset as usize + 1] = high;
}

pub(crate) fn write_corruption(oam: &mut [u8; 160], current_row: u8) {
    debug_assert!(current_row >= 1);

    let a = read_word(oam, current_row, Partition::First);
    let b = read_word(oam, current_row - 1, Partition::First);
    let c = read_word(oam, current_row - 1, Partition::Third);

    let value = ((a ^ c) & (b ^ c)) ^ c;
    write_word(oam, current_row, Partition::First, value);
    for partition in 1..=3 {
        let partition = partition.into();
        write_word(oam, current_row, partition, read_word(oam, current_row - 1, partition));
    }
}

pub(crate) fn read_corruption(oam: &mut [u8; 160], current_row: u8) {
    debug_assert!(current_row >= 1);

    let a = read_word(oam, current_row, Partition::First);
    let b = read_word(oam, current_row - 1, Partition::First);
    let c = read_word(oam, current_row - 1, Partition::Third);

    let value = b | (a & c);
    write_word(oam, current_row, Partition::First, value);
    for partition in 1..=3 {
        let partition = partition.into();
        write_word(oam, current_row, partition, read_word(oam, current_row - 1, partition));
    }
}

#[inline]
pub(crate) fn write_idu_corruption(oam: &mut [u8; 160], current_row: u8) {
    write_corruption(oam, current_row);
}

pub(crate) fn read_idu_corruption(oam: &mut [u8; 160], current_row: u8) {
    // Is not one of the first four, as well as it's not the last one.
    if current_row >= 4 && current_row <= 19 {
        let a = read_word(oam, current_row - 2, Partition::First);
        let b = read_word(oam, current_row - 1, Partition::First);
        let c = read_word(oam, current_row, Partition::First);
        let d = read_word(oam, current_row - 1, Partition::Third);

        let value = (b & (a | c | d)) | (a & c & d);
        write_word(oam, current_row - 1, Partition::First, value);

        // Copy contents from preceding row to current row and the row two rows before.
        for partition in 0..=3 {
            let partition = partition.into();
            write_word(oam, current_row, partition, read_word(oam, current_row - 1, partition));
        }
        for partition in 0..=3 {
            let partition = partition.into();
            write_word(oam, current_row - 2, partition, read_word(oam, current_row - 1, partition));
        }
    }

    read_corruption(oam, current_row);
}
