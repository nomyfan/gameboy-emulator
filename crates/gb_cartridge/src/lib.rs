mod mbc;

use gb_shared::{MachineModel, Snapshot};
use std::{borrow::Cow, fmt::Display};

const CARRIAGE_TYPE: [(u8, &str); 28] = [
    (0x00, "ROM ONLY"),
    (0x01, "MBC1"),
    (0x02, "MBC1+RAM"),
    (0x03, "MBC1+RAM+BATTERY"),
    (0x05, "MBC2"),
    (0x06, "MBC2+BATTERY"),
    (0x08, "ROM+RAM"),
    (0x09, "ROM+RAM+BATTERY"),
    (0x0B, "MMM01"),
    (0x0C, "MMM01+RAM"),
    (0x0D, "MMM01+RAM+BATTERY"),
    (0x0F, "MBC3+TIMER+BATTERY"),
    (0x10, "MBC3+TIMER+RAM+BATTERY"),
    (0x11, "MBC3"),
    (0x12, "MBC3+RAM"),
    (0x13, "MBC3+RAM+BATTERY"),
    (0x19, "MBC5"),
    (0x1A, "MBC5+RAM"),
    (0x1B, "MBC5+RAM+BATTERY"),
    (0x1C, "MBC5+RUMBLE"),
    (0x1D, "MBC5+RUMBLE+RAM"),
    (0x1E, "MBC5+RUMBLE+RAM+BATTERY"),
    (0x20, "MBC6"),
    (0x22, "MBC7+SENSOR+RUMBLE+RAM+BATTERY"),
    (0xFC, "POCKET CAMERA"),
    (0xFD, "BANDAI TAMA5"),
    (0xFE, "HuC3"),
    (0xFF, "HuC1+RAM+BATTERY"),
];

const OLD_LICENSEE_CODE: [(u8, &str); 147] = [
    (0x00, "None"),
    (0x01, "Nintendo"),
    (0x08, "Capcom"),
    (0x09, "Hot-B"),
    (0x0A, "Jaleco"),
    (0x0B, "Coconuts Japan"),
    (0x0C, "Elite Systems"),
    (0x13, "EA (Electronic Arts)"),
    (0x18, "Hudsonsoft"),
    (0x19, "ITC Entertainment"),
    (0x1A, "Yanoman"),
    (0x1D, "Japan Clary"),
    (0x1F, "Virgin Interactive"),
    (0x24, "PCM Complete"),
    (0x25, "San-X"),
    (0x28, "Kotobuki Systems"),
    (0x29, "Seta"),
    (0x30, "Infogrames"),
    (0x31, "Nintendo"),
    (0x32, "Bandai"),
    (0x33, "Indicates that the New licensee code should be used instead."),
    (0x34, "Konami"),
    (0x35, "HectorSoft"),
    (0x38, "Capcom"),
    (0x39, "Banpresto"),
    (0x3C, ".Entertainment i"),
    (0x3E, "Gremlin"),
    (0x41, "Ubisoft"),
    (0x42, "Atlus"),
    (0x44, "Malibu"),
    (0x46, "Angel"),
    (0x47, "Spectrum Holoby"),
    (0x49, "Irem"),
    (0x4A, "Virgin Interactive"),
    (0x4D, "Malibu"),
    (0x4F, "U.S. Gold"),
    (0x50, "Absolute"),
    (0x51, "Acclaim"),
    (0x52, "Activision"),
    (0x53, "American Sammy"),
    (0x54, "GameTek"),
    (0x55, "Park Place"),
    (0x56, "LJN"),
    (0x57, "Matchbox"),
    (0x59, "Milton Bradley"),
    (0x5A, "Mindscape"),
    (0x5B, "Romstar"),
    (0x5C, "Naxat Soft"),
    (0x5D, "Tradewest"),
    (0x60, "Titus"),
    (0x61, "Virgin Interactive"),
    (0x67, "Ocean Interactive"),
    (0x69, "EA (Electronic Arts)"),
    (0x6E, "Elite Systems"),
    (0x6F, "Electro Brain"),
    (0x70, "Infogrames"),
    (0x71, "Interplay"),
    (0x72, "Broderbund"),
    (0x73, "Sculptered Soft"),
    (0x75, "The Sales Curve"),
    (0x78, "t.hq"),
    (0x79, "Accolade"),
    (0x7A, "Triffix Entertainment"),
    (0x7C, "Microprose"),
    (0x7F, "Kemco"),
    (0x80, "Misawa Entertainment"),
    (0x83, "Lozc"),
    (0x86, "Tokuma Shoten Intermedia"),
    (0x8B, "Bullet-Proof Software"),
    (0x8C, "Vic Tokai"),
    (0x8E, "Ape"),
    (0x8F, "I’Max"),
    (0x91, "Chunsoft Co."),
    (0x92, "Video System"),
    (0x93, "Tsubaraya Productions Co."),
    (0x95, "Varie Corporation"),
    (0x96, "Yonezawa/S’Pal"),
    (0x97, "Kaneko"),
    (0x99, "Arc"),
    (0x9A, "Nihon Bussan"),
    (0x9B, "Tecmo"),
    (0x9C, "Imagineer"),
    (0x9D, "Banpresto"),
    (0x9F, "Nova"),
    (0xA1, "Hori Electric"),
    (0xA2, "Bandai"),
    (0xA4, "Konami"),
    (0xA6, "Kawada"),
    (0xA7, "Takara"),
    (0xA9, "Technos Japan"),
    (0xAA, "Broderbund"),
    (0xAC, "Toei Animation"),
    (0xAD, "Toho"),
    (0xAF, "Namco"),
    (0xB0, "acclaim"),
    (0xB1, "ASCII or Nexsoft"),
    (0xB2, "Bandai"),
    (0xB4, "Square Enix"),
    (0xB6, "HAL Laboratory"),
    (0xB7, "SNK"),
    (0xB9, "Pony Canyon"),
    (0xBA, "Culture Brain"),
    (0xBB, "Sunsoft"),
    (0xBD, "Sony Imagesoft"),
    (0xBF, "Sammy"),
    (0xC0, "Taito"),
    (0xC2, "Kemco"),
    (0xC3, "Squaresoft"),
    (0xC4, "Tokuma Shoten Intermedia"),
    (0xC5, "Data East"),
    (0xC6, "Tonkinhouse"),
    (0xC8, "Koei"),
    (0xC9, "UFL"),
    (0xCA, "Ultra"),
    (0xCB, "Vap"),
    (0xCC, "Use Corporation"),
    (0xCD, "Meldac"),
    (0xCE, ".Pony Canyon or"),
    (0xCF, "Angel"),
    (0xD0, "Taito"),
    (0xD1, "Sofel"),
    (0xD2, "Quest"),
    (0xD3, "Sigma Enterprises"),
    (0xD4, "ASK Kodansha Co."),
    (0xD6, "Naxat Soft"),
    (0xD7, "Copya System"),
    (0xD9, "Banpresto"),
    (0xDA, "Tomy"),
    (0xDB, "LJN"),
    (0xDD, "NCS"),
    (0xDE, "Human"),
    (0xDF, "Altron"),
    (0xE0, "Jaleco"),
    (0xE1, "Towa Chiki"),
    (0xE2, "Yutaka"),
    (0xE3, "Varie"),
    (0xE5, "Epcoh"),
    (0xE7, "Athena"),
    (0xE8, "Asmik ACE Entertainment"),
    (0xE9, "Natsume"),
    (0xEA, "King Records"),
    (0xEB, "Atlus"),
    (0xEC, "Epic/Sony Records"),
    (0xEE, "IGS"),
    (0xF0, "A Wave"),
    (0xF3, "Extreme Entertainment"),
    (0xFF, "LJN"),
];

const NEW_LICENSEE_CODE: [(u8, &str); 61] = [
    (0x00, "None"),
    (0x01, "Nintendo R&D1"),
    (0x08, "Capcom"),
    (0x13, "Electronic Arts"),
    (0x18, "Hudson Soft"),
    (0x19, "b-ai"),
    (0x20, "kss"),
    (0x22, "pow"),
    (0x24, "PCM Complete"),
    (0x25, "san-x"),
    (0x28, "Kemco Japan"),
    (0x29, "seta"),
    (0x30, "Viacom"),
    (0x31, "Nintendo"),
    (0x32, "Bandai"),
    (0x33, "Ocean/Acclaim"),
    (0x34, "Konami"),
    (0x35, "Hector"),
    (0x37, "Taito"),
    (0x38, "Hudson"),
    (0x39, "Banpresto"),
    (0x41, "Ubi Soft"),
    (0x42, "Atlus"),
    (0x44, "Malibu"),
    (0x46, "angel"),
    (0x47, "Bullet-Proof"),
    (0x49, "irem"),
    (0x50, "Absolute"),
    (0x51, "Acclaim"),
    (0x52, "Activision"),
    (0x53, "American sammy"),
    (0x54, "Konami"),
    (0x55, "Hi tech entertainment"),
    (0x56, "LJN"),
    (0x57, "Matchbox"),
    (0x58, "Mattel"),
    (0x59, "Milton Bradley"),
    (0x60, "Titus"),
    (0x61, "Virgin"),
    (0x64, "LucasArts"),
    (0x67, "Ocean"),
    (0x69, "Electronic Arts"),
    (0x70, "Infogrames"),
    (0x71, "Interplay"),
    (0x72, "Broderbund"),
    (0x73, "sculptured"),
    (0x75, "sci"),
    (0x78, "THQ"),
    (0x79, "Accolade"),
    (0x80, "misawa"),
    (0x83, "lozc"),
    (0x86, "Tokuma Shoten Intermedia"),
    (0x87, "Tsukuda Original"),
    (0x91, "Chunsoft"),
    (0x92, "Video system"),
    (0x93, "Ocean/Acclaim"),
    (0x95, "Varie"),
    (0x96, "Yonezawa/s’pal"),
    (0x97, "Kaneko"),
    (0x99, "Pack in soft"),
    (0xA4, "Konami (Yu-Gi-Oh!)"),
];

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct CartridgeHeader {
    /// [0x0100, 0x0104)
    pub entry: [u8; 4],
    /// [0x0104, 0x0134)
    pub logo: [u8; 0x30],
    /// [0x0134, 0x0144)
    pub title: [u8; 16],

    /// [0x0144, 0x0146)
    ///
    /// It's only meaningful if the OldLicenseeCode is exactly 0x33,
    /// otherwise, OldLicenseeCode must be considered.
    pub new_licensee_code: u16,
    /// [0x0146, 0x0147)
    ///
    /// Indicates whether the game supports SGB functions.
    pub sgb_flag: u8,
    /// [0x0147, 0x0148)
    ///
    /// Indicates What kind of hardware is present on the cartridge.
    /// https://gbdev.io/pandocs/MBCs.html#mbcs
    pub cart_type: u8,
    /// [0x0148, 0x0149)
    ///
    /// The actual ROM size is given by 32KiB * (1 << <rom_size>)
    pub rom_size: u8,
    /// [0x0149, 0x014A]
    ///
    /// Indicates how much RAM is present on the cartridge, if any.
    /// - 0x00: No RAM
    /// - 0x01: Unused
    /// - 0x02: 8KiB, 1 bank
    /// - 0x03: 32KiB, 4 banks of 8KiB each
    /// - 0x04: 128KiB, 16 banks of 8KiB each
    /// - 0x05: 64KiB, 8 banks of 8KiB each
    pub ram_size: u8,
    /// [0x014A, 0x014B)
    ///
    /// Specifies whether the game is sold in Japan or elsewhere.
    /// - 0x00: Japan(and possibly overseas)
    /// - 0x01: Overseas only
    pub dest_code: u8,
    /// [0x014B, 0x014C)
    ///
    /// If the value is 0x33, new_licensee_code must be consider instead.
    pub licensee_code: u8,
    /// [0x014C, 0x014D)
    ///
    /// Specifies the version of the game. It's usually 0x00.
    pub version: u8,
    /// [0x014D, 0x014E)
    ///
    /// Checksum of [0x0134, 0x014D). About algorithm, see https://gbdev.io/pandocs/The_Cartridge_Header.html#014d--header-checksum
    pub checksum: u8,
    /// [0x014E, 0x0150)
    ///
    /// Checksum of [0x0134, 0x014E). Big-endian.
    pub global_checksum: u16,
}

impl Display for CartridgeHeader {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let title = {
            let mut title = self.title;
            title[15] = 0;
            title
        };
        let title = std::str::from_utf8(&title).unwrap();
        let licensee_code = match self.licensee_code {
            0x33 => {
                OLD_LICENSEE_CODE.iter().find(|c| c.0 as u16 == self.new_licensee_code).map(|c| c.1)
            }
            _ => NEW_LICENSEE_CODE.iter().find(|c| c.0 == self.licensee_code).map(|c| c.1),
        }
        .unwrap_or("Unkown");

        let sgb = match self.sgb_flag {
            0x03 => "Enabled",
            _ => "Disabled",
        };

        let cart_type =
            CARRIAGE_TYPE.iter().find(|c| c.0 == self.cart_type).map(|c| c.1).unwrap_or("Unknown");

        let rom_size = 32 * (1 << self.rom_size); // KiB

        let ram_size: Cow<str> = match self.ram_size {
            0x00 => Cow::Borrowed("0"),
            0x01 => Cow::Borrowed("-"),
            0x02 => Cow::Borrowed("8KiB"),
            0x03 => Cow::Borrowed("32KiB"),
            0x04 => Cow::Borrowed("128KiB"),
            0x05 => Cow::Borrowed("64KiB"),
            _ => Cow::Owned(std::format!("Unknown size: {}", self.ram_size)),
        };

        let dest = match self.dest_code {
            0x00 => "Japan",
            _ => "Overseas",
        };

        write!(
            f,
            "\nTitle: {}\nLicensee code: {}\nSGB: {}\nCartridge type: {}\nROM size: {}KiB\nRAM size: {}\nDest: {}\nVersion: {}\nChecksum: {}\nGlobal checksum: {}",
            title,
            licensee_code,
            sgb,
            cart_type,
            rom_size,
            ram_size,
            dest,
            self.version,
            self.checksum,
            self.global_checksum,
        )
    }
}

pub struct Cartridge {
    pub header: CartridgeHeader,
    pub rom: Vec<u8>,
    mbc: Box<dyn mbc::Mbc<Snapshot = Vec<u8>>>,
}

impl TryFrom<Vec<u8>> for Cartridge {
    type Error = anyhow::Error;

    fn try_from(rom: Vec<u8>) -> std::result::Result<Self, Self::Error> {
        let header = unsafe {
            std::mem::transmute_copy::<[u8; 0x50], CartridgeHeader>(
                &rom[0x0100..0x0150].try_into().unwrap(),
            )
        };

        let checksum = rom[0x0134..0x014D]
            .iter()
            .fold(0u8, |checksum, v| checksum.wrapping_sub(v.wrapping_add(1)));

        assert_eq!(checksum, header.checksum);

        log::trace!("{}", &header);

        let mbc: Box<dyn mbc::Mbc<Snapshot = Vec<u8>>> = match &header.cart_type {
            0x00 => Box::new(mbc::mbc_none::MbcNone::new()),
            0x01..=0x03 => Box::new(mbc::mbc1::Mbc1::new(&header)),
            0x05..=0x06 => Box::new(mbc::mbc2::Mbc2::new(&header)),
            0x0F..=0x13 => Box::new(mbc::mbc3::Mbc3::new(&header)),
            0x19..=0x1E => Box::new(mbc::mbc5::Mbc5::new(&header)),
            _ => panic!(
                "MBC {} is not supported yet",
                CARRIAGE_TYPE
                    .iter()
                    .find(|c| c.0 == header.cart_type)
                    .map(|c| c.1)
                    .unwrap_or("Unknown")
            ),
        };

        Ok(Cartridge { header, rom, mbc })
    }
}

impl gb_shared::Memory for Cartridge {
    fn write(&mut self, addr: u16, value: u8) {
        self.mbc.write(addr, value)
    }

    fn read(&self, addr: u16) -> u8 {
        self.mbc.read(addr, &self.rom)
    }
}

impl Snapshot for Cartridge {
    type Snapshot = Vec<u8>;

    fn take_snapshot(&self) -> Self::Snapshot {
        self.mbc.take_snapshot()
    }

    fn restore_snapshot(&mut self, snapshot: Self::Snapshot) {
        self.mbc.restore_snapshot(snapshot)
    }
}

impl Cartridge {
    pub fn suspend(&self) -> Option<Vec<u8>> {
        self.mbc.suspend()
    }

    pub fn resume(&mut self, data: &[u8]) -> anyhow::Result<()> {
        self.mbc.resume(data)
    }

    pub fn machine_model(&self) -> MachineModel {
        let cgb_flag = self.header.title[15];
        match cgb_flag {
            0xC0 | 0x80 => MachineModel::CGB,
            _ => MachineModel::DMG,
        }
    }

    /// @see https://web.archive.org/web/20170830061747/http://www.vcfed.org/forum/showthread.php?19247-Disassembling-the-GBC-Boot-ROM&p=128734
    /// @see https://gbdev.io/pandocs/Power_Up_Sequence.html?highlight=string%20%22#compatibility-palettes
    pub fn compatibility_palette_id(&self) -> Option<u16> {
        if let MachineModel::CGB = self.machine_model() {
            return None;
        }

        let ok = if self.header.licensee_code == 0x33 {
            // String "01"
            self.header.new_licensee_code == 0x3130
        } else {
            self.header.licensee_code == 0x01
        };
        if !ok {
            return Some(0x0000);
        }

        const TITLE_CHECKSUM_LOOKUP_TABLE: &[u8; 79] = &[
            // 65
            0x00, 0x88, 0x16, 0x36, 0xD1, 0xDB, 0xF2, 0x3C, 0x8C, 0x92, 0x3D, 0x5C, 0x58, 0xC9,
            0x3E, 0x70, 0x1D, 0x59, 0x69, 0x19, 0x35, 0xA8, 0x14, 0xAA, 0x75, 0x95, 0x99, 0x34,
            0x6F, 0x15, 0xFF, 0x97, 0x4B, 0x90, 0x17, 0x10, 0x39, 0xF7, 0xF6, 0xA2, 0x49, 0x4E,
            0x43, 0x68, 0xE0, 0x8B, 0xF0, 0xCE, 0x0C, 0x29, 0xE8, 0xB7, 0x86, 0x9A, 0x52, 0x01,
            0x9D, 0x71, 0x9C, 0xBD, 0x5D, 0x6D, 0x67, 0x3F, 0x6B, //
            // 14
            0xB3, 0x46, 0x28, 0xA5, 0xC6, 0xD3, 0x27, 0x61, 0x18, 0x66, 0x6A, 0xBF, 0x0D, 0xF4,
        ];
        let checksum = self.header.title.iter().fold(0u8, |sum, x| sum.wrapping_add(*x));
        if let Some(index) = TITLE_CHECKSUM_LOOKUP_TABLE.iter().position(|&v| v == checksum) {
            if index <= 64 {
                return Some((checksum as u16) << 8);
            } else {
                const LETTERS: &str = "BEFAARBEKEK R-URAR INAILICE R"; // 14 + 14 + 1
                let the_4th_letter = self.header.title[3];
                let offset = index - 65; // 0-13

                let bytes = LETTERS.as_bytes();
                for x in (offset..bytes.len()).step_by(14) {
                    if bytes[x] == the_4th_letter {
                        return Some((checksum as u16) << 8 | (the_4th_letter as u16));
                    }
                }

                return Some(0x0000);
            }
        }

        Some(0x0000)
    }
}
