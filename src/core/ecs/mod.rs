use std::any::Any;
use std::pin::Pin;
use std::sync::Arc;
use std::cell::UnsafeCell;

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
    pub components: Vec<Arc<UnsafeCell<ComponentStruct>>>,
}

impl Entity {
    pub fn new(id: u32, name: String) -> Entity {
        Entity {
            id,
            name,
            components: Vec::new(),
        }
    }

    pub fn add_component<'a, T: Component + 'a>(&mut self, game_state: &mut GameState, component: T, component_type: ComponentType) {
        let rc = Arc::new(UnsafeCell::new(ComponentStruct {
            component: Box::new(component),
            owner: self.id,
            component_type,
        }));

        self.components.push(rc.clone());
        game_state.components[component_type].push(rc);
    }

    pub async fn get_component<'a, T: Component + 'a>(&'a self, component_type: ComponentType) -> Option<&'a T> {
        for component in &self.components {
            if unsafe { &*component.get() }.component_type == component_type {
                return unsafe { Some((&(&*component.get()).component as &dyn Any).downcast_ref/*_unchecked()*/().unwrap()) };
            }
        }
        None
    }

    pub async fn get_component_mut<'a, T: Component + 'a>(&'a mut self, component_type: ComponentType) -> Option<&'a mut T> {
        for component in &self.components {
            if unsafe { &*component.get() }.component_type == component_type {
                return unsafe { Some((&mut (&mut *component.get()).component as &mut dyn Any).downcast_mut/*(_unchecked()*/().unwrap()) };
            }
        }
        None
    }
}

pub struct ComponentStruct {
    pub component: Box<dyn Component>,
    pub owner: u32,
    pub component_type: ComponentType,
}
pub type ComponentType = usize;

pub trait Component: Any {
    fn as_any(&self) -> &dyn Any;
}

macro_rules! impl_component {
    ($type:ty, $comp_type:expr) => {
        impl Component for $type {
            fn as_any(&self) -> &dyn Any {
                self
            }
        }

        impl $type {
            pub const fn get_component_type() -> ComponentType {
                $comp_type
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
    pub system: Box<dyn for<'a> Fn(&'a mut GameState, f64, f64) -> Pin<Box<dyn futures::Future<Output = ()> + 'a>> + Send + Sync>,
}

macro_rules! force_boxed {
    ($f:ident) => {
        Box::new(move |game_state, t, dt| Box::pin($f(game_state, t, dt)))
    };
}
pub(crate) use force_boxed;