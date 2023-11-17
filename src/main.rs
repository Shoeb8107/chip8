use rand;
use sdl2;

mod drivers;
mod chip;
mod fonts;

use std::thread;
use std::time::Duration;
use std::env;

use drivers::{AudioDriver, DisplayDriver, GameDriver, InputDriver};
use chip::Chip;

const CHIP8_WIDTH: usize = 64;
const CHIP8_HEIGHT: usize = 32;
const CHIP8_MEM: usize = 0x1000;
const ROM_SIZE : usize = 0x200;
const OPCODE_SIZE: usize = 2;
const FRAME_TIME: isize = 16666;

fn main() {
    
}
