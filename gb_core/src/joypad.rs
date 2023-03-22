pub struct Joypad {
    keysA: u8,
    keysD: u8,
    pub select: bool // When false, keysD. When true, keysA
}

impl Joypad {
    pub fn new() -> Self {
        Self {
            keysA: 0xFF, // 1 Means unpressed.
            keysD: 0xFF,
            select: false,
        }
    }

    pub fn down(&mut self, key: u8) { // TODO: Do this using pointers like I originally planned (idk how though).
        // let mut orig = if self.select {&self.keysD} else {&self.keysA};
        // orig = &self.set(key, *orig);
        if self.select {
            self.keysD = self.set(key, self.keysD);
        } else {
            self.keysA = self.set(key, self.keysA);
        }
    }

    pub fn up(&mut self, key: u8) {
        if self.select {
            self.keysD = self.res(key, self.keysD);
        } else {
            self.keysA = self.res(key, self.keysA);
        }
    }

    pub fn write(&mut self, byte: u8) {
        if (byte >> 5) & 0b1 == 0 {
            self.select = false;
        } else if (byte >> 4) & 0b1 == 1 {
            self.select = true;
        }
    }

    pub fn read(&self) -> u8 {
        if self.select {
            self.keysA
        } else {
            self.keysD
        }
    }

    fn res(&self, position: u8, val: u8) -> u8 { // Set bit to 1
        val & !(1 << position) | (u8::from(0) << position)
    }

    fn set(&self, position: u8, val: u8) -> u8 { // Set bit to 0
        val & !(1 << position) | (u8::from(1) << position)
    }
}