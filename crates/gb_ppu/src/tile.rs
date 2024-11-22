/// Return color ID in range of 0..4.
pub(crate) fn get_color_id(data: &[u8; 16], x: u8, y: u8, x_flip: bool, y_flip: bool) -> u8 {
    assert!(x < 8 && y < 8);
    let nth = (if y_flip { 7 - y } else { y } << 1) as usize;
    let offset = if x_flip { x } else { 7 - x } as usize;

    let low = (data[nth] >> offset) & 1;
    let high = (data[nth + 1] >> offset) & 1;

    (high << 1) | low
}

#[cfg(test)]
mod tests {
    use super::get_color_id;

    #[test]
    fn pick_color() {
        let data = &[
            0b00111100, 0b01111110, 0b01000010, 0b01000010, 0b01000010, 0b01000010, 0b01000010,
            0b01000010, 0b01111110, 0b01011110, 0b01111110, 0b00001010, 0b01111100, 0b01010110,
            0b00111000, 0b01111100,
        ];

        assert_eq!(get_color_id(data, 1, 0, false, false), 0b10);
        assert_eq!(get_color_id(data, 3, 4, false, false), 0b11);
        assert_eq!(get_color_id(data, 2, 1, false, false), 0b00);
        assert_eq!(get_color_id(data, 3, 5, false, false), 0b01);
    }

    #[test]
    fn flip_x() {
        let data = &[
            0b00110011, 0b01010101, 0b00000000, 0b00000000, 0b00000000, 0b00000000, 0b00000000,
            0b00000000, 0b00000000, 0b00000000, 0b00000000, 0b00000000, 0b00000000, 0b00000000,
            0b00000000, 0b00000000,
        ];

        assert_eq!(get_color_id(data, 0, 0, true, false), 0b11);
    }

    #[test]
    fn flip_y() {
        let data = &[
            0b00110011, 0b01010101, 0b00000000, 0b00000000, 0b00000000, 0b00000000, 0b00000000,
            0b00000000, 0b00000000, 0b00000000, 0b00000000, 0b00000000, 0b00000000, 0b00000000,
            0b00000000, 0b00000000,
        ];

        assert_eq!(get_color_id(data, 2, 7, false, true), 0b01);
    }
}
