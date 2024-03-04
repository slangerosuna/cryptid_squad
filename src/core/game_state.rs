use crate::core::*;
use std::sync::Arc;

// increase this every time you add a new component type
const COMPONENT_TYPES: usize = 0;

pub struct GameState {
    pub entities: Vec<Arc<Entity>>,
    pub components: Vec<Vec<Arc<ComponentStruct>>>,
    pub scheduler: Scheduler,
    pub resources: Vec<Box<dyn Resource>>,

    pub next_entity_id: u32,
}

impl GameState {
    pub fn new(fixed_update_interval: f64) -> GameState {
        GameState {
            entities: Vec::new(),
            components: vec![Vec::new(); COMPONENT_TYPES],
            scheduler: Scheduler::new(fixed_update_interval),
            resources: Vec::new(),
            next_entity_id: 0,
        }
    }

    pub fn create_entity(&mut self, name: String) -> Arc<Entity> {
        let entity = Entity::new(self.next_entity_id, name);
        let rc = Arc::new(entity);

        self.entities.push(rc.clone());
        self.next_entity_id += 1;

        rc
    }

    pub fn add_resource<T: Resource + 'static>(&mut self, resource: T) {
        self.resources.push(Box::new(resource));
    }

    pub fn get_resource<T: Resource + 'static>(&self) -> Option<&T> {
        for resource in &self.resources {
            if let Some(r) = resource.as_ref().as_any().downcast_ref::<T>() {
                return Some(r);
            }
        }
        None
    }
}
