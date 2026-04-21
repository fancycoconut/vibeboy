/// Accurate GBC DMG-compatibility palettes decoded from the GBC boot ROM binary.

#[derive(Clone, Copy)]
pub struct PaletteSet {
    pub bg:   [u8; 8],
    pub obj0: [u8; 8],
    pub obj1: [u8; 8],
}

const fn pack(c: [u16; 4]) -> [u8; 8] {
    [
        c[0] as u8, (c[0] >> 8) as u8,
        c[1] as u8, (c[1] >> 8) as u8,
        c[2] as u8, (c[2] >> 8) as u8,
        c[3] as u8, (c[3] >> 8) as u8,
    ]
}

const RAW: [[u16; 4]; 35] = [
    [0x7FFF, 0x32BF, 0x00D0, 0x0000], // 0
    [0x639F, 0x4279, 0x15B0, 0x04CB], // 1
    [0x7FFF, 0x6E31, 0x454A, 0x0000], // 2
    [0x7FFF, 0x1BEF, 0x0200, 0x0000], // 3
    [0x7FFF, 0x421F, 0x1CF2, 0x0000], // 4
    [0x7FFF, 0x5294, 0x294A, 0x0000], // 5
    [0x7FFF, 0x03FF, 0x012F, 0x0000], // 6
    [0x7FFF, 0x03EF, 0x01D6, 0x0000], // 7
    [0x7FFF, 0x42B5, 0x3DC8, 0x0000], // 8
    [0x7E74, 0x03FF, 0x0180, 0x0000], // 9
    [0x67FF, 0x77AC, 0x1A13, 0x2D6B], // 10
    [0x7ED6, 0x4BFF, 0x2175, 0x0000], // 11
    [0x53FF, 0x4A5F, 0x7E52, 0x0000], // 12
    [0x4FFF, 0x7ED2, 0x3A4C, 0x1CE0], // 13
    [0x03ED, 0x7FFF, 0x255F, 0x0000], // 14
    [0x036A, 0x021F, 0x03FF, 0x7FFF], // 15
    [0x7FFF, 0x01DF, 0x0112, 0x0000], // 16
    [0x231F, 0x035F, 0x00F2, 0x0009], // 17
    [0x7FFF, 0x03EA, 0x011F, 0x0000], // 18
    [0x299F, 0x001A, 0x000C, 0x0000], // 19
    [0x7FFF, 0x027F, 0x001F, 0x0000], // 20
    [0x7FFF, 0x03E0, 0x0206, 0x0120], // 21
    [0x7FFF, 0x7EEB, 0x001F, 0x7C00], // 22
    [0x7FFF, 0x3FFF, 0x7E00, 0x001F], // 23
    [0x7FFF, 0x03FF, 0x001F, 0x0000], // 24
    [0x03FF, 0x001F, 0x000C, 0x0000], // 25
    [0x7FFF, 0x033F, 0x0193, 0x0000], // 26
    [0x0000, 0x4200, 0x037F, 0x7FFF], // 27
    [0x7FFF, 0x7E8C, 0x7C00, 0x0000], // 28
    [0x7FFF, 0x1BEF, 0x6180, 0x0000], // 29
    [0x7FFF, 0x7C00, 0x03E0, 0x7C1F], // 30
    [0x001F, 0x03FF, 0x4140, 0x2042], // 31
    [0x2221, 0x8180, 0x1082, 0x1211], // 32
    [0xB012, 0xB879, 0x16AD, 0x0717], // 33
    [0x05BA, 0x137C, 0x0000, 0x0000], // 34
];

const fn ps(bg: usize, obj0: usize, obj1: usize) -> PaletteSet {
    PaletteSet { bg: pack(RAW[bg]), obj0: pack(RAW[obj0]), obj1: pack(RAW[obj1]) }
}

/// Classic DMG green-screen palette (matches the hardcoded DMG_COLORS in the PPU).
const GREEN_SCREEN: PaletteSet = {
    let p = pack([0x73DC, 0x46D1, 0x1986, 0x0461]);
    PaletteSet { bg: p, obj0: p, obj1: p }
};

/// Neutral greyscale palette (white → light grey → dark grey → black).
const NEUTRAL_GREY: PaletteSet = {
    let p = pack([0x7FFF, 0x56B5, 0x318C, 0x0000]);
    PaletteSet { bg: p, obj0: p, obj1: p }
};

struct Entry {
    sum:  u8,
    bg:   u8,
    obj0: u8,
    obj1: u8,
    alt:  Option<(u8, u8, u8, u8)>,
}

const fn e(sum: u8, bg: u8, obj0: u8, obj1: u8) -> Entry {
    Entry { sum, bg, obj0, obj1, alt: None }
}
const fn ea(sum: u8, bg: u8, obj0: u8, obj1: u8, d: u8, a_bg: u8, a_obj0: u8, a_obj1: u8) -> Entry {
    Entry { sum, bg, obj0, obj1, alt: Some((d, a_bg, a_obj0, a_obj1)) }
}

static TABLE: &[Entry] = &[
    e(0x00,  4, 29, 28),
    e(0x88,  9, 19, 22),
    e(0x16,  0,  3, 28),
    e(0x36, 27,  4, 15),
    e(0xD1, 27,  0, 14),
    e(0xDB, 24, 28, 22),
    e(0xF2, 22, 24, 28),
    e(0x3C, 23,  4, 28),
    e(0x8C,  8, 22, 16),
    e(0x92,  0,  3, 28),
    e(0x3D,  4, 18, 22),
    e(0x5C, 19, 22,  9),
    e(0x58,  5,  5,  5),
    e(0xC9, 16, 28, 10),
    e(0x3E, 22, 20,  4),
    e(0x70, 21, 28,  4),
    e(0x1D, 19,  9, 22),
    e(0x59, 16, 22,  8),
    e(0x69, 22, 24, 28),
    e(0x19,  4, 20, 22),
    e(0x35,  0,  3, 28),
    e(0xA8, 17,  4, 13),
    e(0x14,  4, 28,  3),
    e(0xAA, 29, 28,  4),
    e(0x75,  0,  3, 28),
    e(0x95, 22, 18,  4),
    e(0x99,  0,  3, 28),
    e(0x34,  4,  7,  4),
    e(0x6F, 26, 26, 26),
    e(0x15, 24, 28, 22),
    e(0xFF, 20,  4, 22),
    e(0x97, 28,  0,  3),
    e(0x4B,  4,  3, 28),
    e(0x90,  4,  3, 28),
    e(0x17,  4, 28,  3),
    e(0x10, 28,  3,  0),
    e(0x39, 28,  0,  3),
    e(0xF7,  3, 28,  0),
    e(0xF6, 28,  3,  0),
    e(0xA2,  3, 28,  0),
    e(0x49, 19, 22,  9),
    e(0x4E,  4, 23, 28),
    e(0x43, 28,  0,  3),
    e(0x68, 28,  3,  0),
    e(0xE0, 22, 20,  4),
    e(0x8B,  4, 28,  3),
    e(0xF0, 27,  0, 14),
    e(0xCE, 27,  0, 14),
    e(0x0C,  0,  3, 28),
    e(0x29, 28,  3,  0),
    e(0xE8, 27,  4,  3),
    e(0xB7,  0,  3, 28),
    e(0x86, 17,  4, 13),
    e(0x9A,  4,  3, 28),
    e(0x52, 28,  3,  0),
    e(0x01, 28,  3,  0),
    e(0x9D,  4,  0,  2),
    e(0x71, 20,  4, 22),
    e(0x9C, 22, 17,  2),
    e(0xBD,  4,  3, 28),
    e(0x5D, 28,  3,  0),
    e(0x6D, 28,  3,  0),
    e(0x67,  0,  3, 28),
    e(0x3F,  4, 29, 28),
    e(0x6B, 17, 22,  2),
    ea(0xB3, 19, 22,  9,  0x42,  4, 28, 29),
    ea(0x46,  3, 11,  3,  0x45, 16,  8, 22),
    ea(0x28,  4,  3, 28,  0x46, 25,  3, 28),
    ea(0xA5, 27,  4,  3,  0x41, 27,  4,  3),
    ea(0xC6, 16, 22,  8,  0x41,  3,  0, 28),
    ea(0xD3,  2,  0,  4,  0x52,  4, 29, 28),
    ea(0x27, 19, 22,  9,  0x42,  0, 28,  8),
    ea(0x61, 28, 23,  4,  0x45,  4, 28,  3),
    ea(0x18, 17, 22,  2,  0x4B,  4, 28,  3),
    ea(0x66,  4,  7,  4,  0x45,  4, 29, 28),
    ea(0x6A, 17, 22,  2,  0x4B,  4, 29, 28),
    ea(0xBF,  4,  2,  0,  0x20,  4, 18, 22),
    ea(0x0D, 22, 24, 28,  0x52, 27,  0, 14),
];

pub fn title_checksum(title_bytes: &[u8]) -> u8 {
    title_bytes.iter().take(16).fold(0u8, |a, &b| a.wrapping_add(b))
}

pub fn auto_palette(title_bytes: &[u8]) -> PaletteSet {
    let sum   = title_checksum(title_bytes);
    let byte3 = title_bytes.get(3).copied().unwrap_or(0);
    for entry in TABLE {
        if entry.sum == sum {
            let (bg, obj0, obj1) = match entry.alt {
                Some((d, a_bg, a_obj0, a_obj1)) if byte3 == d => {
                    (a_bg as usize, a_obj0 as usize, a_obj1 as usize)
                }
                _ => (entry.bg as usize, entry.obj0 as usize, entry.obj1 as usize),
            };
            return ps(bg, obj0, obj1);
        }
    }
    NEUTRAL_GREY
}

pub fn resolve(name: &str, title_bytes: &[u8]) -> PaletteSet {
    match name {
        "grey"  => NEUTRAL_GREY,
        "green" => GREEN_SCREEN,
        _       => auto_palette(title_bytes),
    }
}
