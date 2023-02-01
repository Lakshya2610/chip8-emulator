mod cpu;
mod renderer;
mod rom_loader;
use cpu::*;
use renderer::*;
use std::time::Duration;

const CLOCK_SPEED: u32 = 700; // hz

fn print_args_help() {
    println!("Run the emulator using: chip8 '<rom path>' -mode (chip8|chip48)");
    println!("\t-mode: (Optional) emulator can run in two modes, select the one that your rom was written for");
}

fn handle_args(args: &Vec<String>, cpu: &mut CPU) -> bool {
    match args.len() {
        0 | 1 => {
            println!("No rom specified, exiting");
            false
        },
        2 => {
            let prog_path = args[1].as_str();
            rom_loader::load_prog(cpu, &prog_path)
        },
        4 => {
            let prog_path = args[1].as_str();
            let rom_loaded = rom_loader::load_prog(cpu, &prog_path);
            if !rom_loaded {
                println!("Error loading rom, path may be wrong");
            }

            let mode_specified = args[2].as_str() == "-mode";

            rom_loaded && mode_specified && match args[3].as_str() {
                "chip8" => {
                    cpu.set_mode(CPUMode::Chip8);
                    true
                },
                "chip48" => {
                    cpu.set_mode(CPUMode::Chip48);
                    true
                },
                _ => false
            }
        },
        _ => false
    }
}

fn main() {
    let args: Vec<String> = std::env::args().collect();

    let mut cpu: CPU = make_cpu();
    let mut renderer: Renderer = make_renderer();

    cpu.init();
    renderer.init();

    println!();
    println!();
    if !handle_args(&args, &mut cpu) {
        print_args_help();
        return;
    }

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
            if errored {
                println!("CPU error or end of code");
            }
        }

        if last_display_referesh_t >= display_referesh_t {
            renderer.step();
            last_display_referesh_t = 0.0;
        }

        ::std::thread::sleep(Duration::from_secs_f32(sleep_dur));
        last_display_referesh_t += sleep_dur_ms;
    }
    
}
