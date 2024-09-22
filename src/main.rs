use std::{
    thread::sleep,
    time::{Duration, Instant},
};

use minifb::{Window, WindowOptions};
use tracing::info;
const WIDTH: usize = 64;
const HEIGHT: usize = 32;

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
    stack: [u16; 16],
    /// Minibuf window
    window: Option<Window>,
    /// Display
    display: [u8; (WIDTH / 8) * HEIGHT],
}

/// The `Sprite` struct represent a sprite
struct Sprite {
    x: u8,
    y: u8,
    height: u8,
    width: u8,
    content: Vec<u8>,
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
            0xF0, 0x80, 0xF0, 0x10, 0xF0, // F
        ];

        self.ram[0x050..0x050 + font_sprites.len()].copy_from_slice(&font_sprites);
    }

    fn init_window(&mut self) -> Result<(), anyhow::Error> {
        self.window = Some(Window::new(
            "Chip-8 emulator",
            WIDTH,
            HEIGHT,
            WindowOptions {
                resize: true,
                ..WindowOptions::default()
            },
        )?);
        Ok(())
    }
    fn convert_display_to_buffer(&self) -> Vec<u32> {
        let mut buffer: Vec<u32> = Vec::with_capacity(WIDTH * HEIGHT);

        for row in self.display.iter() {
            for bit_index in 0..8 {
                let first_bit: bool = ((row >> (7 - bit_index)) & 0x1) > 0;
                let color = if first_bit { 0xFFFF } else { 0x0000 };
                buffer.push(color);
            }
        }
        buffer
    }

    fn write_to_window(&mut self) -> Result<(), anyhow::Error> {
        let buffer = self.convert_display_to_buffer();
        if let Some(window) = &mut self.window {
            window.update_with_buffer(&buffer, WIDTH, HEIGHT)?
        }
        Ok(())
    }

    fn load_sprite(&mut self, sprite: Sprite) {
        self.registers[0xF] = 0;

        for y_offset in 0..sprite.height {
            let content_byte: u8 = sprite.content[y_offset as usize];
            let y = (sprite.y + y_offset) as usize % HEIGHT;

            for x_offset in 0..sprite.width {
                let x = (sprite.x + x_offset) as usize % WIDTH;

                let byte_index = (x / 8) + y * (WIDTH / 8);
                let bit_position = 7 - (x % 8);

                let display_byte = self.display[byte_index];
                let display_pixel = (display_byte >> bit_position) & 0x1;

                let sprite_pixel = (content_byte >> (7 - x_offset)) & 0x1;

                if display_pixel == 1 && sprite_pixel == 1 {
                    self.registers[0xF] = 1;
                }

                self.display[byte_index] ^= sprite_pixel << bit_position;
            }
        }
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
            window: None,
            display: [0x0; WIDTH / 8 * HEIGHT],
        }
    }
}

fn main() -> Result<(), anyhow::Error> {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();
    let mut emulator = Emulator::default();
    emulator.load_rom("Pong (1 player).ch8")?;
    emulator.init_window()?;
    let frame_duration = Duration::from_millis(1000 / 60);
    loop {
        let start_time = Instant::now();

        let instruction_high = emulator.ram[emulator.pc as usize];
        let instruction_low = emulator.ram[(emulator.pc + 1) as usize];
        info!(
            "Instruction {:02x} {:02x}",
            instruction_high, instruction_low
        );
        match (instruction_high, instruction_low) {
            (0x60..=0x6F, _) => {
                let nibble = instruction_high & 0x0F;
                emulator.registers[nibble as usize] = instruction_low;
                info!(
                    "Loading value {:2x} inside register {:x}",
                    instruction_low, nibble
                )
            }
            (0xA0..=0xAF, _) => {
                let value = ((instruction_high as u16 & 0x0F) << 8) | instruction_low as u16;
                emulator.register_i = value;
                info!("Loading value {:2x} inside register I", value);
            }
            (0xD0..=0xDF, _) => {
                let x_registry = instruction_high & 0x0F;
                let y_registry = instruction_low >> 4;
                let sprite_height = instruction_low & 0x0F;
                let x_pos = emulator.registers[x_registry as usize];
                let y_pos = emulator.registers[y_registry as usize];
                let sprite_content = emulator.ram[emulator.register_i as usize
                    ..(emulator.register_i as usize + sprite_height as usize)]
                    .to_vec();

                emulator.load_sprite(Sprite {
                    x: x_pos,
                    y: y_pos,
                    width: 8,
                    height: sprite_height,
                    content: sprite_content,
                });
                info!("Loading sprite in pos {x_pos},{y_pos} of height {sprite_height}");
            }
            (0x20..=0x2F, _) => {
                let value = ((instruction_high as u16 & 0x0F) << 8) | instruction_low as u16;
                emulator.stack[emulator.sp as usize] = emulator.pc + 2;
                emulator.sp += 1;
                emulator.pc = value;
                info!("Calling routine at {:4x}", value)
            }
            (0x70..=0x7F, _) => {
                let nibble = instruction_high & 0x0F;
                emulator.registers[nibble as usize] += instruction_low;
                info!(
                    "loading valuet {:4x} into register {nibble}",
                    instruction_low
                )
            }
            (0x00, 0xEE) => {
                emulator.sp -= 1;
                let ret = emulator.stack[emulator.sp as usize];
                emulator.pc = ret;
                info!("Returning to address {:4x}", ret)
            }
            (0xF0..=0xFF, 0x65) => {
                let x = (instruction_high & 0x0F) as usize;
                for i in 0..=x {
                    emulator.registers[i] = emulator.ram[emulator.register_i as usize + i]
                }
                info!("Loading {x} values into registers")
            }
            (0xF0..=0xFF, 0x33) => {
                let nibble = instruction_high & 0x0F;
                let number = emulator.registers[nibble as usize];
                let value_unit = number % 10;
                let value_tens = (number % 10) / 10;
                let value_hundreds = (number / 100) % 10;
                emulator.ram[emulator.register_i as usize] = value_hundreds;
                emulator.ram[emulator.register_i as usize + 1] = value_tens;
                emulator.ram[emulator.register_i as usize + 2] = value_unit;
                info!("Loading into register_i[0..3] values {value_hundreds}, {value_tens}, {value_unit}")
            }
            (0xF0..=0xFF, 0x29) => {
                let x = (instruction_high & 0x0F) as usize;
                let sprite_value = emulator.registers[x];
                emulator.register_i = 0x50 + (sprite_value as u16 * 5);
                info!("Loading embedded sprite number {sprite_value}")
            }
            _ => {
                println!(
                    "Instruction {:02x}{:02x} not implemented",
                    instruction_high, instruction_low
                );
            }
        };
        emulator.write_to_window()?;
        emulator.pc += 2;
        let elapsed_time = start_time.elapsed();
        if frame_duration > elapsed_time {
            sleep(frame_duration - elapsed_time);
        }
    }
}
