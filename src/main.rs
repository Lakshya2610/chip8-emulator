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

    rom_loader::load_prog(&mut cpu, "/Users/lakshya/Desktop/emulators/chip8/ROMs/IBM Logo.ch8");

    let sleep_dur = 1e9 as u32 / CLOCK_SPEED;
    let mut errored = false;
    println!("Sleep duration: {} nsec", sleep_dur);
    loop {
        if !errored {
            errored = cpu.step(&mut renderer) || errored;
            // println!("{:?}", cpu);
        }
        
        let should_exit = renderer.step();
        if should_exit {
            break;
        }

        ::std::thread::sleep(Duration::new(0, sleep_dur));
    }
}
