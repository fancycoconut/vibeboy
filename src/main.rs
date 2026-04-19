use vibeboy::config::Config;
use vibeboy::gameboy::GameBoy;
use sdl2::event::Event;
use sdl2::pixels::PixelFormatEnum;
use std::time::{Duration, Instant};

const FRAME_DURATION: Duration = Duration::from_nanos(16_742_706); // ~59.73 Hz

fn main() {
    let config = Config::load();

    let scale = config.display.scale;
    let width  = 160 * scale;
    let height = 144 * scale;

    let keymap = config.keymap();
    let quit_key = config.keybindings.quit.clone();

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
                    let name = format!("{kc:?}");
                    if name == quit_key {
                        break 'running;
                    }
                    if let Some(&btn) = keymap.get(&name) {
                        gb.bus.joypad.press(btn, &mut gb.bus.interrupts);
                    }
                }
                Event::KeyUp { keycode: Some(kc), .. } => {
                    let name = format!("{kc:?}");
                    if let Some(&btn) = keymap.get(&name) {
                        gb.bus.joypad.release(btn);
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
