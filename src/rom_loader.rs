#![allow(dead_code, unused_variables)]
#![allow(unused_assignments)]
use crate::cpu::*;

pub fn load_test_prog(cpu: &mut CPU)
{
    unsafe {
        let mut i = PROG_MEM_START_OFFSET as usize;

        // V0 = 2
        RAM[i] = 0x60;
        RAM[i + 1] = 0x02;
        i += 2;

        // V1 += 1
        RAM[i] = 0x71;
        RAM[i + 1] = 0x01;
        i += 2;

        // skip next if V0 is pressed (EX9E)
        RAM[i] = 0xE0;
        RAM[i + 1] = 0x9E;
        i += 2;

        // jump to 0x202 (V1 += 1) - 1NNN
        RAM[i] = 0x12;
        RAM[i + 1] = 0x00;
        i += 2;
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
