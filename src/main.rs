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
        self.load_font_sprites();
        for (i, &byte) in rom_data.iter().enumerate() {
            self.ram[0x200 + i] = byte;
        }
        Ok(())
    }
    /// Load the default sprites in memory starting at address 0x50
    fn load_font_sprites(&mut self) {
    let font_sprites: [u8; 80] = [
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
        0xF0, 0x80, 0xF0, 0x10, 0xF0  // F
    ];

    self.ram[0x050..0x050 + font_sprites.len()].copy_from_slice(&font_sprites);
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
        let instruction_high = emulator.ram[emulator.pc as usize];
        let instruction_low = emulator.ram[(emulator.pc + 1) as usize];
        match instruction_high {
            0x60..=0x6F => {
                let nibble = instruction_high & 0x0F;
                emulator.registers[nibble as usize] = instruction_low;
            }
            0xA0..=0xAF => {
                let value = ((instruction_low & 0x0F) as u16) << 8 | instruction_high as u16;
                emulator.register_i = value;
            }

            _ => panic!(
                "Instruction {:02x}{:02x} not implemented",
                instruction_high, instruction_low
            ),
        };
        emulator.pc += 2;
    }
}
