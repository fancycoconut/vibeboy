use vibeboy::apu::SAMPLE_RATE;
use vibeboy::config::Config;
use vibeboy::dmg_palette;
use vibeboy::gameboy::GameBoy;
use vibeboy::joypad::btn;
use sdl2::audio::AudioSpecDesired;
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::pixels::PixelFormatEnum;
use std::collections::HashMap;
use std::path::PathBuf;
use std::time::{Duration, Instant};

// How much audio to keep buffered before throttling (~4 frames at 59.73 Hz).
// The audio hardware clock at 44100 Hz becomes the emulation speed reference.
const AUDIO_TARGET_BYTES: u32 = SAMPLE_RATE * 2 * 4 * 4 / 60;
// Hard cap (~200 ms) to prevent latency build-up if the audio device stalls.
const AUDIO_MAX_BYTES: u32 = SAMPLE_RATE * 2 * 4 / 5;

fn update_title_fps(
    canvas: &mut sdl2::render::Canvas<sdl2::video::Window>,
    fps_frame_count: &mut u32,
    fps_timer: &mut Instant,
    frames_to_run: u32,
) {
    *fps_frame_count += frames_to_run;
    if fps_timer.elapsed() >= Duration::from_secs(1) {
        let fps = *fps_frame_count as f64 / fps_timer.elapsed().as_secs_f64();
        let pct = (fps / 59.7275 * 100.0).round() as u32;
        canvas.window_mut().set_title(&format!("Vibeboy | {pct}%")).ok();
        *fps_frame_count = 0;
        *fps_timer = Instant::now();
    }
}

fn build_keymap(kb: &vibeboy::config::KeyBindings) -> HashMap<Keycode, usize> {
    [
        (kb.right.as_str(),  btn::RIGHT),
        (kb.left.as_str(),   btn::LEFT),
        (kb.up.as_str(),     btn::UP),
        (kb.down.as_str(),   btn::DOWN),
        (kb.a.as_str(),      btn::A),
        (kb.b.as_str(),      btn::B),
        (kb.start.as_str(),  btn::START),
        (kb.select.as_str(), btn::SELECT),
    ]
    .into_iter()
    .filter_map(|(name, button)| {
        match Keycode::from_name(name) {
            Some(kc) => Some((kc, button)),
            None => {
                eprintln!("Warning: unknown key name '{name}' in vibeboy.toml — binding ignored");
                None
            }
        }
    })
    .collect()
}

fn resolve_quit_key(name: &str) -> Keycode {
    Keycode::from_name(name).unwrap_or_else(|| {
        eprintln!("Warning: unknown quit key '{name}', defaulting to Escape");
        Keycode::ESCAPE
    })
}

/// Create a GameBoy from ROM bytes, applying DMG colorization if configured.
fn make_gameboy(rom: &[u8], config: &Config) -> GameBoy {
    let mut gb = GameBoy::new(rom.to_vec());
    if !gb.bus.ppu.cgb_mode && config.display.dmg_mode != "grey" {
        let title_bytes = rom.get(0x0134..=0x0143).unwrap_or(&[]);
        let palette = dmg_palette::resolve(&config.display.dmg_palette, title_bytes);
        gb.bus.ppu.apply_dmg_compat(&palette);
        println!("[DMG] colour palette: {}", &config.display.dmg_palette);
    }
    gb
}

/// Save the current cartridge RAM to disk, then load a new ROM from `path`.
/// Returns the new GameBoy and its .sav path, or None on read failure.
fn switch_rom(
    gb: &mut GameBoy,
    current_sav: &PathBuf,
    new_rom_path: &str,
    config: &Config,
) -> Option<(GameBoy, PathBuf)> {
    // Persist the current game before switching.
    let sav_data = gb.bus.cartridge.save_ram();
    if !sav_data.is_empty() {
        match std::fs::write(current_sav, &sav_data) {
            Ok(()) => println!("[Save] Wrote save to {}", current_sav.display()),
            Err(e) => eprintln!("[Save] Failed to write {}: {e}", current_sav.display()),
        }
    }

    let rom = match std::fs::read(new_rom_path) {
        Ok(r) => r,
        Err(e) => {
            eprintln!("[Load] Failed to read ROM '{new_rom_path}': {e}");
            return None;
        }
    };

    let new_sav_path = std::path::Path::new(new_rom_path).with_extension("sav");
    let mut new_gb = make_gameboy(&rom, config);

    if let Ok(sav) = std::fs::read(&new_sav_path) {
        new_gb.bus.cartridge.load_ram(&sav);
        println!("[Save] Loaded save from {}", new_sav_path.display());
    }

    println!("[Load] Loaded ROM: {new_rom_path}");
    Some((new_gb, new_sav_path))
}

#[cfg(windows)]
mod windows;

fn main() {
    let config = Config::load();

    #[cfg(windows)]
    windows::apply_terminal_visibility(config.display.windows_os_show_terminal);

    let scale  = config.display.scale;
    let width  = 160 * scale;
    let height = 144 * scale;

    let keymap     = build_keymap(&config.keybindings);
    let quit_kc    = resolve_quit_key(&config.keybindings.quit);
    let speedup_kc = Keycode::from_name(&config.keybindings.speedup).unwrap_or_else(|| {
        eprintln!("Warning: unknown speedup key '{}', defaulting to D", config.keybindings.speedup);
        Keycode::D
    });

    let args: Vec<String> = std::env::args().collect();
    let rom_path = args.get(1).map(String::as_str).unwrap_or("red.gb");

    let rom = std::fs::read(rom_path).unwrap_or_else(|e| {
        eprintln!("Failed to read ROM '{}': {e}", rom_path);
        std::process::exit(1);
    });

    let mut sav_path: PathBuf = std::path::Path::new(rom_path).with_extension("sav");
    let mut gb = make_gameboy(&rom, &config);

    if let Ok(sav) = std::fs::read(&sav_path) {
        gb.bus.cartridge.load_ram(&sav);
        println!("[Save] Loaded save from {}", sav_path.display());
    }

    // -------------------------------------------------------------------------
    // SDL2 setup
    // -------------------------------------------------------------------------
    let sdl = sdl2::init().expect("SDL2 init failed");
    let video = sdl.video().expect("SDL2 video init failed");

    let audio_subsystem = sdl.audio().expect("SDL2 audio init failed");
    let desired_spec = AudioSpecDesired {
        freq: Some(SAMPLE_RATE as i32),
        channels: Some(2),
        samples: Some(1024),
    };
    let audio_queue: sdl2::audio::AudioQueue<f32> = audio_subsystem
        .open_queue(None, &desired_spec)
        .expect("Failed to open audio queue");
    audio_queue.resume();

    let window = video
        .window("Vibeboy", width, height)
        .position_centered()
        .build()
        .expect("Window creation failed");

    let mut canvas = window
        .into_canvas()
        .accelerated()
        .build()
        .expect("Canvas creation failed");

    let texture_creator = canvas.texture_creator();
    let mut texture = texture_creator
        .create_texture_streaming(PixelFormatEnum::RGB24, 160, 144)
        .expect("Texture creation failed");

    let mut event_pump = sdl.event_pump().expect("Event pump failed");

    // -------------------------------------------------------------------------
    // Main loop
    // -------------------------------------------------------------------------
    // When sound is disabled we use a simple timer to cap at ~59.73 Hz.
    const FRAME_DURATION: Duration = Duration::from_nanos(16_742_706); // 1 / 59.7275 Hz
    const SPEED_FACTOR: u32 = 2;
    let mut frame_start = Instant::now();
    let mut speedup_held = false;
    let mut fps_frame_count: u32 = 0;
    let mut fps_timer = Instant::now();

    'running: loop {
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit { .. } => break 'running,
                Event::DropFile { filename, .. } => {
                    if let Some((new_gb, new_sav)) = switch_rom(&mut gb, &sav_path, &filename, &config) {
                        gb = new_gb;
                        sav_path = new_sav;
                        audio_queue.clear();
                    }
                }
                Event::KeyDown { keycode: Some(kc), repeat: false, .. } => {
                    if kc == quit_kc {
                        break 'running;
                    }
                    if kc == speedup_kc {
                        speedup_held = true;
                    }
                    if let Some(&button) = keymap.get(&kc) {
                        gb.bus.joypad.press(button, &mut gb.bus.interrupts);
                    }
                }
                Event::KeyUp { keycode: Some(kc), .. } => {
                    if kc == speedup_kc {
                        speedup_held = false;
                        audio_queue.clear();
                    }
                    if let Some(&button) = keymap.get(&kc) {
                        gb.bus.joypad.release(button);
                    }
                }
                _ => {}
            }
        }

        let frames_to_run = if speedup_held { SPEED_FACTOR } else { 1 };
        for _ in 0..frames_to_run.saturating_sub(1) {
            gb.run_frame();
            gb.bus.apu.drain_samples();
        }
        let framebuffer = gb.run_frame();

        texture
            .with_lock(None, |dst, _pitch| {
                dst.copy_from_slice(framebuffer.as_slice());
            })
            .expect("Texture lock failed");

        let samples = gb.bus.apu.drain_samples();
        if config.sound_enabled && !speedup_held {
            // Queue audio, dropping only if the device has stalled beyond the hard cap.
            if audio_queue.size() < AUDIO_MAX_BYTES {
                audio_queue.queue_audio(&samples).ok();
            }
        }

        canvas.clear();
        canvas.copy(&texture, None, None).expect("Texture copy failed");
        canvas.present();

        if !speedup_held {
            if config.sound_enabled {
                // Throttle emulation to the audio hardware clock. When the queue holds
                // more than ~4 frames of audio, sleep until it drains to that level.
                // This keeps the emulator at exactly 59.73 Hz without relying on vsync
                // or OS sleep precision.
                while audio_queue.size() > AUDIO_TARGET_BYTES {
                    std::thread::sleep(Duration::from_millis(1));
                }
            } else {
                // Without audio throttling, sleep for the remainder of the frame period.
                let elapsed = frame_start.elapsed();
                if let Some(remaining) = FRAME_DURATION.checked_sub(elapsed) {
                    std::thread::sleep(remaining);
                }
                frame_start = Instant::now();
            }
        }

        update_title_fps(&mut canvas, &mut fps_frame_count, &mut fps_timer, frames_to_run);
    }

    let sav_data = gb.bus.cartridge.save_ram();
    if !sav_data.is_empty() {
        match std::fs::write(&sav_path, &sav_data) {
            Ok(()) => println!("[Save] Wrote save to {}", sav_path.display()),
            Err(e) => eprintln!("[Save] Failed to write {}: {e}", sav_path.display()),
        }
    }
}
