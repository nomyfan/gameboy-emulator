const PRE_SHIFT: u8 = 32;
const TIME_BITS: u8 = PRE_SHIFT + 20;
const TIME_UNIT: u64 = 1 << TIME_BITS;
const BASS_SHIFT: u8 = 9;
const END_FRAME_EXTRA: u8 = 2;
const HALF_WIDTH: u8 = 8;
const BUF_EXTRA: u8 = HALF_WIDTH * 2 + END_FRAME_EXTRA;
const PHASE_BITS: u8 = 5;
const PHASE_COUNT: u16 = 1 << PHASE_BITS;
const DELTA_BITS: u8 = 15;
const DELTA_UNIT: u16 = 1 << DELTA_BITS;
const FRAC_BITS: u8 = TIME_BITS - PRE_SHIFT;
const MAX_SAMPLE: i16 = i16::MAX;
const MIN_SAMPLE: i16 = i16::MIN;

/// Maximum clock_rate/sample_rate ratio. For a given sample_rate,
/// clock_rate must not be greater than sample_rate*BLIP_MAX_RATIO.
pub const BLIP_MAX_RATIO: u32 = 1 << 20;
/// Maximum number of samples that can be generated from one time frame.
pub const BLIP_MAX_FRAME: u32 = 4000;

const BL_STEP: [[i16; HALF_WIDTH as usize]; PHASE_COUNT as usize + 1] = [
    [43, -115, 350, -488, 1136, -914, 5861, 21022],
    [44, -118, 348, -473, 1076, -799, 5274, 21001],
    [45, -121, 344, -454, 1011, -677, 4706, 20936],
    [46, -122, 336, -431, 942, -549, 4156, 20829],
    [47, -123, 327, -404, 868, -418, 3629, 20679],
    [47, -122, 316, -375, 792, -285, 3124, 20488],
    [47, -120, 303, -344, 714, -151, 2644, 20256],
    [46, -117, 289, -310, 634, -17, 2188, 19985],
    [46, -114, 273, -275, 553, 117, 1758, 19675],
    [44, -108, 255, -237, 471, 247, 1356, 19327],
    [43, -103, 237, -199, 390, 373, 981, 18944],
    [42, -98, 218, -160, 310, 495, 633, 18527],
    [40, -91, 198, -121, 231, 611, 314, 18078],
    [38, -84, 178, -81, 153, 722, 22, 17599],
    [36, -76, 157, -43, 80, 824, -241, 17092],
    [34, -68, 135, -3, 8, 919, -476, 16558],
    [32, -61, 115, 34, -60, 1006, -683, 16001],
    [29, -52, 94, 70, -123, 1083, -862, 15422],
    [27, -44, 73, 106, -184, 1152, -1015, 14824],
    [25, -36, 53, 139, -239, 1211, -1142, 14210],
    [22, -27, 34, 170, -290, 1261, -1244, 13582],
    [20, -20, 16, 199, -335, 1301, -1322, 12942],
    [18, -12, -3, 226, -375, 1331, -1376, 12293],
    [15, -4, -19, 250, -410, 1351, -1408, 11638],
    [13, 3, -35, 272, -439, 1361, -1419, 10979],
    [11, 9, -49, 292, -464, 1362, -1410, 10319],
    [9, 16, -63, 309, -483, 1354, -1383, 9660],
    [7, 22, -75, 322, -496, 1337, -1339, 9005],
    [6, 26, -85, 333, -504, 1312, -1280, 8355],
    [4, 31, -94, 341, -507, 1278, -1205, 7713],
    [3, 35, -102, 347, -506, 1238, -1119, 7082],
    [1, 40, -110, 350, -499, 1190, -1021, 6464],
    [0, 43, -115, 350, -488, 1136, -914, 5861],
];

pub struct Blip {
    factor: u64,
    offset: u64,
    avail: u32,
    size: u32,
    integrator: i32,
    buffer: Vec<i32>,
}

impl Blip {
    /// Creates new buffer that can hold at most sample_count samples. Sets rates
    /// so that there are blip_max_ratio clocks per sample. Returns pointer to new
    /// buffer, or NULL if insufficient memory.
    pub fn new(sample_count: u32) -> Self {
        let buffer = vec![0; (sample_count + BUF_EXTRA as u32) as usize];
        const FACTOR: u64 = TIME_UNIT / BLIP_MAX_RATIO as u64;
        let offset = FACTOR / 2;
        Self { factor: FACTOR, size: sample_count, offset, avail: 0, integrator: 0, buffer }
    }

    /// Sets approximate input clock rate and output sample rate. For every
    /// clock_rate input clocks, approximately sample_rate samples are generated.
    pub fn set_rates(&mut self, clock_rate: f64, sample_rate: f64) {
        let factor = TIME_UNIT as f64 * sample_rate / clock_rate;
        self.factor = factor.ceil() as u64;
    }

    /// Clears entire buffer. Afterwards, samples_avail() == 0.
    pub fn clear(&mut self) {
        self.offset = self.factor / 2;
        self.avail = 0;
        self.integrator = 0;
        self.buffer.fill(0);
    }

    /// Length of time frame, in clocks, needed to make sample_count additional
    /// samples available.
    pub fn clocks_needed(&mut self, samples: u32) -> u32 {
        assert!(self.avail + samples <= self.size);

        let needed = samples as u64 * TIME_UNIT;
        if needed < self.offset {
            0
        } else {
            ((needed - self.offset + self.factor - 1) / self.factor) as u32
        }
    }

    /// Number of buffered samples available for reading.
    pub fn samples_avail(&self) -> u32 {
        self.avail
    }

    /// Makes input clocks before clock_duration available for reading as output
    /// samples. Also begins new time frame at clock_duration, so that clock time 0 in
    /// the new time frame specifies the same clock as clock_duration in the old time
    ///. frame specified. Deltas can have been added slightly past clock_duration (up to
    /// however many clocks there are in two output samples).
    pub fn end_frame(&mut self, clock_duration: u32) {
        let off = (clock_duration as u64).wrapping_mul(self.factor).wrapping_add(self.offset);
        self.avail = (self.avail).wrapping_add((off >> TIME_BITS) as u32);
        self.offset = off & (TIME_UNIT - 1);

        assert!(self.avail <= self.size);
    }

    fn remove_samples(&mut self, count: u32) {
        assert!(self.avail >= count);
        let remain = self.avail + (BUF_EXTRA as u32) - count;
        self.avail -= count;

        self.buffer.copy_within(count as usize..(count + remain) as usize, 0);
        self.buffer[remain as usize..(remain + count) as usize].fill(0);
    }

    /// Reads and removes at most 'count' samples and writes them to 'out'. If
    /// 'stereo' is true, writes output to every other element of 'out', allowing easy
    /// interleaving of two buffers into a stereo sample stream. Outputs 16-bit signed
    /// samples. Returns number of samples actually read.
    pub fn read_samples(&mut self, out: &mut [i16], count: u32, stereo: bool) -> u32 {
        let count = count.min(self.avail);
        assert!(count as usize <= out.len());

        if count > 0 {
            let chunk_size = if stereo { 2 } else { 1 };
            let mut sum = self.integrator;

            out.chunks_mut(chunk_size).zip(&self.buffer).for_each(|(o, v)| {
                // Eliminate fraction
                let s = (sum >> DELTA_BITS).clamp(MIN_SAMPLE as i32, MAX_SAMPLE as i32);
                o[0] = s as i16;

                sum = sum.wrapping_add(*v);
                // High-pass filter
                sum = sum.wrapping_sub(s << (DELTA_BITS - BASS_SHIFT));
            });

            self.integrator = sum;
            self.remove_samples(count);
        }

        count
    }

    /// Adds positive/negative delta into buffer at specified clock time.
    pub fn add_delta(&mut self, clock_time: u32, delta: i32) {
        let fixed = ((clock_time as u64).wrapping_mul(self.factor).wrapping_add(self.offset)
            >> PRE_SHIFT) as u32;
        let out = self.avail + (fixed >> FRAC_BITS);

        const PHASE_SHIFT: u8 = FRAC_BITS - PHASE_BITS;
        let phase = fixed >> PHASE_SHIFT & (PHASE_COUNT - 1) as u32;
        let in_0 = BL_STEP[phase as usize];
        let in_1 = BL_STEP[(phase + 1) as usize];
        let rev_0 = BL_STEP[(PHASE_COUNT as u32 - phase) as usize];
        let rev_1 = BL_STEP[(PHASE_COUNT as u32 - phase - 1) as usize];

        let interpolate = (fixed >> (PHASE_SHIFT - DELTA_BITS) & (DELTA_UNIT - 1) as u32) as i32;
        let delta2 = (delta * interpolate) >> DELTA_BITS;
        let delta = delta - delta2;

        assert!(out <= (self.size + END_FRAME_EXTRA as u32));

        let out = out as usize;
        self.buffer[out..(out + 8)].iter_mut().enumerate().for_each(|(i, v)| {
            *v = v.wrapping_add(in_0[i] as i32 * delta + in_1[i] as i32 * delta2);
        });
        self.buffer[(out + 8)..(out + 16)].iter_mut().enumerate().for_each(|(i, v)| {
            *v = v.wrapping_add(rev_0[7 - i] as i32 * delta + rev_1[7 - i] as i32 * delta2);
        });
    }

    /// Same as `add_delta`, but uses faster, lower-quality synthesis.
    pub fn add_delta_fast(&mut self, clock_time: u32, delta: i32) {
        let fixed = ((clock_time as u64).wrapping_mul(self.factor).wrapping_add(self.offset)
            >> PRE_SHIFT) as u32;
        let out = self.avail + (fixed >> FRAC_BITS);

        let interpolate = (fixed >> (FRAC_BITS - DELTA_BITS) & (DELTA_UNIT - 1) as u32) as i32;
        let delta2 = delta * interpolate;

        assert!(out <= (self.size + END_FRAME_EXTRA as u32));
        let out = out as usize;
        self.buffer[out + 7] = self.buffer[out + 7]
            .wrapping_add(delta.wrapping_mul(DELTA_UNIT as i32).wrapping_sub(delta2));
        self.buffer[out + 8] = self.buffer[out + 8].wrapping_add(delta2);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn assumptions() {
        assert!(BLIP_MAX_RATIO as u64 <= TIME_UNIT);
        assert!(BLIP_MAX_FRAME as u64 <= (u64::MAX >> TIME_BITS));
    }
}
