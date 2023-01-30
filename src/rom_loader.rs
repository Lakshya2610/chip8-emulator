#![allow(dead_code, unused_variables)]
#![allow(unused_assignments)]
use crate::cpu::*;

pub fn load_test_prog(cpu: &mut CPU)
{
    unsafe {
        let mut i = PROG_MEM_START_OFFSET as usize;

        // V0 = 1
        RAM[i] = 0x60;
        RAM[i + 1] = 0x01;
        i += 2;

        // V1 = 5
        RAM[i] = 0x61;
        RAM[i + 1] = 0x05;
        i += 2;

        // V2 = 8
        RAM[i] = 0x62;
        RAM[i + 1] = 0x08;
        i += 2;

        // Dump regs to mem
        RAM[i] = 0xF2;
        RAM[i + 1] = 55;
        i += 2;

        // V0 = 0
        RAM[i] = 0x60;
        RAM[i + 1] = 0x00;
        i += 2;

        // V1 = 0
        RAM[i] = 0x61;
        RAM[i + 1] = 0x00;
        i += 2;

        // V2 = 0
        RAM[i] = 0x62;
        RAM[i + 1] = 0x00;
        i += 2;

        // Load regs from mem
        RAM[i] = 0xF2;
        RAM[i + 1] = 65;
        i += 2;
    }

    cpu.set_prog_counter(PROG_MEM_START_OFFSET);
    cpu.set_mode(CPUMode::Chip48);
}

pub fn echo_prog(cpu: &mut CPU)
{
    unsafe {
        let mut i = PROG_MEM_START_OFFSET as usize;

        // save key to V0
        RAM[i] = 0xF0;
        RAM[i + 1] = 0x0A;
        i += 2;

        // clear screen
        RAM[i] = 0x00;
        RAM[i + 1] = 0xE0;
        i += 2;

        // set font sprite to pressed key (in V0)
        RAM[i] = 0xF0;
        RAM[i + 1] = 0x29;
        i += 2;

        // draw
        RAM[i] = 0xD5;
        RAM[i + 1] = 0x55;
        i += 2;

        // jump to start
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
