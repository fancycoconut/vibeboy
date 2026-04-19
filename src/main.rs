mod apu;
mod bus;
mod cartridge;
mod cpu;
mod gameboy;
mod interrupts;
mod joypad;
mod ppu;
mod timer;

use gameboy::GameBoy;
use joypad::btn;
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::pixels::PixelFormatEnum;
use std::time::{Duration, Instant};

const SCALE: u32 = 3;
const WIDTH: u32 = 160 * SCALE;
const HEIGHT: u32 = 144 * SCALE;
const FRAME_DURATION: Duration = Duration::from_nanos(16_742_706); // ~59.73 Hz

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let rom_path = args.get(1).map(String::as_str).unwrap_or("red.gb");

    let rom = std::fs::read(rom_path).unwrap_or_else(|e| {
        eprintln!("Failed to read ROM '{}': {e}", rom_path);
        std::process::exit(1);
    });

    let mut gb = GameBoy::new(rom);

    // -------------------------------------------------------------------------
    // SDL2 setup
    // -------------------------------------------------------------------------
    let sdl = sdl2::init().expect("SDL2 init failed");
    let video = sdl.video().expect("SDL2 video init failed");

    let window = video
        .window("Vibeboy", WIDTH, HEIGHT)
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
        // Handle events
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit { .. } => break 'running,
                Event::KeyDown { keycode: Some(kc), .. } => {
                    if let Some(b) = keycode_to_btn(kc) {
                        gb.bus.joypad.press(b, &mut gb.bus.interrupts);
                    }
                    if kc == Keycode::Escape {
                        break 'running;
                    }
                }
                Event::KeyUp { keycode: Some(kc), .. } => {
                    if let Some(b) = keycode_to_btn(kc) {
                        gb.bus.joypad.release(b);
                    }
                }
                _ => {}
            }
        }

        // Emulate one full frame
        let framebuffer = gb.run_frame();

        // Blit framebuffer to SDL2 texture
        texture
            .with_lock(None, |dst, _pitch| {
                dst.copy_from_slice(framebuffer.as_slice());
            })
            .expect("Texture lock failed");

        canvas.clear();
        canvas
            .copy(&texture, None, None)
            .expect("Texture copy failed");
        canvas.present();

        // Pace to ~60 FPS (vsync handles it if enabled, this is a fallback)
        let elapsed = frame_start.elapsed();
        if elapsed < FRAME_DURATION {
            std::thread::sleep(FRAME_DURATION - elapsed);
        }
        frame_start = Instant::now();
    }
}

fn keycode_to_btn(kc: Keycode) -> Option<usize> {
    match kc {
        Keycode::Right  => Some(btn::RIGHT),
        Keycode::Left   => Some(btn::LEFT),
        Keycode::Up     => Some(btn::UP),
        Keycode::Down   => Some(btn::DOWN),
        Keycode::Z      => Some(btn::B),
        Keycode::X      => Some(btn::A),
        Keycode::Return | Keycode::Return2 => Some(btn::START),
        Keycode::RShift | Keycode::LShift  => Some(btn::SELECT),
        _ => None,
    }
}
