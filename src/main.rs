use std::{
    thread::sleep,
    time::{Duration, Instant},
};

use minifb::{Key, Window, WindowOptions};
use rand::Rng;
use tracing::{info, warn};
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
    /// Delay timer
    register_dt: u8,
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
            register_dt: 0,
        }
    }
}

fn u8_to_key(key: u8) -> Key {
    match key {
        0x01 => Key::Key1,
        0x02 => Key::Key2,
        0x03 => Key::Key3,
        0x0C => Key::Key4,

        0x04 => Key::Q,
        0x05 => Key::W,
        0x06 => Key::E,
        0x0D => Key::R,

        0x07 => Key::A,
        0x08 => Key::S,
        0x09 => Key::D,
        0x0E => Key::F,

        0x0A => Key::Z,
        0x00 => Key::X,
        0x0B => Key::C,
        0x0F => Key::V,

        _ => Key::Key1,
    }
}

fn main() -> Result<(), anyhow::Error> {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();
    let mut emulator = Emulator::default();
    emulator.load_rom("Pong (1 player).ch8")?;
    emulator.init_window()?;
    let mut last_timer_update = Instant::now();
    loop {
        if last_timer_update.elapsed() >= Duration::from_micros(16667) {
            if emulator.register_dt > 0 {
                emulator.register_dt -= 1;
            }
            last_timer_update = Instant::now();
        }

        let instruction_high = emulator.ram[emulator.pc as usize];
        let instruction_low = emulator.ram[(emulator.pc + 1) as usize];
        let instruction = (instruction_high as u16) << 8 | instruction_low as u16;
        let x_nibble = instruction_high & 0x0F;
        let y_nibble = ((instruction_low as u16) & 0xF0) >> 4;
        let nnn = ((instruction_high as u16 & 0x0F) << 8) | instruction_low as u16;
        match instruction {
            0x6000..=0x6FFF => {
                emulator.registers[x_nibble as usize] = instruction_low;
                info!(
                    "Loading value {:2x} inside V{:x}",
                    instruction_low, x_nibble
                )
            }
            0xA000..=0xAFFF => {
                emulator.register_i = nnn;
                info!("Loading value {:2x} inside VI", nnn);
            }
            0xD000..=0xDFFF => {
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
            0x2000..=0x2FFF => {
                emulator.stack[emulator.sp as usize] = emulator.pc + 2;
                emulator.sp += 1;
                emulator.pc = nnn;
                info!("Calling routine at {:4x}", nnn)
            }
            0x7000..=0x7FFF => {
                emulator.registers[x_nibble as usize] =
                    emulator.registers[x_nibble as usize].wrapping_add(instruction_low);
                info!("loading value {:4x} into V{x_nibble}", instruction_low)
            }
            0x00EE => {
                emulator.sp -= 1;
                let ret = emulator.stack[emulator.sp as usize];
                emulator.pc = ret;
                info!("Returning to address {:4x}", ret);
                continue;
            }
            0xF065..=0xFF65 if instruction & 0xFF == 0x65 => {
                for i in 0..=x_nibble as usize {
                    emulator.registers[i] = emulator.ram[emulator.register_i as usize + i]
                }
                info!("Loading {x_nibble} values into registers")
            }
            0xF033..=0xFF33 if instruction & 0xFF == 0x33 => {
                let number = emulator.registers[x_nibble as usize];
                let value_unit = number % 10;
                let value_tens = (number / 10) % 10;
                let value_hundreds = (number / 100) % 10;
                emulator.ram[emulator.register_i as usize] = value_hundreds;
                emulator.ram[emulator.register_i as usize + 1] = value_tens;
                emulator.ram[emulator.register_i as usize + 2] = value_unit;
                info!("Loading into VI[0..3] values {value_hundreds}, {value_tens}, {value_unit}")
            }
            0xF029..=0xFF29 if instruction & 0xFF == 0x29 => {
                let sprite_value = emulator.registers[x_nibble as usize];
                emulator.register_i = 0x50 + (sprite_value as u16 * 5);
                info!("Loading embedded sprite number {sprite_value}")
            }

            0xF007..=0xFF07 if instruction & 0xFF == 0x07 => {
                emulator.registers[x_nibble as usize] = emulator.register_dt;
                info!("Loading dt into V{x_nibble}")
            }
            0xF015..=0xFF15 if instruction & 0xFF == 0x15 => {
                emulator.register_dt = emulator.registers[x_nibble as usize];
                info!("Loading V{x_nibble} into dt")
            }
            0x3000..=0x3FFF => {
                if emulator.registers[x_nibble as usize] == instruction_low {
                    emulator.pc += 2;
                }
                info!(
                    "Incrementing pc if V{x_nibble} ({:02x}) is equal to {:04x} ",
                    emulator.registers[x_nibble as usize], instruction_low
                )
            }
            0x1000..=0x1FFF => {
                emulator.pc = nnn;
                info!("Jumping to {:04x}", nnn);
                continue;
            }
            0xC000..=0xCFFF => {
                let random_number: u8 = rand::thread_rng().gen();
                emulator.registers[x_nibble as usize] = random_number & instruction_low;
                info!("Adding random value to V{x_nibble}");
            }
            0xE09E..=0xEF9E if instruction & 0xFF == 0x9E => {
                if let Some(window) = &emulator.window {
                    if window.is_key_down(u8_to_key(emulator.registers[x_nibble as usize])) {
                        emulator.pc += 2;
                    }
                    info!("Checking if key is down");
                    continue;
                }
            }
            0xE0A1..=0xEFA1 if instruction & 0xFF == 0xA1 => {
                if let Some(window) = &emulator.window {
                    if !window.is_key_down(u8_to_key(emulator.registers[x_nibble as usize])) {
                        emulator.pc += 2;
                    }
                    info!("Checking if key is up");
                    continue;
                }
            }
            0x8002..=0x8FF2 if instruction & 0xF == 0x2 => {
                emulator.registers[x_nibble as usize] &= emulator.registers[y_nibble as usize];
                info!("V{x_nibble} = V{x_nibble} & V{y_nibble}")
            }
            0x8004..=0x8FF4 if instruction & 0xF == 0x4 => {
                let (result, overflowed) = emulator.registers[x_nibble as usize]
                    .overflowing_add(emulator.registers[y_nibble as usize]);
                emulator.registers[0xF] = overflowed as u8;
                emulator.registers[x_nibble as usize] = result;
                info!("V{x_nibble} = V{x_nibble} + V{y_nibble} as overflow in VF")
            }
            0x8005..=0x8FF5 if instruction & 0xF == 0x5 => {
                let (result, borrowed) = emulator.registers[x_nibble as usize]
                    .overflowing_sub(emulator.registers[y_nibble as usize]);
                emulator.registers[0xF] = if borrowed { 0 } else { 1 }; 
                emulator.registers[x_nibble as usize] = result;
                info!(
                    "V{x_nibble} = V{x_nibble} - V{y_nibble}, VF = {}",
                    emulator.registers[0xF]
                );
            }
            0x8002..=0x8FF0 if instruction & 0xF == 0x0 => {
                emulator.registers[x_nibble as usize] = emulator.registers[y_nibble as usize];
                info!("V{x_nibble} = V{y_nibble}")
            }
            0x4000..=0x4FFF => {
                if emulator.registers[x_nibble as usize] != instruction_low {
                    emulator.pc += 2;
                }
                info!("SE v{x_nibble} {instruction_low}")
            }
            _ => {
                println!(
                    "Instruction {:02x}{:02x} not implemented",
                    instruction_high, instruction_low
                );
            }
        };
        sleep(Duration::from_millis(1));
        emulator.write_to_window()?;
        emulator.pc += 2;
    }
}
