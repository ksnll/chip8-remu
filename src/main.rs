/// The `Emulator` struct represents a CHIP-8 emulator, containing the memory,
/// registers, stack, program counter, and other state needed to emulate a CHIP-8 system.
struct Emulator {
    /// 4KB of memory for the CHIP-8 system.
    ram: [u8; 4096],
    /// 16 general purpose registries V0 to VF
    registers: [u8; 16],
    /// Index register (I), used for memory operations.
    register_i: u16,
    /// Program counter (PC), pointing to the current instruction.
    pc: u16,
    /// Stack pointer (SP), pointing to the current level of the call stack.
    sp: u8,
    /// Call stack, used for handling subroutines.
    stack: [u8; 16],
}

impl Emulator {
    /// Loads a ROM file into memory starting at address `0x200`.
    fn load_rom(&mut self, filename: &str) -> Result<(), anyhow::Error> {
        let rom_data = std::fs::read(filename)?;
        for (i, &byte) in rom_data.iter().enumerate() {
            self.ram[0x200 + i] = byte;
        }
        Ok(())
    }
}

impl Default for Emulator {
    fn default() -> Self {
        Self {
            ram: [0; 4096],
            registers: Default::default(),
            register_i: Default::default(),
            pc: 0x200,
            sp: 0,
            stack: [0x0; 16],
        }
    }
}

fn main() -> Result<(), anyhow::Error> {
    let mut emulator = Emulator::default();
    emulator.load_rom("Pong (1 player).ch8")?;
    loop {
        let instruction: u16 = ((emulator.ram[emulator.pc as usize] as u16) << 8)
            | emulator.ram[(emulator.pc + 1) as usize] as u16;
        println!("{:04x}", instruction);
    }
}
