use std::any::Any;
use std::{any::TypeId, collections::HashMap};

trait ComponentMap {
    fn remove(&mut self, id: u32);
    fn as_any(&self) -> &dyn Any;
    fn as_any_mut(&mut self) -> &mut dyn Any;
}

impl<T: 'static> ComponentMap for HashMap<u32, T> {
    fn remove(&mut self, id: u32) {
        self.remove(&id);
    }
    fn as_any(&self) -> &dyn Any {
        self
    }
    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}

pub struct ComponentStore {
    storage: HashMap<TypeId, Box<dyn ComponentMap>>,
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
            .as_any_mut()
            .downcast_mut::<HashMap<u32, T>>()
            .expect("type mismatch in component storage");

        map.insert(id, component);
    }

    //looks up by TypeId, downcasts back to &T
    pub fn get<T: 'static>(&self, id: u32) -> Option<&T> {
        let type_id = TypeId::of::<T>();
        let boxed = self.storage.get(&type_id)?;
        let map = boxed.as_any().downcast_ref::<HashMap<u32, T>>()?;
        map.get(&id)
    }

    pub fn get_mut<T: 'static>(&mut self, id: u32) -> Option<&mut T> {
        let type_id = TypeId::of::<T>();
        let boxed = self.storage.get_mut(&type_id)?;
        let map = boxed.as_any_mut().downcast_mut::<HashMap<u32, T>>()?;
        map.get_mut(&id)
    }

    pub fn iter<T: 'static>(&self) -> impl Iterator<Item = (u32, &T)> {
        let type_id = TypeId::of::<T>();

        // Try to get and downcast the inner HashMap
        let maybe_map = self
            .storage
            .get(&type_id)
            .and_then(|boxed| boxed.as_any().downcast_ref::<HashMap<u32, T>>());

        // If it exists, iterate it. If not, iterate an empty slice.
        maybe_map
            .into_iter()
            .flat_map(|map| map.iter().map(|(&id, val)| (id, val)))
    }

    pub fn remove_all(&mut self, id: u32) {
        for store in self.storage.values_mut() {
            store.remove(id);
        }
    }
}
