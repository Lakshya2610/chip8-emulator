#![allow(non_snake_case)]
#![allow(dead_code, unused_variables)]

use bitmatch::bitmatch;
use std::sync::mpsc::{self, TryRecvError, Receiver, Sender};
use crate::renderer::*;
use crate::events::*;
use crate::save::*;

pub const RAM_SIZE: usize = 4096;
pub const PROG_MEM_START_OFFSET: u16 = 0x200; // 0x000 to 0x1ff was reserved for interpreter
pub const FONT_SPRITES_START_OFFSET: u16 = 0x050; // 0x050 to 0x9F
const MAX_STACK_SIZE: usize = 32; // original chip8 only supported 16, so this should be good enough
const OPTYPE_MASK: u16 = 0xF000;
const TIMER_CLOCK_SPEED: f32 = 60.0; // Hz
static FONT_SPRITES: [u8; 80] =
[
    0xF0, 0x90, 0x90, 0x90, 0xF0, // 0
    0x20, 0x60, 0x20, 0x20, 0x70, // 1
    0xF0, 0x10, 0xF0, 0x80, 0xF0, // 2
    0xF0, 0x10, 0xF0, 0x10, 0xF0, // 3
    0x90, 0x90, 0xF0, 0x10, 0x10, // 4
    0xF0, 0x80, 0xF0, 0x10, 0xF0, // 5
    0xF0, 0x80, 0xF0, 0x90, 0xF0, // 6
    0xF0, 0x10, 0x20, 0x40, 0x40, // 7
    0xF0, 0x90, 0xF0, 0x90, 0xF0, // 8
    0xF0, 0x90, 0xF0, 0x10, 0xF0, // 9
    0xF0, 0x90, 0xF0, 0x90, 0x90, // A
    0xE0, 0x90, 0xE0, 0x90, 0xE0, // B
    0xF0, 0x80, 0x80, 0x80, 0xF0, // C
    0xE0, 0x90, 0x90, 0x90, 0xE0, // D
    0xF0, 0x80, 0xF0, 0x80, 0xF0, // E
    0xF0, 0x80, 0xF0, 0x80, 0x80  // F
];

pub static mut RAM: [u8; RAM_SIZE] = [0; RAM_SIZE];
pub static mut TIMERS: [u8; 2] = [0; 2];

#[derive(Debug, PartialEq)]
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

enum RegisterRWMode
{
    Read = 0,
    Write
}

#[derive(Debug)]
pub struct CPU {
    pc: u16,
    I: u16,              // index register - points to loc in mem
    registers: [u8; 16], // general purpose registers [V0, V1, ...VF]
    stack: [u16; MAX_STACK_SIZE],
    sp: u8,              // stack pointer
    mode: CPUMode,
}

impl CPU {

    pub fn init(&mut self) {
        self.I = 0;
        self.pc = 0;
        self.registers = [0; 16];
        self.stack = [0; MAX_STACK_SIZE];
        self.sp = 0;

        // load font sprites
        unsafe {
            let mut write_ptr = FONT_SPRITES_START_OFFSET as usize;
            for el in FONT_SPRITES {
                RAM[write_ptr] = el;
                write_ptr += 1;
            }
        }
    }

    pub fn save_state(&mut self, save: &mut Save)
    {
        save.write((self.pc & 0xFF) as u8);
        save.write((self.pc >> 8) as u8);
        save.write((self.I & 0xFF) as u8);
        save.write((self.I >> 8) as u8);
        for reg in self.registers {
            save.write(reg);
        }

        for stack_val in self.stack {
            save.write((stack_val & 0xFF) as u8);
            save.write((stack_val >> 8) as u8);
        }

        save.write(self.sp);
        match self.mode {
            CPUMode::Chip8 => save.write(0),
            CPUMode::Chip48 => save.write(1)
        }

        unsafe {
            save.write(TIMERS[TimerRegs::Delay as usize]);
            save.write(TIMERS[TimerRegs::Sound as usize]);
        }
    }

    pub fn load_state(&mut self, save: &mut Save)
    {
        self.pc = save.read_u16();
        self.I = save.read_u16();
        for i in 0..self.registers.len() {
            self.registers[i] = save.read();
        }

        for i in 0..self.stack.len() {
            self.stack[i] = save.read_u16();
        }

        self.sp = save.read();
        match save.read() {
            0 => self.mode = CPUMode::Chip8,
            1 => self.mode = CPUMode::Chip48,
            _ => panic!("Invalid CPU mode in save file")
        }

        unsafe {
            TIMERS[TimerRegs::Delay as usize] = save.read();
            TIMERS[TimerRegs::Sound as usize] = save.read();
        }
    }

    pub fn set_prog_counter(&mut self, pc: u16)
    {
        self.pc = pc;
    }

    pub fn prog_counter(&self) -> u16 {
        self.pc
    }

    pub fn set_mode(&mut self, mode: CPUMode)
    {
        self.mode = mode;
    }

    fn register_rw(&mut self, last_reg_num: usize, mode: RegisterRWMode) -> bool
    {
        let mut mem_ptr = self.I as usize;
        if mem_ptr + last_reg_num >= RAM_SIZE {
            return false; // overflowing mem
        }

        for reg in 0..(last_reg_num + 1) {
            unsafe {
                match mode {
                    RegisterRWMode::Write => RAM[mem_ptr] = self.registers[reg],
                    RegisterRWMode::Read => self.registers[reg] = RAM[mem_ptr]
                }
            }

            mem_ptr += 1;
        }

        if self.mode == CPUMode::Chip8 {
            self.I = mem_ptr as u16;
        }

        return true;
    }

    fn stack_push(&mut self, n: u16)
    {
        if self.sp >= MAX_STACK_SIZE as u8 {
            panic!("Stack overflow: tried to push when stack was already full");
        }
        
        self.stack[self.sp as usize] = n;
        self.sp += 1;
    }

    fn stack_pop(&mut self) -> u16
    {
        if self.sp == 0 {
            panic!("Tried to pop from stack when it's already empty");
        }

        self.sp -= 1;
        self.stack[self.sp as usize]
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
                    self.stack_push(self.pc);
                    self.pc = n - 2; // -2 since pc is incr at the end
                },
                "0000_0000_1110_1110" => { // return (00EE)
                    self.pc = self.stack_pop();
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
                "1111_xxxx_iiii_iiii" => { // timers, add to index, get key
                    match i {
                        0x07 => self.registers[x as usize] = TIMERS[TimerRegs::Delay as usize],
                        0x15 => TIMERS[TimerRegs::Delay as usize] = self.registers[x as usize],
                        0x18 => TIMERS[TimerRegs::Sound as usize] = self.registers[x as usize],
                        0x1E => {
                            let overflow: bool;
                            (self.I, overflow) = self.I.overflowing_add(self.registers[x as usize] as u16);
                            if overflow || self.I >= RAM_SIZE as u16 {
                                self.registers[0xF] = 1;
                            }
                        },
                        0x0A => if renderer.is_any_key_pressed() {
                            // TODO: support key released as well (original behaviour on COSMAC VIP)
                            self.registers[x as usize] = renderer.get_first_key_pressed();
                        } else {
                            self.pc -= 2;
                        },
                        0x29 => self.I = FONT_SPRITES_START_OFFSET + ((self.registers[x as usize] as u16 & 0x0F) * 5),
                        0x33 => if !self.write_decimal_at_I(x as usize) { return false; },
                        0x55 => if !self.register_rw(x as usize, RegisterRWMode::Write) { return false; },
                        0x65 => if !self.register_rw(x as usize, RegisterRWMode::Read) { return false; },
                        _ => return false
                    }
                },
                _ => return false,
            }

            self.pc += 2;
            return true;
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

    fn write_decimal_at_I(&mut self, reg: usize) -> bool
    {
        if self.I + 2 >= RAM_SIZE as u16
        {
            return false;
        }

        let mut intval = self.registers[reg];
        unsafe {
            RAM[self.I as usize + 2] = intval % 10;
            intval /= 10;
            RAM[self.I as usize + 1] = intval % 10;
            intval /= 10;
            RAM[self.I as usize] = intval % 10;
        }

        return true;
    }

}

unsafe fn timer_ticker(rx: Receiver<SystemEvent>)
{
    let mut paused = false;
    loop {
        match rx.try_recv() {
            Ok(event) => {
                println!("TimerThread: recv event {:?}", event);
                match event {
                    SystemEvent::Pause => paused = true,
                    SystemEvent::Resume => paused = false,
                    SystemEvent::Exit => break,
                    SystemEvent::Save => {},
                    _ => {}
                }
            }
            Err(TryRecvError::Disconnected) => {
                println!("TimerThread: comms channel disconnected, exiting");
                break;
            }
            Err(TryRecvError::Empty) => {}
        }

        if !paused && TIMERS[TimerRegs::Delay as usize] > 0 {
            TIMERS[TimerRegs::Delay as usize] -= 1;
        }

        if !paused && TIMERS[TimerRegs::Sound as usize] > 0 {
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
        stack: [0; MAX_STACK_SIZE],
        mode: CPUMode::Chip8
    }
}

pub fn start_timer_thread() -> Sender<SystemEvent>
{
    let (tx, rx) = mpsc::channel::<SystemEvent>();
    std::thread::spawn(|| unsafe { timer_ticker(rx); });
    return tx;
}
