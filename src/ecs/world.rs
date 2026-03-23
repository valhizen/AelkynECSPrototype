use super::component_store::ComponentStore;
use super::components::health::Health;
use super::entities::Entity;
use super::entities::EntityAllocator;

pub struct World {
    allocator: EntityAllocator,
    components: ComponentStore,
}

impl World {
    pub fn new() -> Self {
        Self {
            allocator: EntityAllocator::new(),
            components: ComponentStore::new(),
        }
    }

    pub fn spawn(&mut self) -> Entity {
        self.allocator.allocate()
    }

    pub fn despawn(&mut self, entity: Entity) -> bool {
        self.allocator.free(entity)
    }

    pub fn insert<T: 'static>(&mut self, entity: Entity, component: T) {
        self.components.insert(entity.id, component);
    }
    pub fn get<T: 'static>(&self, entity: Entity) -> Option<&T> {
        if !self.allocator.is_alive(entity) {
            return None;
        }
        self.components.get(entity.id)
    }

    pub fn get_mut<T: 'static>(&mut self, entity: Entity) -> Option<&mut T> {
        self.components.get_mut(entity.id)
    }

    pub fn read_value(&self, entity: Entity) {
        println!("{}", entity.id);
    }

    pub fn deal_damage(&mut self, entity: Entity, amount: u32) {
        if let Some(health) = self.get_mut::<Health>(entity) {
            health.take_damage(amount);
        }
    }
}
