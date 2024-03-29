use gb_core::cpu::Cpu;
use sdl2::event::Event;
use sdl2::pixels::Color;
use sdl2::rect::Rect;
use sdl2::render::Canvas;
use sdl2::video::Window;
use std::env;
use std::fs::File;
use std::io::Read;
use sdl2::keyboard::Keycode;

const SCALE: u32 = 2;
const SCREEN_WIDTH: usize = 160;
const SCREEN_HEIGHT: usize = 144;
const WINDOW_WIDTH: u32 = (SCREEN_WIDTH as u32) * SCALE;
const WINDOW_HEIGHT: u32 = (SCREEN_HEIGHT as u32) * SCALE;

// Gets input rom path and starts main loop
fn main() {
    let mut args: Vec<_> = env::args().collect();
    if args.len() != 2 {
        args = vec![String::from(""), String::from("./tetris.gb")];
    }

    let mut gb = Cpu::new();
    let mut rom = File::open(&args[1]).expect("Unable to open file");
    let mut buffer = Vec::new();
    rom.read_to_end(&mut buffer).unwrap();
    gb.load(&buffer);

    let sdl_context = sdl2::init().unwrap();
    let video_subsystem = sdl_context.video().unwrap();
    let window = video_subsystem
        .window("Gebb", WINDOW_WIDTH, WINDOW_HEIGHT)
        .position_centered()
        .opengl()
        .build()
        .unwrap();
    let mut canvas = window.into_canvas().present_vsync().build().unwrap();
    canvas.clear();
    canvas.present();
    let mut event_pump = sdl_context.event_pump().unwrap();

    'gameloop: loop {
        for evt in event_pump.poll_iter() {
            match evt {
                Event::Quit { .. } | Event::KeyDown{keycode: Some(Keycode::Escape), ..} => {
                    break 'gameloop;
                },
                Event::KeyDown{keycode: Some(key), ..} => {
                    if let Some((key, select)) = key_code((key)) {
                        gb.mmu.joypad.select = select;
                        gb.mmu.joypad.up(key);
                    }
                },
                Event::KeyUp{keycode: Some(key), ..} => {
                    if let Some((key, select)) = key_code((key)) {
                        gb.mmu.joypad.select = select;
                        gb.mmu.joypad.down(key);
                    }
                },
                _ => (),
            }
        }

        gb.do_cycle();
        // TODO: Run renderer on seperate thread.
        if gb.ppu_updated() {
            draw_screen(&gb, &mut canvas)
        }
    }
}

fn key_code(key: Keycode) -> Option<(u8, bool)> {
    match key {
        Keycode::Right => Some((0, false)),
        Keycode::Left => Some((1, false)),
        Keycode::Up => Some((2, false)),
        Keycode::Down => Some((3, false)),

        Keycode::C => Some((0, true)), // This is actually A (on the gameboy), but the C key is a nicer keybind for a keyboard.
        Keycode::X => Some((1, true)), // Actually B
        Keycode::Q => Some((2, true)), // Actually Start
        Keycode::W => Some((3, true)), // Actually Select
        _ => {None}
    }
}

fn draw_screen(emu: &Cpu, canvas: &mut Canvas<Window>) {
    // Clear canvas as black
    canvas.set_draw_color(Color::RGB(0, 0, 0));
    canvas.clear();
    let screen_buf = emu.get_display();
    // Now set draw color to white, iterate through each point and see if it should be drawn
    canvas.set_draw_color(Color::RGB(255, 255, 255));
    for i in 0..(screen_buf.iter().len() / 3) {
        let (r, g, b) = (screen_buf[i * 3], screen_buf[(i * 3) + 1], screen_buf[(i * 3) + 2]);
        canvas.set_draw_color(Color::RGB(r, g, b));

        let x = (i % SCREEN_WIDTH) as u32;
        let y = (i / SCREEN_WIDTH) as u32;
        // Draw a rectangle at (x,y), scaled up by our SCALE value
        let rect = Rect::new((x * SCALE) as i32, (y * SCALE) as i32, SCALE, SCALE);
        canvas.fill_rect(rect).unwrap();
    }
    canvas.present();
}
