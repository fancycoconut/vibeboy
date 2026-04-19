use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Config {
    pub display: DisplayConfig,
    pub keybindings: KeyBindings,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DisplayConfig {
    /// Integer scale factor applied to the 160×144 Game Boy screen.
    pub scale: u32,
}

/// Maps each Game Boy input to an SDL2 keycode name string.
/// See: https://docs.rs/sdl2/latest/sdl2/keyboard/enum.Keycode.html
#[derive(Debug, Serialize, Deserialize)]
pub struct KeyBindings {
    pub right: String,
    pub left: String,
    pub up: String,
    pub down: String,
    pub a: String,
    pub b: String,
    pub start: String,
    pub select: String,
    pub quit: String,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            display: DisplayConfig { scale: 3 },
            keybindings: KeyBindings {
                right:  "Right".into(),
                left:   "Left".into(),
                up:     "Up".into(),
                down:   "Down".into(),
                a:      "Z".into(),
                b:      "X".into(),
                start:  "Return".into(),
                select: "LShift".into(),
                quit:   "Escape".into(),
            },
        }
    }
}

impl Config {
    const PATH: &'static str = "vibeboy.toml";

    /// Load config from `vibeboy.toml` next to the binary.
    /// Falls back to defaults if the file is missing; exits on parse errors.
    pub fn load() -> Self {
        let text = match std::fs::read_to_string(Self::PATH) {
            Ok(t) => t,
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Self::default(),
            Err(e) => {
                eprintln!("Failed to read {}: {e}", Self::PATH);
                return Self::default();
            }
        };

        match toml::from_str(&text) {
            Ok(cfg) => cfg,
            Err(e) => {
                eprintln!("Failed to parse {}: {e}", Self::PATH);
                eprintln!("Using default config.");
                Self::default()
            }
        }
    }

}
