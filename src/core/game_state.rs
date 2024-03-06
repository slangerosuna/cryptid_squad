use crate::core::*;
use std::sync::Arc;
use std::cell::UnsafeCell;

// increase this every time you add a new component type
const COMPONENT_TYPES: usize = 6;

pub struct GameState {
    pub entities: Vec<Arc<UnsafeCell<Entity>>>,
    pub components: Vec<Vec<Arc<UnsafeCell<ComponentStruct>>>>,
    pub resources: Vec<Box<dyn Resource>>,

    pub next_entity_id: u32,
}

impl GameState {
    pub fn new() -> GameState {
        GameState {
            entities: Vec::new(),
            components: vec![Vec::new(); COMPONENT_TYPES],
            resources: Vec::new(),
            next_entity_id: 0,
        }
    }

    pub fn create_entity<'a>(&mut self, name: String) -> &'a mut Entity {
        let entity = Entity::new(self.next_entity_id, name);
        let rc = Arc::new(UnsafeCell::new(entity));

        self.entities.push(rc.clone());
        self.next_entity_id += 1;

        unsafe { &mut *rc.get() }
    }

    pub fn get_entity<'a>(&'a self, id: usize) -> Option<&'a mut Entity> {
        if id >= self.entities.len() {
            return None;
        }
        Some(unsafe { &mut *self.entities[id].get() })
    }

    pub fn get_entity_mut<'a>(&'a self, id: usize) -> Option<&'a mut Entity> {
        if id >= self.entities.len() {
            return None;
        }
        Some(unsafe { &mut *self.entities[id].get() })
    }

    pub fn add_resource<T: Resource>(&mut self, resource: T) {
        self.resources.push(Box::new(resource));
    }

    pub fn get_resource<'a, T: Resource>(&'a self) -> Option<&'a T> {
        for resource in &self.resources {
            if let Some(r) = resource.as_ref().as_any().downcast_ref::<T>() {
                return Some(r);
            }
        }
        None
    }

    pub fn get_resource_mut<'a, T: Resource>(&'a mut self) -> Option<&'a mut T> {
        for resource in &mut self.resources {
            if let Some(r) = (resource.as_mut() as &mut dyn std::any::Any).downcast_mut::<T>() {
                return Some(r);
            }
        }
        None
    }
}
