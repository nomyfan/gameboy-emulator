use super::{real_ram_size, Mbc, RamBank};
use crate::CartridgeHeader;
use gb_shared::{box_array, kib, ByteView, Snapshot};
use serde::{Deserialize, Serialize};
use web_time::SystemTime;

pub(crate) struct Mbc3 {
    ram_banks: Vec<Box<RamBank>>,
    rtc: RealTimeClock,
    with_battery: bool,
    /// 0x0000..=0x1FFF
    ///
    /// Writing 0x0A will enable RAM banking and RTC registers.
    ram_rtc_enabled: bool,
    /// 0x2000..=0x3FFFF
    ///
    /// ROM bank number. 0x00 will select bank 0x01.
    rom_bank_num: u8,
    /// 0x4000..=0x5FFF
    ///
    /// RAM bank number or RTC register select.
    /// When it's value is 0x00..=0x03, it's RAM bank number.
    /// When it's value is 0x08..=0x0C, it's RTC register select.
    /// It controls whose data is mapped in the area 0xA000..=0xBFFF.
    reg_ram_bank_rtc: u8,
    /// Writing 0x00 and then 0x01, the current time becomes latched
    /// into the RTC registers.
    reg_latch_clock: u8,
}

impl Mbc3 {
    pub(crate) fn new(header: &CartridgeHeader) -> Self {
        let cart_type = header.cart_type;
        let with_battery = cart_type == 0x0F || cart_type == 0x10 || cart_type == 0x13;

        let ram_banks_len = real_ram_size(header.ram_size) / kib(8);
        let mut ram_banks: Vec<Box<RamBank>> = Vec::with_capacity(ram_banks_len);
        for _ in 0..ram_banks_len {
            ram_banks.push(box_array![u8; 0x2000]);
        }

        Self {
            ram_banks,
            rtc: RealTimeClock::new(),
            with_battery,
            ram_rtc_enabled: false,
            rom_bank_num: 0,
            reg_ram_bank_rtc: 0,
            reg_latch_clock: 0xFF,
        }
    }
}

impl Mbc for Mbc3 {
    fn write(&mut self, addr: u16, value: u8) {
        match addr {
            0x0000..=0x1FFF => {
                self.ram_rtc_enabled = (value & 0x0F) == 0x0A;
            }
            0x2000..=0x3FFF => {
                self.rom_bank_num = value & 0x7F;
            }
            0x4000..=0x5FFF => {
                self.reg_ram_bank_rtc = value & 0xF;
            }
            0x6000..=0x7FFF => {
                if self.reg_latch_clock == 0x00 && value == 0x01 {
                    self.rtc.latch();
                }
                self.reg_latch_clock = value;
            }
            0xA000..=0xBFFF => {
                if !self.ram_rtc_enabled {
                    return;
                }

                match self.reg_ram_bank_rtc {
                    0x00..=0x03 => {
                        let ram_bank_num = self.reg_ram_bank_rtc as usize;
                        self.ram_banks[ram_bank_num][addr as usize - 0xA000] = value;
                    }
                    0x08 => self.rtc.s = value,
                    0x09 => self.rtc.m = value,
                    0x0A => self.rtc.h = value,
                    0x0B => self.rtc.dl = value,
                    0x0C => self.rtc.dh = value,
                    _ => unreachable!(),
                }
            }
            _ => unreachable!("Invalid addr {:#04X} for MBC3", addr),
        }
    }

    fn read(&self, addr: u16, rom: &[u8]) -> u8 {
        match addr {
            0x0000..=0x3FFF => rom[addr as usize],
            0x4000..=0x7FFF => {
                let mut rom_bank_num = self.rom_bank_num as usize;
                if rom_bank_num == 0 {
                    rom_bank_num = 1;
                }
                rom[rom_bank_num * kib(16) + (addr as usize - 0x4000)]
            }
            0xA000..=0xBFFF => {
                if !self.ram_rtc_enabled {
                    return 0xFF;
                }

                match self.reg_ram_bank_rtc {
                    0x00..=0x03 => {
                        let ram_bank_num = self.reg_ram_bank_rtc as usize;
                        self.ram_banks[ram_bank_num][addr as usize - 0xA000]
                    }
                    0x08 => self.rtc.s,
                    0x09 => self.rtc.m,
                    0x0A => self.rtc.h,
                    0x0B => self.rtc.dl,
                    0x0C => self.rtc.dh,
                    _ => unreachable!(),
                }
            }
            _ => unreachable!("Invalid addr {:#04X} for MBC3", addr),
        }
    }

    fn suspend(&self) -> Option<Vec<u8>> {
        if self.with_battery {
            let mut data = vec![];
            let rtc_data = self.rtc.epoch;
            data.extend_from_slice(&rtc_data.to_be_bytes());
            for bank in &self.ram_banks {
                data.extend_from_slice(bank.as_ref());
            }

            return Some(data);
        }

        None
    }

    fn resume(&mut self, data: &[u8]) -> anyhow::Result<()> {
        if self.with_battery {
            if data.len() != (self.ram_banks.len() * kib(8) + 8) {
                anyhow::bail!("Invalid data length for MBC3")
            }
            self.rtc.epoch = u64::from_be_bytes(data[..8].try_into().unwrap());
            data[8..].chunks(kib(8)).zip(&mut self.ram_banks).for_each(|(src, dst)| {
                dst.copy_from_slice(src);
            });
        }
        Ok(())
    }
}

#[derive(Serialize, Deserialize)]
struct Mbc3Snapshot {
    ram_banks: Vec<u8>,
    rtc: RealTimeClock,
    with_battery: bool,
    ram_rtc_enabled: bool,
    rom_bank_num: u8,
    reg_ram_bank_rtc: u8,
    reg_latch_clock: u8,
}

impl Snapshot for Mbc3 {
    type Snapshot = Vec<u8>;

    fn take_snapshot(&self) -> Self::Snapshot {
        let mut ram_banks_snapshot = vec![];
        for bank in &self.ram_banks {
            ram_banks_snapshot.extend_from_slice(bank.as_ref());
        }

        bincode::serialize(&Mbc3Snapshot {
            ram_banks: ram_banks_snapshot,
            rtc: self.rtc,
            with_battery: self.with_battery,
            ram_rtc_enabled: self.ram_rtc_enabled,
            rom_bank_num: self.rom_bank_num,
            reg_ram_bank_rtc: self.reg_ram_bank_rtc,
            reg_latch_clock: self.reg_latch_clock,
        })
        .unwrap()
    }

    fn restore_snapshot(&mut self, snapshot: Self::Snapshot) {
        let Mbc3Snapshot {
            ram_banks,
            rtc,
            with_battery,
            ram_rtc_enabled,
            rom_bank_num,
            reg_ram_bank_rtc,
            reg_latch_clock,
        } = bincode::deserialize(&snapshot).unwrap();
        assert_eq!(ram_banks.len(), self.ram_banks.len() * kib(8));

        self.rtc = rtc;
        self.with_battery = with_battery;
        self.ram_rtc_enabled = ram_rtc_enabled;
        self.rom_bank_num = rom_bank_num;
        self.reg_ram_bank_rtc = reg_ram_bank_rtc;
        self.reg_latch_clock = reg_latch_clock;

        ram_banks.chunks(kib(8)).zip(&mut self.ram_banks).for_each(|(src, dst)| {
            dst.copy_from_slice(src);
        });
    }
}

#[derive(Clone, Copy, Serialize, Deserialize)]
pub(crate) struct RealTimeClock {
    /// Seconds
    s: u8,
    /// Minutes
    m: u8,
    /// Hours
    h: u8,
    /// Lower 8 bits of Day Counter
    dl: u8,
    /// Bit 0: Most significant bit of Day Counter (Bit 8)
    /// Bit 6: Halt (0=Active, 1=Stop Timer)
    /// Bit 7: Day Counter Carry Bit (1=Counter Overflow)
    dh: u8,
    /// Emulator internal epoch, not the real world's.
    /// By saving this value, we can keep the RTC running
    /// even if the emulator is closed.
    epoch: u64,
}

impl RealTimeClock {
    pub(crate) fn new() -> Self {
        let epoch = SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).unwrap().as_secs();

        Self { s: 0, m: 0, h: 0, dl: 0, dh: 0, epoch }
    }

    pub(crate) fn latch(&mut self) {
        let duration = SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).unwrap().as_secs()
            - self.epoch;

        self.s = (duration % 60) as u8;
        self.m = ((duration / 60) % 60) as u8;
        self.h = ((duration / 3600) % 24) as u8;
        let days = (duration / 3600 / 24) as u16;
        self.dl = days.lsb();
        self.dh |= days.msb() & 1;
        if days > 0x01FF {
            self.dh |= 0x80;
        }
    }
}
