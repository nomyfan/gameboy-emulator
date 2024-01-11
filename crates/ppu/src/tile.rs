use crate::object::ObjectAttrs;

/// Return color ID in range of 0..4.
pub(crate) fn get_color_id(data: &[u16; 8], x: u8, y: u8) -> u8 {
    assert!(x < 8 && y < 8);

    let colors = data[y as usize];
    let offset = (7 - x) as usize * 2;
    let color_id = (colors >> offset) & 0b11;

    color_id as u8
}

pub(crate) fn mix_colors(low: &[u8; 8], high: &[u8; 8]) -> [u16; 8] {
    let mut colors: [u16; 8] = Default::default();

    let mut mix = |data: &[u8; 8], offset: usize| {
        for i in (0..data.len()).step_by(2) {
            let lsbs = data[i];
            let msbs = data[i + 1];

            let mut color = 0u16;
            for bit in 0..8 {
                let lsb = ((lsbs >> bit) & 1) as u16;
                let msb = ((msbs >> bit) & 1) as u16;

                let lsb = lsb << (bit * 2);
                let msb = msb << (bit * 2 + 1);

                color |= msb | lsb;
            }
            colors[i / 2 + offset] = color;
        }
    };

    mix(low, 0);
    mix(high, 4);

    colors
}

pub(crate) fn mix_colors_16(data: &[u8; 16]) -> [u16; 8] {
    mix_colors(data[0..8].try_into().unwrap(), data[8..16].try_into().unwrap())
}

pub(crate) fn apply_object_attrs(data: &mut [u16; 8], attrs: &ObjectAttrs) {
    if attrs.y_flip() {
        for i in 0..4 {
            data.swap(i, 7 - i);
        }
    }
    if attrs.x_flip() {
        for value in data.iter_mut() {
            let mut new_value = 0;
            for offset in (0..16).step_by(2) {
                new_value |= (((*value) >> offset) & 0b11) << (14 - offset);
            }
            *value = new_value;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{apply_object_attrs, get_color_id, mix_colors};
    use crate::object::ObjectAttrs;

    #[test]
    fn test_build_colors() {
        let low: [u8; 8] = [
            0b00_11_11_00,
            0b01_11_11_10,
            0b01_00_00_10,
            0b01_00_00_10,
            0b01_00_00_10,
            0b01_00_00_10,
            0b01_00_00_10,
            0b01_00_00_10,
        ];
        let high: [u8; 8] = [
            0b01_11_11_10,
            0b01_01_11_10,
            0b01_11_11_10,
            0b00_00_10_10,
            0b01_11_11_00,
            0b01_01_01_10,
            0b00_11_10_00,
            0b01_11_11_00,
        ];

        let colors = mix_colors(&low, &high);

        let expected: [u16; 8] = [
            0b00_10_11_11_11_11_10_00,
            0b00_11_00_00_00_00_11_00,
            0b00_11_00_00_00_00_11_00,
            0b00_11_00_00_00_00_11_00,
            0b00_11_01_11_11_11_11_00,
            0b00_01_01_01_11_01_11_00,
            0b00_11_01_11_01_11_10_00,
            0b00_10_11_11_11_10_00_00,
        ];

        assert_eq!(expected, colors);
    }

    #[test]
    fn pick_color() {
        let colors = [
            0b00_10_11_11_11_11_10_00,
            0b00_11_00_00_00_00_11_00,
            0b00_11_00_00_00_00_11_00,
            0b00_11_00_00_00_00_11_00,
            0b00_11_01_11_11_11_11_00,
            0b00_01_01_01_11_01_11_00,
            0b00_11_01_11_01_11_10_00,
            0b00_10_11_11_11_10_00_00,
        ];

        assert_eq!(get_color_id(&colors, 1, 0), 0b10);
        assert_eq!(get_color_id(&colors, 3, 4), 0b11);
        assert_eq!(get_color_id(&colors, 2, 1), 0b00);
        assert_eq!(get_color_id(&colors, 3, 5), 0b01);
    }

    #[test]
    fn flip_x() {
        let mut data = [
            0b00_10_01_11_00_10_01_11,
            0b00_00_00_00_00_00_00_00,
            0b00_00_00_00_00_00_00_00,
            0b00_00_00_00_00_00_00_00,
            0b00_00_00_00_00_00_00_00,
            0b00_00_00_00_00_00_00_00,
            0b00_00_00_00_00_00_00_00,
            0b00_00_00_00_00_00_00_00,
        ];
        apply_object_attrs(&mut data, &ObjectAttrs(0b0010_0000));

        let expected = [
            0b11_01_10_00_11_01_10_00,
            0b00_00_00_00_00_00_00_00,
            0b00_00_00_00_00_00_00_00,
            0b00_00_00_00_00_00_00_00,
            0b00_00_00_00_00_00_00_00,
            0b00_00_00_00_00_00_00_00,
            0b00_00_00_00_00_00_00_00,
            0b00_00_00_00_00_00_00_00,
        ];

        assert_eq!(expected, data);
    }

    #[test]
    fn flip_y() {
        let mut data = [
            0b00_10_01_11_00_10_01_11,
            0b00_00_00_00_00_00_00_00,
            0b00_00_00_00_00_00_00_00,
            0b00_00_00_00_00_00_00_00,
            0b00_00_00_00_00_00_00_00,
            0b00_00_00_00_00_00_00_00,
            0b00_00_00_00_00_00_00_00,
            0b00_00_00_00_00_00_00_00,
        ];
        apply_object_attrs(&mut data, &ObjectAttrs(0b0100_0000));

        let expected = [
            0b00_00_00_00_00_00_00_00,
            0b00_00_00_00_00_00_00_00,
            0b00_00_00_00_00_00_00_00,
            0b00_00_00_00_00_00_00_00,
            0b00_00_00_00_00_00_00_00,
            0b00_00_00_00_00_00_00_00,
            0b00_00_00_00_00_00_00_00,
            0b00_10_01_11_00_10_01_11,
        ];

        assert_eq!(expected, data);
    }
}
