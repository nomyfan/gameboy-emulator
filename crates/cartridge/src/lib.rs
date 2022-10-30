use anyhow::Result;
use std::path::Path;

pub struct CartridgeHeader {
    /// [0x0100, 0x0104)
    pub entry: [u8; 4],
    /// [0x0104, 0x0134)
    pub logo: [u8; 0x30],
    /// [0x0134, 0x0144)
    pub title: [char; 16],

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

pub struct Cartridge {
    pub rom_path: String,
    pub header: CartridgeHeader,
}

impl Cartridge {
    pub fn load(path: &Path) -> Result<Cartridge> {
        todo!()
    }
}
