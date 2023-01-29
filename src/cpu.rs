#![allow(non_snake_case)]
#![allow(dead_code, unused_variables)]

use bitmatch::bitmatch;
use crate::renderer::*;
pub const RAM_SIZE: usize = 4096;
pub const PROG_MEM_START_OFFSET: u16 = 0x200; // 0x000 to 0x1ff was reserved for interpreter
const MAX_STACK_SIZE: usize = 32; // original chip8 only supported 16, so this should be good enough
const OPTYPE_MASK: u16 = 0xF000;
pub static mut RAM: [u8; RAM_SIZE] = [0; RAM_SIZE];

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

struct Stack {
    data: Vec<u16>,
}

#[derive(Debug)]
pub struct CPU {
    pc: u16,
    I: u16,              // index register - points to loc in mem
    registers: [u8; 16], // general purpose registers [V0, V1, ...VF]
    sp: u8,              // stack pointer
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
                    self.pc = n;
                },
                "1101_xxxx_yyyy_nnnn" => {
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
}

pub fn make_cpu() -> CPU {
    CPU { pc: 0, I: 0, registers: [0; 16], sp: 0 }
}
