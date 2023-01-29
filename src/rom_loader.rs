#![allow(dead_code, unused_variables)]
#![allow(unused_assignments)]
use crate::cpu::*;

pub fn load_test_prog(cpu: &mut CPU)
{
    unsafe {
        let mut i = PROG_MEM_START_OFFSET as usize;
        // set I
        RAM[i] = 0xA2;
        RAM[i + 1] = 0xF0;
        i += 2;

        RAM[0x2F0] = 0b1010_1010;

        // clear screen
        RAM[i] = 0x00;
        RAM[i + 1] = 0xE0;
        i += 2;

        // set V0 = 55
        RAM[i] = 0x60;
        RAM[i + 1] = 0x37;
        i += 2;

        // V1 = 16
        RAM[i] = 0x61;
        RAM[i + 1] = 0x10;
        i += 2;

        // Draw at V0, V1 of len 10
        RAM[i] = 0xD0;
        RAM[i + 1] = 0x1A;
        i += 2;

        // jump to start
        // RAM[i] = 0x10;
        // RAM[i + 1] = 0x00;
        // i += 2;
    }

    cpu.set_prog_counter(PROG_MEM_START_OFFSET);
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
