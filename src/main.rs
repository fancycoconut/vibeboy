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
use std::time::{Duration, Instant};

const FRAME_DURATION: Duration = Duration::from_nanos(16_742_706); // ~59.73 Hz

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

fn main() {
    let config = Config::load();

    let scale  = config.display.scale;
    let width  = 160 * scale;
    let height = 144 * scale;

    let keymap  = build_keymap(&config.keybindings);
    let quit_kc = resolve_quit_key(&config.keybindings.quit);

    let args: Vec<String> = std::env::args().collect();
    let rom_path = args.get(1).map(String::as_str).unwrap_or("red.gb");

    let rom = std::fs::read(rom_path).unwrap_or_else(|e| {
        eprintln!("Failed to read ROM '{}': {e}", rom_path);
        std::process::exit(1);
    });

    let mut gb = GameBoy::new(rom.clone());

    // For DMG ROMs, optionally apply GBC-style colorization.
    if !gb.bus.ppu.cgb_mode && config.display.dmg_mode != "grey" {
        let title_bytes = rom.get(0x0134..=0x0143).unwrap_or(&[]);
        let palette = dmg_palette::resolve(&config.display.dmg_palette, title_bytes);
        gb.bus.ppu.apply_dmg_compat(&palette);
        println!("[DMG] colour palette: {}", &config.display.dmg_palette);
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
        .present_vsync()
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
    let mut frame_start = Instant::now();

    'running: loop {
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit { .. } => break 'running,
                Event::KeyDown { keycode: Some(kc), .. } => {
                    if kc == quit_kc {
                        break 'running;
                    }
                    if let Some(&button) = keymap.get(&kc) {
                        gb.bus.joypad.press(button, &mut gb.bus.interrupts);
                    }
                }
                Event::KeyUp { keycode: Some(kc), .. } => {
                    if let Some(&button) = keymap.get(&kc) {
                        gb.bus.joypad.release(button);
                    }
                }
                _ => {}
            }
        }

        let framebuffer = gb.run_frame();

        texture
            .with_lock(None, |dst, _pitch| {
                dst.copy_from_slice(framebuffer.as_slice());
            })
            .expect("Texture lock failed");

        // Queue audio samples — cap queue size (~1 s) to avoid latency buildup
        let samples = gb.bus.apu.drain_samples();
        if audio_queue.size() < SAMPLE_RATE * 2 * 4 {
            audio_queue.queue_audio(&samples).ok();
        }

        canvas.clear();
        canvas.copy(&texture, None, None).expect("Texture copy failed");
        canvas.present();

        let elapsed = frame_start.elapsed();
        if elapsed < FRAME_DURATION {
            std::thread::sleep(FRAME_DURATION - elapsed);
        }
        frame_start = Instant::now();
    }
}
