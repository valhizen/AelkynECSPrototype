use std::any::Any;
use std::{any::TypeId, collections::HashMap};

pub struct ComponentStore {
    storage: HashMap<TypeId, Box<dyn Any>>,
}

impl ComponentStore {
    pub fn new() -> Self {
        Self {
            storage: HashMap::new(),
        }
    }

    pub fn insert<T: 'static>(&mut self, id: u32, component: T) {
        // get the TypeId
        let type_id = TypeId::of::<T>();

        // get existing map, or insert a new empty one

        let storage = self
            .storage
            .entry(type_id)
            .or_insert_with(|| Box::new(HashMap::<u32, T>::new()));

        // downcast Box<dyn Any> to &mut HashMap<u32, T>

        let map = storage
            .downcast_mut::<HashMap<u32, T>>()
            .expect("type mismatch in component storage");

        map.insert(id, component);
    }

    //looks up by TypeId, downcasts back to &T
    pub fn get<T: 'static>(&self, id: u32) -> Option<&T> {
        let type_id = TypeId::of::<T>();
        let boxed = self.storage.get(&type_id)?;
        let map = boxed.downcast_ref::<HashMap<u32, T>>()?;
        map.get(&id)
    }

    pub fn get_mut<T: 'static>(&mut self, id: u32) -> Option<&mut T> {
        let type_id = TypeId::of::<T>();
        let boxed = self.storage.get_mut(&type_id)?;
        let map = boxed.downcast_mut::<HashMap<u32, T>>()?;
        map.get_mut(&id)
    }
}
