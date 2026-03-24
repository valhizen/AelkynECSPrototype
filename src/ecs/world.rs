use super::component_store::ComponentStore;
use super::entities::Entity;
use super::entities::EntityAllocator;
use super::resource::Resource;

pub struct World {
    allocator: EntityAllocator,
    components: ComponentStore,
    resource: Resource,
}

impl World {
    pub fn new() -> Self {
        Self {
            allocator: EntityAllocator::new(),
            components: ComponentStore::new(),
            resource: Resource::new(),
        }
    }

    pub fn spawn(&mut self) -> Entity {
        self.allocator.allocate()
    }

    pub fn despawn(&mut self, entity: Entity) -> bool {
        if self.allocator.free(entity) {
            self.components.remove_all(entity.id);
            true
        } else {
            false
        }
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

    pub fn iter<T: 'static>(&self) -> Vec<(Entity, &T)> {
        self.components
            .iter::<T>()
            .filter_map(|(id, val)| {
                let entity = self.allocator.get_entity(id)?;
                Some((entity, val))
            })
            .collect()
    }

    pub fn get_by_id<T: 'static>(&self, id: u32) -> Option<&T> {
        self.components.get(id)
    }

    pub fn insert_resource<T: 'static>(&mut self, value: T) {
        self.resource.insert(value);
    }

    pub fn get_resource<T: 'static>(&self) -> Option<&T> {
        self.resource.get()
    }

    pub fn get_resource_mut<T: 'static>(&mut self) -> Option<&mut T> {
        self.resource.get_mut()
    }

    pub fn query2<A: 'static, B: 'static>(&self) -> Vec<(Entity, &A, &B)> {
        self.iter::<A>()
            .into_iter()
            .filter_map(|(entity, a)| {
                let b = self.get::<B>(entity)?;
                Some((entity, a, b))
            })
            .collect()
    }

    pub fn query3<A: 'static, B: 'static, C: 'static>(&self) -> Vec<(Entity, &A, &B, &C)> {
        self.iter::<A>()
            .into_iter()
            .filter_map(|(entity, a)| {
                let b = self.get::<B>(entity)?;
                let c = self.get::<C>(entity)?;
                Some((entity, a, b, c))
            })
            .collect()
    }
}
