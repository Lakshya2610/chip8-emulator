#![allow(dead_code, unused_variables)]
#![allow(unused_assignments)]
use crate::cpu::*;

pub fn load_test_prog(cpu: &mut CPU)
{
    unsafe {
        let mut i = PROG_MEM_START_OFFSET as usize;
        // call func @ 0x2F0
        RAM[i] = 0x22;
        RAM[i + 1] = 0xF0;
        i += 2;

        // I = 255
        RAM[i] = 0xA0;
        RAM[i + 1] = 0xFF;
        i += 2;

        // V1 = 255
        RAM[i] = 0x61;
        RAM[i + 1] = 0xFF;
        i += 2;

        // V0 = V1 >> 1
        RAM[i] = 0xB1;
        RAM[i + 1] = 0x11;
        i += 2;

        // func @ 0x2F0
        // V0 = 2
        RAM[0x2F0] = 0x60;
        RAM[0x2F0 + 1] = 0x02;
        // return
        RAM[0x2F0 + 2] = 0x00;
        RAM[0x2F0 + 3] = 0xEE;
    }

    cpu.set_prog_counter(PROG_MEM_START_OFFSET);
    cpu.set_mode(CPUMode::Chip48);
}

pub fn load_prog(cpu: &mut CPU, prog_path: &str) -> bool
{
    let data = std::fs::read(prog_path).expect("Failed to read program becuase file couldn't be found");
    let mut write_ptr = PROG_MEM_START_OFFSET as usize;
    for byte in data.iter()
    {
        if write_ptr >= RAM_SIZE {
            return false;
        }

        unsafe
        {
            RAM[write_ptr] = *byte;
        }
        write_ptr += 1;
    }

    cpu.set_prog_counter(PROG_MEM_START_OFFSET);
    println!("Loaded {} of size {} bytes", prog_path, data.len());
    return true;
}
