use std::fs;
use crate::cpu::*;
use crate::renderer::*;

const SAVE_BUFFER_SIZE: usize = 8192;

pub struct Save {
    buffer: [u8; SAVE_BUFFER_SIZE],
    write_ptr: usize,
    read_ptr: usize
}

impl Save {

    pub fn write(&mut self, byte: u8) {
        self.buffer[self.write_ptr] = byte;
        self.write_ptr += 1;
    }

    pub fn build(&mut self, cpu: &mut CPU, renderer: &mut Renderer) {
        unsafe {
            for byte in RAM {
                self.write(byte);
            }
        }
        
        cpu.save_state(self);
        renderer.save_state(self);
    }

    pub fn write_to_disk(&mut self) {
        fs::write("save.c8s", self.buffer).unwrap();
    }

    // Load funcs

    pub fn read(&mut self) -> u8 {
        if self.read_ptr >= SAVE_BUFFER_SIZE {
            panic!("Error reading save, tried to read but save buffer is already exhausted");
        }

        let byte = self.buffer[self.read_ptr];
        self.read_ptr += 1;
        return byte;
    }

    pub fn read_u16(&mut self) -> u16 {
        (self.read() as u16) | ((self.read() as u16) << 8)
    }

    pub fn load(&mut self, savefile: &str, cpu: &mut CPU, renderer: &mut Renderer) -> bool {
        let data = std::fs::read(savefile).expect("Failed to read save becuase file couldn't be found");
        assert!( data.len() <= SAVE_BUFFER_SIZE );
        for (index, byte) in data.iter().enumerate() {
            self.buffer[index] = *byte;
        }

        unsafe {
            for i in 0..RAM_SIZE {
                RAM[i] = self.read();
            }
        }

        cpu.load_state(self);
        renderer.load_state(self);
        return true;
    }
}

pub fn make_save() -> Save {
    Save { buffer: [0; SAVE_BUFFER_SIZE], write_ptr: 0, read_ptr: 0 }
}
