pub struct Health {
    pub current: u32,
    pub max: u32,
}

impl Health {
    pub fn new(current: u32, max: u32) -> Self {
        Self { current, max }
    }

    pub fn take_damage(&mut self, amount: u32) {
        if amount >= self.current {
            self.current = 0
        } else {
            self.current -= amount;
        }
    }
}
