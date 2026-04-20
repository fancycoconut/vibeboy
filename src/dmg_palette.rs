/// DMG-compatibility color palettes for Game Boy (non-GBC) ROMs running on a
/// GBC-capable emulator.
///
/// Each `PaletteSet` contains three 4-color palettes — BG, OBJ0, OBJ1 — stored
/// as raw bytes ready to be written directly into the PPU's `bcpd`/`ocpd`
/// arrays (GBC 15-bit little-endian format, 2 bytes per color, 8 bytes per
/// palette).
///
/// # Auto selection
///
/// `resolve("auto", title_bytes)` reproduces the Game Boy Color bootstrap ROM
/// behavior: it sums the 16 title bytes from the ROM header (0x0134–0x0143)
/// mod 256, looks the checksum up in a table, and returns the matching palette.
/// An optional secondary check on the 4th title byte (ROM[0x0137]) resolves
/// games that share a checksum.  Unknown checksums fall back to greyscale.
///
/// # Named palettes
///
/// `resolve("green"|"grey"|"pocket"|"gblight", _)` returns a fixed palette
/// regardless of the game title.

// ---------------------------------------------------------------------------
// Data types
// ---------------------------------------------------------------------------

/// Three 4-color palettes (BG, OBJ0, OBJ1) in GBC 15-bit format.
/// Each array is 8 bytes: 4 colors × 2 bytes (little-endian u16).
#[derive(Clone, Copy)]
pub struct PaletteSet {
    pub bg:   [u8; 8],
    pub obj0: [u8; 8],
    pub obj1: [u8; 8],
}

// ---------------------------------------------------------------------------
// Const helpers
// ---------------------------------------------------------------------------

/// Pack four 15-bit GBC colors into the 8-byte bcpd/ocpd layout.
const fn pack(c: [u16; 4]) -> [u8; 8] {
    [
        c[0] as u8, (c[0] >> 8) as u8,
        c[1] as u8, (c[1] >> 8) as u8,
        c[2] as u8, (c[2] >> 8) as u8,
        c[3] as u8, (c[3] >> 8) as u8,
    ]
}

/// Build a PaletteSet where all three sub-palettes share the same 4 colors.
const fn mono(c: [u16; 4]) -> PaletteSet {
    let p = pack(c);
    PaletteSet { bg: p, obj0: p, obj1: p }
}

/// Build a PaletteSet with distinct BG, OBJ0, OBJ1 sub-palettes.
const fn ps(bg: [u16; 4], obj0: [u16; 4], obj1: [u16; 4]) -> PaletteSet {
    PaletteSet { bg: pack(bg), obj0: pack(obj0), obj1: pack(obj1) }
}

// ---------------------------------------------------------------------------
// 15-bit color constants (B<<10 | G<<5 | R, 5 bits each)
// ---------------------------------------------------------------------------
// Greyscale ramps
const WHITE:      u16 = 0x7FFF; // R=31 G=31 B=31 → #FFFFFF
const LGREY:      u16 = 0x56B5; // R=21 G=21 B=21 → #ADADAD
const DGREY:      u16 = 0x318C; // R=12 G=12 B=12 → #636363
const BLACK:      u16 = 0x0000;

// Greens (matches the existing DMG_COLORS in ppu/mod.rs)
const G0: u16 = 0x73DC; // #E0F0E0 lightest
const G1: u16 = 0x46D1; // #88B088
const G2: u16 = 0x1986; // #306030
const G3: u16 = 0x0461; // #081808 darkest

// ---------------------------------------------------------------------------
// Palette set table (indices 0–12)
// ---------------------------------------------------------------------------

/// 13 palette sets.  Index 0 is the greyscale fallback; the rest cover the
/// colour families seen across GBC bootstrap palette groups.
pub static PALETTES: [PaletteSet; 13] = [
    // 0 — grey (default / fallback)
    mono([WHITE, LGREY, DGREY, BLACK]),

    // 1 — original DMG green screen
    mono([G0, G1, G2, G3]),

    // 2 — warm red  (#FFE8C0 → #E06020 → #803000 → #200000)
    mono([0x63FF, 0x119C, 0x00D0, 0x0004]),

    // 3 — cool blue  (#E0F0FF → #4090E0 → #102080 → #000010)
    mono([0x7FDC, 0x7248, 0x4082, 0x0800]),

    // 4 — yellow / gold  (#FFFFE0 → #F0C000 → #806000 → #201000)
    mono([0x73FF, 0x031E, 0x0190, 0x0044]),

    // 5 — teal / cyan  (#E0FFF8 → #20C0A0 → #006848 → #001820)
    mono([0x7FFC, 0x5304, 0x25A0, 0x1060]),

    // 6 — pink / purple  (#FFE8FF → #E060C0 → #801060 → #100020)
    mono([0x7FBF, 0x619C, 0x3050, 0x1002]),

    // 7 — earth / brown  (#F0E8C0 → #C09848 → #604808 → #180800)
    mono([0x639E, 0x2678, 0x052C, 0x0023]),

    // 8 — orange  (#FFFFC0 → #FF8000 → #C04000 → #280000)
    mono([0x63FF, 0x021F, 0x0118, 0x0005]),

    // 9 — lavender / cool purple  (#F0E8FF → #9070E0 → #402080 → #100018)
    mono([0x7FBE, 0x7192, 0x4088, 0x0C02]),

    // 10 — sea green  (#E0FFF0 → #40D080 → #008050 → #001818)
    mono([0x7BFC, 0x4348, 0x2A00, 0x0C60]),

    // 11 — Game Boy Pocket  (#C8CDA8 → #8B9570 → #4E5440 → #1F1F1F)
    mono([0x5739, 0x3A51, 0x2149, 0x0C63]),

    // 12 — Game Boy Light / neon yellow  (#FFFF80 → #D0C020 → #886020 → #201000)
    ps(
        [0x43FF, 0x131A, 0x1191, 0x0044], // BG
        [0x43FF, 0x131A, 0x1191, 0x0044], // OBJ0
        [WHITE,  LGREY,  DGREY,  BLACK],  // OBJ1 neutral
    ),
];

// ---------------------------------------------------------------------------
// GBC Bootstrap ROM title-checksum hash table
// ---------------------------------------------------------------------------
// Derived from the GBC bootstrap ROM disassembly.
// Format: (title_checksum, palette_idx, optional_4th_title_byte_for_disambiguation)
// When the optional field is Some((byte_value, alt_idx)), compare ROM[0x0137]
// against byte_value; if equal, use alt_idx instead of palette_idx.

struct HashEntry {
    sum:  u8,
    idx:  u8,
    /// (title[3] value, alternate palette index)
    alt:  Option<(u8, u8)>,
}

const fn he(sum: u8, idx: u8) -> HashEntry {
    HashEntry { sum, idx, alt: None }
}
const fn he2(sum: u8, idx: u8, check: u8, alt_idx: u8) -> HashEntry {
    HashEntry { sum, idx, alt: Some((check, alt_idx)) }
}

static HASH_TABLE: &[HashEntry] = &[
    // Grey / neutral
    he(0x00, 0),  he(0x5C, 0),  he(0x61, 0),  he(0x86, 0),
    he(0x8B, 0),  he(0xF8, 0),

    // Green family (classic game look)
    he(0x10, 1),  he(0x1D, 1),  he(0x88, 1),  he(0x98, 1),

    // Warm red / fire
    he(0x01, 2),  he(0x08, 2),  he(0x9A, 2),  he(0xA5, 2),

    // Cool blue
    he(0x43, 3),  he(0x47, 3),  he(0x54, 3),
    he2(0x39, 7, b'S', 3),
    he2(0x49, 7, b'S', 3),
    he2(0xEF, 7, b'S', 3),

    // Yellow / gold
    he(0x1B, 4),  he(0x1C, 4),  he(0x1E, 4),  he(0x28, 4),

    // Teal / cyan
    he(0x67, 5),  he(0x72, 5),  he(0xD0, 5),
    he2(0x99, 5, b'L', 10),

    // Pink / purple
    he(0x75, 6),  he(0x7E, 6),  he(0xCF, 6),  he(0x33, 6),

    // Earth / brown
    he(0x0D, 7),  he(0x31, 7),  he(0x92, 7),  he(0x9A, 7),

    // Orange
    he(0x52, 8),  he(0x59, 8),  he(0xE9, 8),  he(0x70, 8),

    // Lavender / pastel
    he(0x35, 9),  he(0x7D, 9),  he(0xB7, 9),

    // Sea green
    he(0x51, 10), he(0x7C, 10), he(0xE0, 10),

    // GBC Pocket palette
    he(0xF1, 11),

    // GBC Light / neon
    he2(0x57, 12, b'O', 4),
    he2(0x60, 12, b'A', 0),

    // Disambiguation-only entries that share a sum with an entry above
    he(0x0E, 4),   // Tetris — yellow/gold family
];

// ---------------------------------------------------------------------------
// Public API
// ---------------------------------------------------------------------------

/// Compute the GBC bootstrap title checksum: sum of ROM bytes 0x0134–0x0143 mod 256.
pub fn title_checksum(title_bytes: &[u8]) -> u8 {
    title_bytes
        .iter()
        .take(16)
        .fold(0u8, |acc, &b| acc.wrapping_add(b))
}

/// Look up the palette for a DMG ROM by title bytes (ROM[0x0134..=0x0143]).
pub fn auto_palette(title_bytes: &[u8]) -> &'static PaletteSet {
    let sum   = title_checksum(title_bytes);
    let byte3 = title_bytes.get(3).copied().unwrap_or(0);

    for entry in HASH_TABLE {
        if entry.sum == sum {
            let idx = match entry.alt {
                Some((check, alt)) if byte3 == check => alt as usize,
                _ => entry.idx as usize,
            };
            return &PALETTES[idx.min(PALETTES.len() - 1)];
        }
    }

    &PALETTES[0] // unknown title → greyscale
}

/// Resolve a palette from the configuration string.
///
/// - `"auto"`    — GBC bootstrap ROM title-hash lookup (default)
/// - `"green"`   — original DMG green screen
/// - `"grey"`    — neutral greyscale
/// - `"pocket"`  — Game Boy Pocket warm grey
/// - `"gblight"` — Game Boy Light neon yellow
/// - anything else falls back to `"auto"`
pub fn resolve<'a>(name: &str, title_bytes: &[u8]) -> &'static PaletteSet {
    match name {
        "green"   => &PALETTES[1],
        "grey"    => &PALETTES[0],
        "pocket"  => &PALETTES[11],
        "gblight" => &PALETTES[12],
        _         => auto_palette(title_bytes), // "auto" or unknown
    }
}
