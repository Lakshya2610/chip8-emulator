extern crate sdl2;

use sdl2::EventPump;
use sdl2::pixels::Color;
use sdl2::rect::Point;
use sdl2::render::Canvas;
use sdl2::video::Window;
use sdl2::event::Event;
use sdl2::keyboard::*;
use std::collections::{HashMap, HashSet};

pub const DISPLAY_WIDTH: u32 = 64;
pub const DISPLAY_HEIGHT: u32 = 32;
pub const DISPLAY_REFRESH_RATE: f32 = 60.0; // Hz
const DISPLAY_SCALE: u32 = 10;

static VALID_KEYS: [Scancode; 16] = [
    Scancode::Num1, Scancode::Num2, Scancode::Num3, Scancode::Num4,
    Scancode::Q, Scancode::W, Scancode::E, Scancode::R,
    Scancode::A, Scancode::S, Scancode::D, Scancode::F,
    Scancode::Z, Scancode::X, Scancode::C, Scancode::V
];

pub struct Renderer {
    pixel_buffer: [u64; DISPLAY_HEIGHT as usize],
    display: Option<Canvas<Window>>,
    event_pump: Option<EventPump>,
    keys_pressed: HashSet<Scancode>,
    key_to_scancode_table: HashMap<u8, Scancode>,
    valid_keys_set: HashSet<Scancode>
}

impl Renderer {
    pub fn init(&mut self) {
        let sdl_context = sdl2::init().unwrap();
        let event_pump = sdl_context.event_pump().unwrap();
        let video_subsystem = sdl_context.video().unwrap();

        let window = video_subsystem
            .window("Chip8", DISPLAY_WIDTH * DISPLAY_SCALE, DISPLAY_HEIGHT * DISPLAY_SCALE)
            .position_centered()
            .build()
            .unwrap();

        let mut canvas = window.into_canvas().build().unwrap();

        canvas.set_scale(DISPLAY_SCALE as f32, DISPLAY_SCALE as f32).unwrap();
        canvas.set_draw_color(Color::RGB(0, 0, 0));
        canvas.clear();
        canvas.present();

        self.display = Some(canvas);
        self.event_pump = Some(event_pump);

        self.init_key_to_scancode_table();
        self.init_valid_keys_set();
    }

    fn init_key_to_scancode_table(&mut self)
    {
        self.key_to_scancode_table.insert(0x1, Scancode::Num1);
        self.key_to_scancode_table.insert(0x2, Scancode::Num2);
        self.key_to_scancode_table.insert(0x3, Scancode::Num3);
        self.key_to_scancode_table.insert(0xC, Scancode::Num4);

        self.key_to_scancode_table.insert(0x4, Scancode::Q);
        self.key_to_scancode_table.insert(0x5, Scancode::W);
        self.key_to_scancode_table.insert(0x6, Scancode::E);
        self.key_to_scancode_table.insert(0xD, Scancode::R);

        self.key_to_scancode_table.insert(0x7, Scancode::A);
        self.key_to_scancode_table.insert(0x8, Scancode::S);
        self.key_to_scancode_table.insert(0x9, Scancode::D);
        self.key_to_scancode_table.insert(0xE, Scancode::F);

        self.key_to_scancode_table.insert(0xA, Scancode::Z);
        self.key_to_scancode_table.insert(0x0, Scancode::X);
        self.key_to_scancode_table.insert(0xB, Scancode::C);
        self.key_to_scancode_table.insert(0xF, Scancode::V);
    }

    fn init_valid_keys_set(&mut self)
    {
        for key in VALID_KEYS {
            self.valid_keys_set.insert(key);
        }
    }

    pub fn step(&mut self)
    {
        self.display.as_mut().unwrap().set_draw_color(Color::BLACK);
        self.display.as_mut().unwrap().clear();

        let canvas = self.display.as_mut().unwrap();
        canvas.set_draw_color(Color::WHITE);

        let mut points: Vec<Point> = Vec::with_capacity((DISPLAY_WIDTH * DISPLAY_HEIGHT / 2) as usize);
        for (row, color_bitfield) in self.pixel_buffer.iter().enumerate()
        {
            if *color_bitfield == 0 {
                continue;
            }

            let mut bitfield = *color_bitfield;
            for col in 0..64 {
                if (bitfield & 1) > 0 {
                    points.push(Point::new(col, row as i32));
                }

                bitfield >>= 1;
                if bitfield == 0 {
                    break;
                }
            }
        }

        canvas.draw_points(points.as_slice()).unwrap();
        self.refresh_screen();
    }

    // returns true if user requested quit
    pub fn poll_input(&mut self) -> bool
    {
        for event in self.event_pump.as_mut().unwrap().poll_iter() {
            match event {
                Event::Quit {..} => { return true; }
                Event::KeyDown { scancode: Some(key), .. } => {
                    match key {
                        Scancode::Escape => return true,
                        _ => if self.valid_keys_set.contains(&key) {
                            self.keys_pressed.insert(key);
                        }
                    }
                },
                Event::KeyUp { scancode: Some(key), .. } => {
                    self.keys_pressed.remove(&key);
                }
                _ => {}
            }
        }

        return false;
    }

    pub fn draw(&mut self, x: u8, y: u8) -> bool {
        let (row, col) = (y as usize, x as usize);
        if row >= DISPLAY_HEIGHT as usize || col >= DISPLAY_WIDTH as usize
        {
            return false;
        }

        let curr_pixel = (self.pixel_buffer[row] >> col) & 1;

        if (curr_pixel ^ 1) > 0 {
            self.pixel_buffer[row] |= 1 << col;
        } else {
            self.pixel_buffer[row] &= !(1 << col);
        }
        
        return curr_pixel > 0;
    }

    pub fn clear_screen(&mut self) {
        for i in 0..self.pixel_buffer.len() {
            self.pixel_buffer[i] = 0;
        }

        self.display.as_mut().unwrap().set_draw_color(Color::BLACK);
        self.display.as_mut().unwrap().clear();
    }

    #[inline(always)]
    pub fn refresh_screen(&mut self) {
        self.display.as_mut().unwrap().present();
    }

    pub fn is_key_pressed(&mut self, key: u8) -> bool {
        let target_key = self.key_to_scancode_table.get(&key).unwrap();
        for pressed_key in self.keys_pressed.iter() {
            if *pressed_key == *target_key {
                return true;
            }
        }

        return false;
    }

    #[inline(always)]
    pub fn is_any_key_pressed(&mut self) -> bool {
        self.keys_pressed.len() > 0
    }

    pub fn get_first_key_pressed(&mut self) -> u8 {
        if !self.is_any_key_pressed() {
            return 0;
        }

        for el in self.keys_pressed.iter() {
            for key in self.key_to_scancode_table.keys() {
                if *self.key_to_scancode_table.get(key).unwrap() == *el {
                    return *key;
                }
            }
        }

        return 0;
    }

}

pub fn make_renderer() -> Renderer
{
    Renderer {
        display: None,
        event_pump: None,
        pixel_buffer: [0; DISPLAY_HEIGHT as usize],
        keys_pressed: HashSet::with_capacity(4),
        key_to_scancode_table: HashMap::new(),
        valid_keys_set: HashSet::new()
    }
}
