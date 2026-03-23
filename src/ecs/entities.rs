use std::fmt;

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct Entity {
    pub id: u32,
    pub generation: u32,
}

impl fmt::Debug for Entity {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}v{}", self.id, self.generation)
    }
}

impl Entity {
    pub fn new(id: u32, generation: u32) -> Self {
        Self { id, generation }
    }
}

pub struct EntityAllocator {
    generations: Vec<u32>,
    free_ids: Vec<u32>,
    alive: Vec<bool>,
}

impl EntityAllocator {
    pub fn new() -> Self {
        Self {
            generations: Vec::new(),
            free_ids: Vec::new(),
            alive: Vec::new(),
        }
    }

    pub fn allocate(&mut self) -> Entity {
        if let Some(id) = self.free_ids.pop() {
            self.alive[id as usize] = true;
            Entity::new(id, self.generations[id as usize])
        } else {
            let id = self.generations.len() as u32;
            self.generations.push(0);
            self.alive.push(true);
            Entity::new(id, 0)
        }
    }

    pub fn free(&mut self, entity: Entity) -> bool {
        let idx = entity.id as usize;

        self.alive[idx] = false;
        self.generations[idx] += 1;
        self.free_ids.push(entity.id);
        true
    }

    pub fn is_alive(&self, entity: Entity) -> bool {
        let idx = entity.id as usize;
        idx < self.generations.len()
            && self.generations[idx] == entity.generation
            && self.alive[idx]
    }
}
