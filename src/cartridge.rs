//! iNES (.NES) file parser
//!
//! Reference: https://www.nesdev.org/wiki/INES

const INES_IDENTIFIER: [u8; 4] = [0x4E, 0x45, 0x53, 0x1A];
const PRG_ROM_PAGE_SIZE: usize = 16384;
const CHR_ROM_PAGE_SIZE: usize = 8192;

#[derive(Debug, PartialEq, Clone)]
pub enum Mirroring {
    Vertical,
    Horizontal,
    FourScreen,
}
pub struct Cartridge {
    pub prg_rom: Vec<u8>,
    pub chr_rom: Vec<u8>,
    pub mapper: u8,
    pub screen_mirroring: Mirroring,
}

impl Cartridge {
    // Creates a Cartridge from raw .nes file (array of u8s).
    pub fn new(raw: &[u8]) -> Result<Cartridge, String> {
        if raw[0..4] != INES_IDENTIFIER {
            return Err("File is not in iNES file format".to_string());
        }

        let prg_rom_size = raw[4] as usize * PRG_ROM_PAGE_SIZE;
        let chr_rom_size = raw[5] as usize * CHR_ROM_PAGE_SIZE;

        // raw[6] bitflag breakdown
        // ________
        // 76543210
        // ||||||||
        // |||||||+- Nametable arrangement: 0: vertical arrangement ("horizontal mirrored") (CIRAM A10 = PPU A11)
        // |||||||                          1: horizontal arrangement ("vertically mirrored") (CIRAM A10 = PPU A10)
        // ||||||+-- 1: Cartridge contains battery-backed PRG RAM ($6000-7FFF) or other persistent memory
        // |||||+--- 1: 512-byte trainer at $7000-$71FF (stored before PRG data)
        // ||||+---- 1: Alternative nametable layout
        // ++++----- Lower nybble of mapper number

        // If there's a trainers
        let trainer = raw[6] & 0b100 == 0b100;

        let four_screen = raw[6] & 0b1000 != 0;
        let vertical_mirroring = raw[6] & 0b1 != 0;
        let screen_mirroring = match (four_screen, vertical_mirroring) {
            (true, _) => Mirroring::FourScreen,
            (false, true) => Mirroring::Vertical,
            (false, false) => Mirroring::Horizontal,
        };

        let mapper = (raw[7] & 0b1111_0000) | (raw[6] >> 4);

        let ines_ver = (raw[7] >> 2) & 0b11;
        if ines_ver != 0 {
            return Err("NES2.0 format is not supported".to_string());
        }

        // TODO: PRG-RAM size

        let prg_rom_start = 16 + if trainer { 512 } else { 0 };
        let chr_rom_start = prg_rom_start + prg_rom_size;

        Ok(Cartridge {
            prg_rom: raw[prg_rom_start..(prg_rom_start + prg_rom_size)].to_vec(),
            chr_rom: raw[chr_rom_start..(chr_rom_start + chr_rom_size)].to_vec(),
            mapper,
            screen_mirroring,
        })
    }

    // Creates an empty cartridge.
    pub fn default() -> Cartridge {
        const prg_rom_size: usize = 2 * PRG_ROM_PAGE_SIZE;
        const chr_rom_size: usize = 1 * CHR_ROM_PAGE_SIZE;

        Cartridge {
            prg_rom: [0; prg_rom_size].to_vec(),
            chr_rom: [0; chr_rom_size].to_vec(),
            mapper: 0,
            screen_mirroring: Mirroring::Horizontal,
        }
        
    }
}

pub mod test {
    use super::*;

    // Note that we must set the program counter manually with this test cartridge. 0xFFFC will NOT
    // contain the "reset vector."
    pub fn create_test_cartridge() -> Cartridge {
        let mut header = vec![
            0x4E, 0x45, 0x53, 0x1A, 0x02, 0x01, 0x31, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00,
        ];
        let mut pgr_rom = vec![0; 2 * PRG_ROM_PAGE_SIZE];
        let mut chr_rom = vec![0; CHR_ROM_PAGE_SIZE];
        header.append(&mut pgr_rom);
        header.append(&mut chr_rom);
        Cartridge::new(&header).unwrap()
    }

    #[test]
    fn test_invalid_ines_identifier() {
        let raw_data = vec![
            // Incorrect iNES header
            0x00, 0x00, 0x00, 0x00, // Invalid NES<EOF>
            0x02, 0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        ];

        let result = Cartridge::new(&raw_data);
        assert!(result.is_err());
        assert_eq!(result.err().unwrap(), "File is not in iNES file format");
    }
    #[test]
    fn test_unsupported_nes_version() {
        let raw_data = vec![
            // iNES header with NES2.0 version
            0x4E, 0x45, 0x53, 0x1A, // NES<EOF>
            0x02, 0x01, 0x00, 0x08, // NES2.0 version (set bits in flags 7)
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00,
            // PRG ROM data
            // ... (fill as needed)
            // CHR ROM data
            // ... (fill as needed)
        ];

        let result = Cartridge::new(&raw_data);
        assert!(result.is_err());
        assert_eq!(result.err().unwrap(), "NES2.0 format is not supported");
    }
}
