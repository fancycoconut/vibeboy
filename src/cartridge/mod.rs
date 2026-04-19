mod mbc0;
mod mbc1;
mod mbc3;

pub use mbc0::Mbc0;
pub use mbc1::Mbc1;
pub use mbc3::Mbc3;

/// Common interface for all cartridge types.
pub trait Cartridge {
    fn read(&self, addr: u16) -> u8;
    fn write(&mut self, addr: u16, val: u8);
    fn ram_read(&self, addr: u16) -> u8;
    fn ram_write(&mut self, addr: u16, val: u8);
}

/// Parsed Game Boy cartridge header (0x0100–0x014F).
pub struct CartridgeHeader {
    pub title: String,
    pub cartridge_type: u8,
    pub rom_size: u8,
    pub ram_size: u8,
    pub cgb_flag: u8,
}

impl CartridgeHeader {
    pub fn parse(rom: &[u8]) -> Self {
        let title_bytes = &rom[0x0134..=0x0143];
        let title = String::from_utf8_lossy(title_bytes)
            .trim_matches('\0')
            .to_string();
        Self {
            title,
            cartridge_type: rom[0x0147],
            rom_size: rom[0x0148],
            ram_size: rom[0x0149],
            cgb_flag: rom[0x0143],
        }
    }

    pub fn is_gbc(&self) -> bool {
        self.cgb_flag == 0x80 || self.cgb_flag == 0xC0
    }
}

/// Create the appropriate cartridge implementation based on the ROM header.
pub fn load(rom: Vec<u8>) -> Box<dyn Cartridge> {
    let header = CartridgeHeader::parse(&rom);
    println!(
        "[Cart] Title: {:?}  Type: 0x{:02X}  ROM: {}  RAM: {}  GBC: {}",
        header.title,
        header.cartridge_type,
        header.rom_size,
        header.ram_size,
        header.is_gbc()
    );

    match header.cartridge_type {
        0x00 => Box::new(Mbc0::new(rom)),
        0x01..=0x03 => Box::new(Mbc1::new(rom, header.ram_size)),
        0x0F..=0x13 => Box::new(Mbc3::new(rom, header.ram_size)),
        t => panic!("Unsupported cartridge type: 0x{t:02X}"),
    }
}
