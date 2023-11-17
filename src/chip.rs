use rand;
use rand::Rng;
use fonts::FONT_SET;

use CHIP8_WIDTH;
use CHIP8_HEIGHT;
use CHIP8_MEM;
use ROM_SIZE;
use OPCODE_SIZE;
use FRAME_TIME;

#[derive (Debug)]
pub enum Error {
    InvalidOperation(u8, u8),
    RomTooLarge(usize),
    PcOutOfBounds(u16),
    Debug,
}

pub enum ProgramCounter {
    Next,
    Skip,
    Jump(usize),
}

impl ProgramCounter {

}

pub struct Chip {
    memory : [u8; CHIP8_MEM],                       // Memory
    v : [u8; 16],                                   // 16 8-bit registers
    i : u16,                                        // 16-bit index
    pc : u16,                                       // 16-bit program counter
    stack : [u16; 0x10],                            // 16 level 16-bit stack
    sp : u8,                                        // 8-bit stack pointer
    dt : u8,                                        // 8-bit delay timer
    st : u8,                                        // 8-bit sound timer
    input_wait : bool,                              // Waits for a keypad input 
    input_keys : [bool; 16],                        // 16 input keys
    input_register : u16,                           // Registers keypad inputs
    disp : [u8; CHIP8_WIDTH * CHIP8_HEIGHT / 8],    // Display
    tone: bool,                                     // toggle beep
    time : isize,                                   // keypad register time
}

impl  Chip {
    pub fn new() -> Self {
        // Load the fonts into memory
        let mut mem = [0, CHIP8_MEM];
        for i in 0..FONT_SET.len() {
            mem[i] = FONT_SET[i];
        }

        Self {
            memory: mem,
            v: [0; 16],
            i: 0,
            pc : ROM_SIZE as u16,
            stack : [0; 0x10],
            sp : 0,
            dt : 0,
            st : 0,
            input_wait : false,
            input_keys : 0,
            input_register : 0,
            disp : [0; CHIP8_WIDTH * CHIP8_HEIGHT / 8],
            tone: false,
            time : 0,
        }
    }

    pub fn load_rom(&mut self, rom : &[u8]) -> Result<(), Error> {
        // Load a rom into memory
        if rom.len() > CHIP8_MEM - RAM_SIZE {
            return Err(Error::RomTooLarge(rom.len()))
        }
        self.memory[ROM_SIZE..ROM_SIZE + rom.len()].copy_from_slice(rom);
        Ok(())
    }

    pub fn tone(&self) -> bool {
        // Whether a tone should be played or not
        self.tone
    }

    pub fn disp(&self) -> [u8; CHIP8_WIDTH * CHIP8_HEIGHT / 8] {
        self.disp
    }

    pub fn frame(&mut self, input_keys : [bool; 16]) -> Result<(), Error> {
        // Executes instructions and simulates hardware for the duration of a frame
        self.input_keys = input_keys;
        if self.input_wait {
            for i in 0..input_keys.len() {
                if input_keys[i] {
                    self.input_wait = false;
                    self.v[self.input_register] = i as u8;
                    break;
                }
            }
        }
        else {
            if self.delay_timer > 0 {
                self.delay_timer -= 1
            }
            if self.sound_timer > 0 {
                self.sound_timer -= 1
            }
            let opcode = self.get_opcode();
            self.exec(opcode);
        }
        
        self.time += FRAME_TIME;

        while self.time > 0 {
            if self.pc as usize > CHIP8_MEM -1 {
                return Err(Error::PcOutOfBounds(self.pc));
            }
            let w0 = self.memory[self.pc as usize];
            let w1 = self.memory[self.pc + 1 as usize];
            let adv = self.exec(w0, w1)?;
            self.time -= adv as isize;
        }
        Ok(())
    }

    pub fn op_00e0(&mut self) -> ProgramCounter {
        // Clears the display (CLS)
        for byte in self.disp.iter_mut() {
            *byte = 0;
        }
        ProgramCounter::Next
    }

    pub fn op_00ee(&mut self) -> ProgramCounter {
        // Return from a subroutine
        self.sp -= 1;
        ProgramCounter::Jump(self.stack[self.sp])
    }

    pub fn op_1nnn(&mut self, nnn: u16) -> ProgramCounter {
        // Jump to location address nnn
        ProgramCounter::Jump(nnn)
    }

    pub fn op_2nnn(&mut self, nnn: u16) -> ProgramCounter {
        // Call operation, increments stack pointer
        // Places current PC to stop of stack
        // PC is then set to nnn
        self.stack[self.sp] = self.pc + OPCODE_SIZE;
        self.sp += 1;
        ProgramCounter::Jump(nnn)
    }

    pub fn op_3xkk(&mut self, x: u8, kk: u8) -> ProgramCounter {
        // Skips next instruction is Vx == kk
        if self.v[x] == kk {
            ProgramCounter::Skip
        }
        else{
            ProgramCounter::Next
        }
    }

    pub fn op_4xkk(&mut self, x: u8, kk: u8) -> ProgramCounter {
        // Skips next instruction if Vx != kk
        if self.v[x] != kk {
            ProgramCounter::Skip
        }
        else{
            ProgramCounter::Next
        }
    }

    pub fn op_5xy0(&mut self, x: u8, y :u8) -> ProgramCounter {
        // Skips next instruction if Vx == Vy
        if self.v[x] == self.v[y] {
            ProgramCounter::Skip
        }    
    }

    pub fn op_6xkk(&mut self, x: u8, kk: u8) -> ProgramCounter {
        // Sets Vx = kk
        self.v[x] = kk;
        ProgramCounter::Next
    }

    pub fn op_7xkk(&mut self, x: u8, kk: u8) -> ProgramCounter {
        // Sets Vx = Vx + kk
        let vx = V[x] as u16;
        let val = kk as u16;
        let result = vx + val;
        self.v[x] = result as u8;
        ProgramCounter::Next    
    }

    pub fn op_8xy0(&mut self, x: u8, y :u8) -> ProgramCounter {
        // Sets Vx = Vy
        self.v[x] = self.v[y];
        ProgramCounter::Next
    }

    pub fn op_8xy1(&mut self, x: u8, y :u8) -> ProgramCounter {
        // Sets Vx = Vx OR Vy
        self.v[x] |=  self.v[y];
        ProgramCounter::Next
    }

    pub fn op_8xy2(&mut self, x: u8, y :u8) -> ProgramCounter {
        // Sets Vx = Vx AND Vy
        self.v[x] &= self.v[y];
        ProgramCounter::Next
    }

    pub fn op_8xy3(&mut self, x: u8, y :u8) -> ProgramCounter {
        // Sets Vx = Vx XOR Vy
        self.v[x] ^= self.v[y];
        ProgramCounter::Next
    }

    pub fn op_8xy4(&mut self, x: u8, y :u8) -> ProgramCounter {
        // Set Vx = Vx + Vy, Set VF = carry
        let vx = self.v[x] as u16;
        let vy = self.v[y] as u16;
        let result = vx - vy;
        self.v[x] = result as u8;
        self.v[0x0f] = if result > 0xFF {1} else {0};
        ProgramCounter::Next 
    }

    pub fn op_8xy5(&mut self, x: u8, y :u8) -> ProgramCounter {
        // Set Vx = Vx - Vy, set VF = NOT BORROW
        self.v[0x0f] = if self.v[x] > self.v[y] {1} else {0};
        self.v[x] = self.v[x].wrapping_sub(v[y]);
        ProgramCounter::Next
    }

    pub fn op_8xy6(&mut self, x: u8) -> ProgramCounter {
        // Set Vx = Vx SHR 1
        self.v[0x0f] = self.v[x] & 1;
        self.v[x] >>= 1;
        ProgramCounter::Next
    }

    pub fn op_8xy7(&mut self, x: u8, y :u8) -> ProgramCounter {
        //
        self.v[0x0f] = if self.v[y] > self.v[x] {1} else {0};
        self.v[x] = self.v[y].wrapping_sub(v[x]);
        ProgramCounter::Next
    }

    pub fn op_8xye(&mut self, x: u8) -> ProgramCounter {
        //
        self.v[0x0f] = (self.v[x] & 0b10000000) >> 7;
        self.v[x] <<= 1;
        ProgramCounter::Next
    }

    pub fn op_9xy0(&mut self, x: u8, y: u8) -> ProgramCounter {
        //
        if self.v[x] != self.v[y] {
            ProgramCounter::Skip
        }
        else{
            ProgramCounter::Next
        }
    }

    pub fn op_annn(&mut self, nnn: u16) -> ProgramCounter {
        //
        self.i = nnn;
        ProgramCounter::Next
    }

    pub fn op_bnnn(&mut self, nnn: u16) -> ProgramCounter {
        //
        ProgramCounter::Jump((self.v[0] as u16) + nnn)
    }

    pub fn op_cxkk(&mut self, x: u8, kk: u16) -> ProgramCounter {
        //
        let mut rng = rand::thread_rng();
        self.v[x] = rng.gen::<u8>() & kk;
        ProgramCounter::Next
    }

    pub fn op_dxyn(&mut self, x: u8, y :u8, n: u8) -> ProgramCounter {
        self.v[0x0f] = 0;
        for byte in 0..(n as usize) {
            let y = (self.v[y] as usize + byte) % CHIP8_HEIGHT;
            for bit in 0..8 {
                let x = (self.v[x] as usize + bit) % CHIP8_WIDTH;
                let colour = (self.memory[self.i + byte] >> (7 - bit)) & 1;
                self.v[0x0f] |= colour & self.mem[y][x];
                self.mem[y][x] ^= colour;
            }
        }

        ProgramCounter::Next
    }

    pub fn op_ex9e(&mut self, x: u8) -> ProgramCounter {
        if self.input_keys[self.v[x]] as usize {
            ProgramCounter::Skip
        }
        else {
            ProgramCounter::Skip
        }
    }

    pub fn op_exal(&mut self, x: u8) -> ProgramCounter {
        if !self.input_keys[self.v[x]] as usize {
            ProgramCounter::Skip
        }
        else {
            ProgramCounter::Skip
        }
    }

    pub fn op_fx07(&mut self, x: u8) -> ProgramCounter {
        self.v[x] = self.dt;
        ProgramCounter::Nexto
    }

    pub fn op_fx0a(&mut self, x: u8) -> ProgramCounter {
        self.input_wait = true;
        self.input_keys = x;
        ProgramCounter::Next
    }

    pub fn op_fx18(&mut self, x: u8) -> ProgramCounter {
        self.st = self.v[x];
        ProgramCounter::Next
    }

    pub fn op_fx1e(&mut self, x: u8) -> ProgramCounter {
        self.i = self.v[x] as u8;
        self.v[0x0f] = if self.i > 0x0F00 {1} else {0};
        ProgramCounter::Next
    }

    pub fn op_fx29(&mut self, x: u8) -> ProgramCounter {
        self.i = (self.v[x] as u8) * 5;
        ProgramCounter::Next
    }

    pub fn op_fx33(&mut self, x: u8) -> ProgramCounter {
        self.mem[self.i] = self.v[x] / 100;
        self.mem[self.i + 1] = (self.v[x] % 100) / 10;
        self.mem[self.i + 2] = self.v[x] % 10;
        ProgramCounter::Next
    }

    pub fn op_fx55(&mut self, x: u8) -> ProgramCounter {
        for i in 0..x + 1 {
            self.ram[self.i + 1] = self.v[i];
        }
        ProgramCounter::Next
    }

    pub fn op_fx65(&mut self, x: u8) -> ProgramCounter {
        for i in 0..x + 1 {
            self.v[i] = self.mem[self.i + i];
        }
        ProgramCounter::Next
    }

    pub fn get_opcode(&mut self) -> u16 {
        (self.ram[self.pc] as u16) << 8 | (self.ram[self.pc + 1] as u16)
    }

    pub fn exec(&mut self, opcode: u16) -> Result<usize, Error> {
        // Execute steps given w0 and w1
        let nibbles = (
            (opcode & 0xF000) >> 12 as u8,
            (opcode & 0x0F00) >> 8 as u8,
            (opcode & 0x00F0) >> 4 as u8,
            (opcode & 0x000F) as u8,
        );

        let nnn = (opcode & 0x0FFF) as u16;
        let kk = (opcode & 0x00FF) as u8;
        let x = nibbles.1 as u8;
        let y = nibbles.2 as u8;
        let n = nibbles.3 as u8;

        let pc_change = match nibbles {
            (0x00, 0x00, 0x0e, 0x00) => self.op_00e0(),
            (0x00, 0x00, 0x0e, 0x0e) => self.op_00ee(),
            (0x01, _, _, _) => self.op_1nnn(nnn),
            (0x02, _, _, _) => self.op_2nnn(nnn),
            (0x03, _, _, _) => self.op_3xkk(x, kk),
            (0x04, _, _, _) => self.op_4xkk(x, kk),
            (0x05, _, _, 0x00) => self.op_5xy0(x, y),
            (0x06, _, _, _) => self.op_6xkk(x, kk),
            (0x07, _, _, _) => self.op_7xkk(x, kk),
            (0x08, _, _, 0x00) => self.op_8xy0(x, y),
            (0x08, _, _, 0x01) => self.op_8xy1(x, y),
            (0x08, _, _, 0x02) => self.op_8xy2(x, y),
            (0x08, _, _, 0x03) => self.op_8xy3(x, y),
            (0x08, _, _, 0x04) => self.op_8xy4(x, y),
            (0x08, _, _, 0x05) => self.op_8xy5(x, y),
            (0x08, _, _, 0x06) => self.op_8xy6(x),
            (0x08, _, _, 0x07) => self.op_8xy7(x, y),
            (0x08, _, _, 0x0e) => self.op_8xye(x),
            (0x09, _, _, 0x00) => self.op_9xy0(x, y),
            (0x0a, _, _, _) => self.op_annn(nnn),
            (0x0b, _, _, _) => self.op_bnnn(nnn),
            (0x0c, _, _, _) => self.op_cxkk(x, kk),
            (0x0d, _, _, _) => self.op_dxyn(x, y, n),
            (0x0e, _, 0x09, 0x0e) => self.op_ex9e(x),
            (0x0e, _, 0x0a, 0x01) => self.op_exa1(x),
            (0x0f, _, 0x00, 0x07) => self.op_fx07(x),
            (0x0f, _, 0x00, 0x0a) => self.op_fx0a(x),
            (0x0f, _, 0x01, 0x05) => self.op_fx15(x),
            (0x0f, _, 0x01, 0x08) => self.op_fx18(x),
            (0x0f, _, 0x01, 0x0e) => self.op_fx1e(x),
            (0x0f, _, 0x02, 0x09) => self.op_fx29(x),
            (0x0f, _, 0x03, 0x03) => self.op_fx33(x),
            (0x0f, _, 0x05, 0x05) => self.op_fx55(x),
            (0x0f, _, 0x06, 0x05) => self.op_fx65(x),
        }
    } 
}

