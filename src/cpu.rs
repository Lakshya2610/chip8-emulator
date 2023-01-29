#![allow(non_snake_case)]
#![allow(dead_code, unused_variables)]

use bitmatch::bitmatch;
use std::sync::mpsc::{self, TryRecvError, Receiver, Sender};
use crate::renderer::*;
pub const RAM_SIZE: usize = 4096;
pub const PROG_MEM_START_OFFSET: u16 = 0x200; // 0x000 to 0x1ff was reserved for interpreter
const MAX_STACK_SIZE: usize = 32; // original chip8 only supported 16, so this should be good enough
const OPTYPE_MASK: u16 = 0xF000;
const TIMER_CLOCK_SPEED: f32 = 60.0; // Hz

pub static mut RAM: [u8; RAM_SIZE] = [0; RAM_SIZE];
static mut TIMERS: [u8; 2] = [0; 2];

#[derive(Debug)]
#[derive(PartialEq)]
// needed due to both ver having diff impl for some instr
pub enum CPUMode
{
    Chip8,
    Chip48
}

enum Instructions
{
    CallMachineCode(u16), // 0NNN
    ClearScreen, // 00E0
    Return, // 00EE
    Jump(u16), // 1NNN
    SetVX(u16), // 6XNN
    AddVX(u16), // 7XNN
    SetIReg(u16), // ANNN
    Draw(u16, u16, u16) // DXYN
}

enum TimerRegs
{
    Delay = 0,
    Sound
}

#[derive(Debug)]
struct Stack {
    data: Vec<u16>,
}

#[derive(Debug)]
pub struct CPU {
    pc: u16,
    I: u16,              // index register - points to loc in mem
    registers: [u8; 16], // general purpose registers [V0, V1, ...VF]
    sp: u8,              // stack pointer
    stack: Stack,
    mode: CPUMode,
}

impl Stack {
    pub fn push(&mut self, n: u16) {
        if self.data.len() >= MAX_STACK_SIZE {
            format!(
                "Stack Overflow - exceeded stack size limit of {} when pushing value {}",
                MAX_STACK_SIZE, n
            );
            panic!();
        }
        self.data.push(n);
    }

    pub fn pop(&mut self) -> u16 {
        self.data.pop().unwrap()
    }
}

impl CPU {

    pub fn init(&mut self) {
        self.I = 0;
        self.pc = 0;
        self.registers = [0; 16];
        self.sp = 0;
    }

    pub fn set_prog_counter(&mut self, pc: u16)
    {
        self.pc = pc;
    }

    pub fn set_mode(&mut self, mode: CPUMode)
    {
        self.mode = mode;
    }

    // returns true if there was an error
    #[bitmatch]
    pub fn step(&mut self, renderer: &mut Renderer) -> bool {
        unsafe {
            // fetch
            let opcode: u16 =
                ((RAM[self.pc as usize] as u16) << 8) | (RAM[(self.pc + 1) as usize] as u16);

            // decode & exec
            #[bitmatch]
            match opcode {
                "1010_nnnn_nnnn_nnnn" => {
                    self.I = n;
                },
                "0110_xxxx_nnnn_nnnn" => { // set Vx = n (6XNN)
                    assert!(x <= 0xF);
                    self.registers[x as usize] = n as u8;
                },
                "0111_xxxx_nnnn_nnnn" => { // set Vx = n (6XNN)
                    assert!(x <= 0xF);
                    self.registers[x as usize] = self.registers[x as usize].wrapping_add(n as u8);
                },
                "0001_nnnn_nnnn_nnnn" => { // jump
                    self.pc = n - 2;
                },
                "1011_nnnn_nnnn_nnnn" => { // BNNN jump (or BXNN)
                    let mut jump_reg: usize = 0;
                    if self.mode == CPUMode::Chip48 {
                        jump_reg = ((n >> 8) & 0x0F) as usize;
                    }

                    self.pc = n + (self.registers[jump_reg] as u16) - 2;
                },
                "0010_nnnn_nnnn_nnnn" => { // 2NNN => call subroutine
                    self.stack.push(self.pc);
                    self.sp = self.stack.data.len() as u8;
                    self.pc = n - 2; // -2 since pc is incr at the end
                },
                "0000_0000_1110_1110" => { // return (00EE)
                    self.pc = self.stack.pop();
                    self.sp -= 1;
                },
                "iiii_xxxx_nnnn_nnnn" if i == 3 || i == 4 => { // 3XNN or 4XNN
                    let vx = self.registers[x as usize];
                    if (i == 3 && vx == n as u8) || (i == 4 && vx != n as u8)
                    {
                        self.pc += 2;
                    }
                },
                "iiii_xxxx_yyyy_0000" if i == 5 || i == 9 => { // 5XY0 or 9XY0
                    let vx = self.registers[x as usize];
                    let vy = self.registers[y as usize];
                    if (i == 5 && vx == vy) || (i == 9 && vx != vy)
                    {
                        self.pc += 2;
                    }
                },
                "1110_xxxx_iiii_iiii" if i == 0x9E || i == 0xA1 => { // skip if pressed
                    let vx = self.registers[x as usize];
                    assert!(vx <= 0xF);

                    let pressed = renderer.is_key_pressed(vx);
                    if (i == 0x9E && pressed) || (i == 0xA1 && !pressed) {
                        self.pc += 2;
                    }
                },
                "1000_xxxx_yyyy_nnnn" => { // 8XYN (math ops)
                    if !self.handle_math_ops(x as usize, y as usize, n as u8) {
                        return false;
                    }
                },
                "1100_xxxx_nnnn_nnnn" => { // CXNN (rand)
                    self.registers[x as usize] = rand::random::<u8>() & n as u8;
                },
                "1101_xxxx_yyyy_nnnn" => { // DXYN (draw)
                    self.handle_draw(renderer, Instructions::Draw(x, y, n))
                },
                "0000_0000_1110_0000" => {
                    renderer.clear_screen();
                },
                _ => return true,
            }

            self.pc += 2;
            return false;
        }
    }

    fn handle_draw(&mut self, renderer: &mut Renderer, instr: Instructions) {
        match instr {
            Instructions::Draw(x, y, n) => {
                let sX = self.registers[x as usize] & ((DISPLAY_WIDTH - 1) as u8);
                let sY = self.registers[y as usize] & ((DISPLAY_HEIGHT - 1) as u8);
                self.registers[0xF] = 0; // VF = 0

                let mut row = 0;
                while row < n && row < DISPLAY_HEIGHT as u16 {
                    let mut drawX = sX;
                    unsafe {
                        let mut sprite = RAM[(self.I + row) as usize];
                        while sprite > 0 && drawX < DISPLAY_WIDTH as u8 {
                            let pixel = (sprite & 0x80) > 0;
                            if pixel {
                                let pixel_unset = renderer.draw(drawX, sY + (row as u8));
                                if pixel_unset {
                                    self.registers[0xF] = 1; // VF = 1
                                }
                            }

                            sprite = sprite << 1;
                            drawX += 1;
                        }
                    }
                    
                    row += 1;
                }
            }
            _ => panic!("handle_draw: called with invalid instr\n")
        }
    } 

    fn handle_math_ops(&mut self, regX: usize, regY: usize, op: u8) -> bool
    {
        assert!(regX <= 0xF);
        assert!(regY <= 0xF);

        let mut vx = self.registers[regX];
        let vy = self.registers[regY];
        let overflow: bool;
        match op {
            0 => { self.registers[regX] = vy; },
            1 => { self.registers[regX] = vx | vy; },
            2 => { self.registers[regX] = vx & vy; },
            3 => { self.registers[regX] = vx ^ vy; },
            4 => {
                (self.registers[regX], overflow) = vx.overflowing_add(vy);
                self.registers[0xF] = overflow as u8;
            },
            5 => {
                (self.registers[regX], overflow) = vx.overflowing_sub(vy);
                self.registers[0xF] = !overflow as u8;
            },
            7 => {
                (self.registers[regX], overflow) = vy.overflowing_sub(vx);
                self.registers[0xF] = !overflow as u8;
            },
            6 | 0xE => {
                if self.mode == CPUMode::Chip8 {
                    vx = vy;
                }

                let shifted_bit: u8;
                if op == 6 {
                    shifted_bit = vx & 1;
                    vx >>= 1;
                } else {
                    shifted_bit = vx & 0x80;
                    vx <<= 1;
                }

                self.registers[regX] = vx;
                self.registers[0xF] = (shifted_bit > 0) as u8;
            },
            _ => { return false; }
        }

        return true;
    }
}

unsafe fn timer_ticker(rx: Receiver<bool>)
{
    loop {
        match rx.try_recv() {
            Ok(_) | Err(TryRecvError::Disconnected) => {
                println!("TimerThread: recv terminate signal, exiting");
                break;
            }
            Err(TryRecvError::Empty) => {}
        }

        if TIMERS[TimerRegs::Delay as usize] > 0 {
            TIMERS[TimerRegs::Delay as usize] -= 1;
        }

        if TIMERS[TimerRegs::Sound as usize] > 0 {
            TIMERS[TimerRegs::Sound as usize] -= 1;
        }

        std::thread::sleep(std::time::Duration::from_secs_f32( 1.0 / TIMER_CLOCK_SPEED ));
    }
}

pub fn make_cpu() -> CPU {
    CPU {
        pc: 0,
        I: 0,
        registers: [0; 16],
        sp: 0,
        stack:
        Stack { data: Vec::with_capacity(16) },
        mode: CPUMode::Chip8
    }
}

pub fn start_timer_thread() -> Sender<bool>
{
    let (tx, rx) = mpsc::channel();
    std::thread::spawn(|| unsafe { timer_ticker(rx); });
    return tx;
}
