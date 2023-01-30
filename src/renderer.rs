extern crate sdl2;

use sdl2::EventPump;
use sdl2::pixels::Color;
use sdl2::rect::Point;
use sdl2::render::Canvas;
use sdl2::video::Window;
use sdl2::event::Event;
use sdl2::keyboard::*;
use std::collections::HashMap;

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
    pixel_buffer: [u8; DISPLAY_WIDTH as usize * DISPLAY_HEIGHT as usize],
    display: Option<Canvas<Window>>,
    event_pump: Option<EventPump>,
    keys_pressed: Vec<Scancode>,
    key_to_scancode_table: HashMap<u8, Scancode>
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
    }

    pub fn step(&mut self)
    {
        self.display.as_mut().unwrap().set_draw_color(Color::BLACK);
        self.display.as_mut().unwrap().clear();

        let canvas = self.display.as_mut().unwrap();
        canvas.set_draw_color(Color::WHITE);

        let mut points: Vec<Point> = Vec::with_capacity((DISPLAY_WIDTH * DISPLAY_HEIGHT / 2) as usize);
        for (index, col) in self.pixel_buffer.iter().enumerate()
        {
            if *col > 0
            {
                let x = index % DISPLAY_WIDTH as usize;
                let y = index / DISPLAY_WIDTH as usize;
                points.push(Point::new(x as i32, y as i32));
            }
        }

        canvas.draw_points(points.as_slice()).unwrap();
        self.refresh_screen();
    }

    // returns true if user requested quit
    pub fn poll_input(&mut self) -> bool
    {
        self.keys_pressed.clear();
        for event in self.event_pump.as_mut().unwrap().poll_iter() {
            match event {
                Event::Quit {..} => { return true; }
                Event::KeyDown { scancode: Some(key), .. } => {
                    if VALID_KEYS.contains(&key) {
                        self.keys_pressed.push(key);
                    }

                    if key == Scancode::Escape {
                        return true;
                    }
                },
                _ => {}
            }
        }

        return false;
    }

    pub fn draw(&mut self, x: u8, y: u8) -> bool {
        let pixel_index = ((y as u32 * DISPLAY_WIDTH) + x as u32) as usize;
        if pixel_index >= self.pixel_buffer.len() {
            return false; // TODO: not sure what to do in this case
        }

        let curr_pixel = self.pixel_buffer[pixel_index];

        self.pixel_buffer[pixel_index] = curr_pixel ^ 1;
        
        return curr_pixel > 0;
    }

    pub fn clear_screen(&mut self) {
        for i in 0..self.pixel_buffer.len() {
            self.pixel_buffer[i] = 0;
        }

        self.display.as_mut().unwrap().set_draw_color(Color::BLACK);
        self.display.as_mut().unwrap().clear();
    }

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

    pub fn is_any_key_pressed(&mut self) -> bool {
        self.keys_pressed.len() > 0
    }

    pub fn get_first_key_pressed(&mut self) -> u8 {
        if !self.is_any_key_pressed() {
            return 0;
        }

        let pressed_key = self.keys_pressed[0];
        for key in self.key_to_scancode_table.keys() {
            if *self.key_to_scancode_table.get(key).unwrap() == pressed_key {
                return *key;
            }
        }

        return 0;
    }

}

unsafe impl Send for Renderer {}

pub fn make_renderer() -> Renderer
{
    let mut r = Renderer {
        display: None,
        event_pump: None,
        pixel_buffer: [0; DISPLAY_WIDTH as usize * DISPLAY_HEIGHT as usize],
        keys_pressed: vec![],
        key_to_scancode_table: HashMap::new()
    };

    r.key_to_scancode_table.insert(0x1, Scancode::Num1);
    r.key_to_scancode_table.insert(0x2, Scancode::Num2);
    r.key_to_scancode_table.insert(0x3, Scancode::Num3);
    r.key_to_scancode_table.insert(0xC, Scancode::Num4);

    r.key_to_scancode_table.insert(0x4, Scancode::Q);
    r.key_to_scancode_table.insert(0x5, Scancode::W);
    r.key_to_scancode_table.insert(0x6, Scancode::E);
    r.key_to_scancode_table.insert(0xD, Scancode::R);

    r.key_to_scancode_table.insert(0x7, Scancode::A);
    r.key_to_scancode_table.insert(0x8, Scancode::S);
    r.key_to_scancode_table.insert(0x9, Scancode::D);
    r.key_to_scancode_table.insert(0xE, Scancode::F);

    r.key_to_scancode_table.insert(0xA, Scancode::Z);
    r.key_to_scancode_table.insert(0x0, Scancode::X);
    r.key_to_scancode_table.insert(0xB, Scancode::C);
    r.key_to_scancode_table.insert(0xF, Scancode::V);

    return r;
}
