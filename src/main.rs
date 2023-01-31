mod cpu;
mod renderer;
mod rom_loader;
use cpu::*;
use renderer::*;
use std::time::Duration;

const CLOCK_SPEED: u32 = 700; // hz

fn main() {
    let mut cpu: CPU = make_cpu();
    let mut renderer: Renderer = make_renderer();

    cpu.init();
    renderer.init();

    // rom_loader::load_test_prog(&mut cpu);
    // rom_loader::echo_prog(&mut cpu);
    rom_loader::load_prog(&mut cpu, "/Users/lakshya/Desktop/emulators/chip8/ROMs/Chip8 emulator Logo [Garstyciuks].ch8");
    // cpu.set_mode(CPUMode::Chip8);

    let sleep_dur = 1.0 / CLOCK_SPEED as f32;
    let sleep_dur_ms: f32 = 1000.0 / CLOCK_SPEED as f32;
    let mut errored = false;

    let timer_thread_chan = cpu::start_timer_thread();

    let mut last_display_referesh_t: f32 = 0.0;
    let display_referesh_t: f32 = 1000.0 / DISPLAY_REFRESH_RATE;
    loop {
        let should_exit = renderer.poll_input();
        if should_exit {
            timer_thread_chan.send(true).unwrap(); // signal timer thread to finish
            break;
        }

        if !errored {
            errored = !cpu.step(&mut renderer) || errored;
            // println!("{:?}", cpu);
            if errored {
                println!("CPU error or end of code");
            }
        }
        // unsafe { println!("Timers: {:?}", TIMERS); }

        if last_display_referesh_t >= display_referesh_t {
            renderer.step();
            last_display_referesh_t = 0.0;
        }

        ::std::thread::sleep(Duration::from_secs_f32(sleep_dur));
        last_display_referesh_t += sleep_dur_ms;
    }
    
}
