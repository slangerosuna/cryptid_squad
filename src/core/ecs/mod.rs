use std::any::Any;
use std::sync::Arc;
use crate::core::*;

pub trait Resource: Any {
    fn as_any(&self) -> &dyn Any;
}

macro_rules! impl_resource {
    ($type:ty) => {
        impl Resource for $type {
            fn as_any(&self) -> &dyn Any {
                self
            }
        }
    };
}
pub(crate) use impl_resource;

pub struct Entity {
    pub id: u32,
    pub name: String,
    pub components: Vec<Arc<ComponentStruct>>,
}

impl Entity {
    pub fn new(id: u32, name: String) -> Entity {
        Entity {
            id,
            name,
            components: Vec::new(),
        }
    }

    pub fn add_component<T: Component + 'static>(&mut self, mut game_state: &mut GameState, component: T, component_type: ComponentType) {
        let rc = Arc::new(ComponentStruct {
            component: Box::new(component),
            owner: self.id,
        });

        self.components.push(rc.clone());
        game_state.components[component_type].push(rc);
    }

    pub fn get_component<T: Component + 'static>(&self) -> Option<&T> {
        for component in &self.components {
            if let Some(c) = (component.component.as_ref() as &dyn Any).downcast_ref::<T>() {
                return Some(c);
            }
        }
        None
    }
}

pub struct ComponentStruct {
    pub component: Box<dyn Component>,
    pub owner: u32,
}
pub type ComponentType = usize;

pub trait Component: Any {
    fn as_any(&self) -> &dyn Any;
}

macro_rules! impl_component {
    ($type:ty) => {
        impl Component for $type {
            fn as_any(&self) -> &dyn Any {
                self
            }
        }
    };
}
pub(crate) use impl_component;

pub enum SystemType {
    Init,
    Update,
    FixedUpdate,
}

pub struct System {
    pub args: Vec<ComponentType>,
    pub system: fn(&mut GameState),
}
