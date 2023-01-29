extern crate sdl2;

use sdl2::EventPump;
use sdl2::pixels::Color;
use sdl2::rect::Point;
use sdl2::render::Canvas;
use sdl2::video::Window;
use sdl2::event::Event;
use sdl2::keyboard::Keycode;

pub const DISPLAY_WIDTH: u32 = 64;
pub const DISPLAY_HEIGHT: u32 = 32;
const DISPLAY_SCALE: u32 = 10;

pub struct Renderer {
    pixel_buffer: [u8; DISPLAY_WIDTH as usize * DISPLAY_HEIGHT as usize],
    display: Option<Canvas<Window>>,
    event_pump: Option<EventPump>
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

    // returns true if user requested quit
    pub fn step(&mut self) -> bool
    {
        self.clear_screen();

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
        
        for event in self.event_pump.as_mut().unwrap().poll_iter() {
            match event {
                Event::Quit {..} |
                Event::KeyDown { keycode: Some(Keycode::Escape), .. } => {
                    return true;
                },
                _ => {}
            }
        }

        return false;
    }

    pub fn draw(&mut self, x: u8, y: u8) -> bool {
        let pixel_index = ((y as u32 * DISPLAY_WIDTH) + x as u32) as usize;
        let curr_pixel = self.pixel_buffer[pixel_index];

        self.pixel_buffer[pixel_index] = curr_pixel ^ 1;
        
        return curr_pixel > 0;
    }

    pub fn clear_screen(&mut self) {
        self.display.as_mut().unwrap().set_draw_color(Color::BLACK);
        self.display.as_mut().unwrap().clear();
    }

    pub fn refresh_screen(&mut self) {
        self.display.as_mut().unwrap().present();
    }

}

pub fn make_renderer() -> Renderer
{
    Renderer { display: None, event_pump: None, pixel_buffer: [0; DISPLAY_WIDTH as usize * DISPLAY_HEIGHT as usize] }
}
