use std::{any::Any, any::TypeId, collections::HashMap};

pub struct Resource {
    storage: HashMap<TypeId, Box<dyn Any>>,
}

impl Resource {
    pub fn new() -> Self {
        Self {
            storage: HashMap::new(),
        }
    }

    pub fn insert<T: 'static>(&mut self, value: T) {
        let type_id = TypeId::of::<T>();
        self.storage.insert(type_id, Box::new(value));
    }

    pub fn get<T: 'static>(&self) -> Option<&T> {
        let type_id = TypeId::of::<T>();
        self.storage.get(&type_id)?.downcast_ref::<T>()
    }

    pub fn get_mut<T: 'static>(&mut self) -> Option<&mut T> {
        let type_id = TypeId::of::<T>();
        self.storage.get_mut(&type_id)?.downcast_mut::<T>()
    }
}
