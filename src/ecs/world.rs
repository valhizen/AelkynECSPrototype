use std::collections::HashMap;

use super::components::health::Health;
use super::components::tag::Tag;

pub struct World {
    pub health: HashMap<u32, Health>,
    pub tag: HashMap<u32, Tag>,
    pub next_id: u32,
}

impl World {
    pub fn new() -> Self {
        Self {
            health: HashMap::new(),
            tag: HashMap::new(),
            next_id: 0,
        }
    }

    pub fn spawn(&mut self) -> u32 {
        let id = self.next_id;
        self.next_id += 1;
        id
    }

    pub fn insert_health(&mut self, id: u32, health: Health) {
        self.health.insert(id, health);
    }

    pub fn insert_tag(&mut self, id: u32, tag: Tag) {
        self.tag.insert(id, tag);
    }

    pub fn deal_damage(&mut self, id: u32, amount: u32) {
        if let Some(health) = self.health.get_mut(&id) {
            health.take_damage(amount);
        }
    }

    pub fn print_value(&self) {
        for (id, h) in &self.health {
            if let Some(n) = self.tag.get(id) {
                println!("{} has health {} / {}", n.tag, h.current, h.max);
            }
        }
    }
}
