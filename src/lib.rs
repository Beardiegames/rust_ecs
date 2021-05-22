
mod pool;
mod systems;
mod factory;
mod types;
mod tests;

use std::fmt::Debug;
pub use pool::{ Objects, Entities };
pub use systems::{ System, Behaviour };
pub use types::{ NameTag, ComponentRefs, ComponentRef, BitFlags };
pub use factory::*;

pub type ObjectIndex = usize;
pub type ComponentIndex = usize;
pub type SystemIndex = usize;


// start by defining components
pub struct EcsBuilder {
    size: usize,
    component_refs: ComponentRefs,
}

impl EcsBuilder {

    pub fn new(size: usize) -> Self {
        EcsBuilder{ size, component_refs: ComponentRefs(Vec::new()) }
    }

    pub fn define_component(mut self, name: &str) -> Self {
        self.component_refs.0.push(ComponentRef::new(self.component_refs.0.len(), name));
        self
    }

    pub fn build_systems<T: Default>(self) -> SystemBuilder<T> {
        SystemBuilder {
            size: self.size,
            component_refs: self.component_refs,
            systems: Vec::new(),
            behaviours: Vec::new(),
        }
    }
}

// secondly define systems
pub struct SystemBuilder<T: Default> {
    size: usize,
    component_refs: ComponentRefs, 
    systems: Vec<System>, 
    behaviours: Vec<Box<dyn Behaviour<T>>>
}

impl<'a, T: Default> SystemBuilder<T> {

    pub fn define_system(mut self, behaviour: Box<dyn Behaviour<T>>) -> Self {
        let mut components = BitFlags (0);

        for s in &mut behaviour.required_components().iter() {
            if let Some(c) = self.component_refs.get(s) {
                components.set_bit(*c.index(), true)
            }
        }
        self.behaviours.push(behaviour);
        self.systems.push(System::new(self.systems.len(), components));
        self
    }

    pub fn setup_factories(self) -> FactoryBuilder<'a, T> {
        FactoryBuilder { 
            size: self.size,
            systems: self.systems,
            behaviours: self.behaviours,
            component_refs: self.component_refs,
            factories: Vec::new()
        }
    }
}

// secondly define systems
pub struct FactoryBuilder<'a, T: Default> {
    size: usize,
    component_refs: ComponentRefs, 
    systems: Vec<System>, 
    behaviours: Vec<Box<dyn Behaviour<T>>>,
    factories: Vec<(String, Box<dyn Factory<'a, T>>)>,
}


impl<'a, T: Default + Debug> FactoryBuilder<'a, T> {

    pub fn define_factory(mut self, type_name: &str, spawn_factory: Box<dyn Factory<'a, T>>) -> Self {
        self.factories.push((type_name.to_string(), spawn_factory));
        self
    }

    pub fn finalize(self) -> Ecs<'a, T> {
        Ecs { 
            size: self.size,
            objects: Objects::new(self.size),
            entities: Entities::new(self.size),
            systems: self.systems,
            behaviours: self.behaviours,
            component_refs: self.component_refs,
            factories: self.factories,
        }
    }
}


// actual core ECS system
pub struct Ecs<'a, T: Default> { 
    size: usize,
    objects: Objects<T>, // object data pool, in other words entity component data
    entities: Entities, // object component implementation flags
    systems: Vec<System>, // behaviour wrappers for executing custom behaviour scripts
    behaviours: Vec<Box<dyn Behaviour<T>>>,
    component_refs: ComponentRefs, // component definitions, flag position & amount of components available
    factories: Vec<(String, Box<dyn Factory<'a, T>>)>, // used for spawning predefined objects
}

impl<'a, T: Default + Debug> Ecs<'a, T> {

    pub fn start(&mut self) {
        // update routine
        for system in &mut self.systems {
            self.behaviours[system.index].on_startup(&mut self.objects, system);
        }
        // handle requests
        for system in &mut self.systems {
            self.behaviours[system.index].on_early_update(&mut self.objects, system);

            if system.destroy_requests.len() > 0 || system.spawn_requests.len() > 0 {
                system.handle_requests(&mut self.objects, &mut self.entities, &mut self.factories, &self.component_refs);
            }
        }
    }

    pub fn update(&mut self) {
        // update routine
        for system in &mut self.systems {
            self.behaviours[system.index].on_early_update(&mut self.objects, system);

            for pointer in &self.entities.active {
                if system.components.0 == 
                    self.entities.pool[*pointer].0 & system.components.0 
                {
                    self.behaviours[system.index].on_update(&pointer, &mut self.objects, system);
                }
            }
        }
        // handle requests
        for system in &mut self.systems {
            if system.destroy_requests.len() > 0 || system.spawn_requests.len() > 0 {
                system.handle_requests(&mut self.objects, &mut self.entities, &mut self.factories, &self.component_refs);
            }
        }
    }

    pub fn open_update<F>(&mut self, mut update_methode: F )
    where F: FnMut(&usize, &mut Vec<T>) {
        for pointer in &self.entities.active {
            update_methode(pointer, &mut self.objects.pool);
        }
    }

    pub fn components(&self) -> &ComponentRefs {
        &self.component_refs
    }

    // pub fn add_component(&mut self, target: &usize, component: &ComponentIndex) {
    //     self.entities.pool[*target].set_bit(*component, true);
    // }

    // pub fn remove_component(&mut self, target: &usize, component: &ComponentIndex) {
    //     self.entities.pool[*target].set_bit(*component, false);
    // }

    pub fn spawn(&mut self, obj_name: &str, type_of: &str) -> Option<ObjectIndex> {
        create_object(
            NameTag::from_str(obj_name), 
            type_of,
            &mut self.entities,
            &mut self.objects,
            &mut self.factories,
            &self.component_refs,
        )
    }

    pub fn destroy(&mut self, target: &ObjectIndex) {
        destroy_object(
            target,
            &mut self.entities,
            &mut self.objects,
        );
    }

    pub fn get_mut(&mut self, target: &ObjectIndex) -> &mut T {
        self.objects.get_mut(target)
    }

    pub fn get_ref(&mut self, target: &ObjectIndex) -> &T {
        self.objects.get_ref(target)
    }

    pub fn find(&self, name: &str) -> Option<ObjectIndex> {
        self.objects.find(name)
    }
}

// DRY implementations 

fn create_object<T: Default> (
    obj_name: NameTag, 
    type_name: &str,

    entities: &mut Entities,
    objects: &mut Objects<T>,
    factories: &mut Vec<(String, Box<dyn Factory<T>>)>,
    component_refs: &ComponentRefs,

) -> Option<ObjectIndex> {

    if let Some(pointer) = entities.free.pop() {

        entities.active.push(pointer);
        objects.active.push((pointer, obj_name.clone()));
        entities.pool[pointer].reset();

        // for comp in components {
        //     if let Some(c) = component_refs.get(&comp) {
        //         entities.pool[pointer].set_bit(*c.index(), true)
        //     }
        // }
        // return Some(pointer);

        if let Some(factory) = factories.iter_mut().find(|f| f.0 == *type_name) {
            let mut build_tools = BuildTools::new( 
                &mut objects.pool[pointer],
                &component_refs,
                &mut entities.pool[pointer]
            );
            factory.1.make_spawn(&mut build_tools);
        }
    }
    None
}

fn destroy_object<T: Default> (
    target: &ObjectIndex,

    entities: &mut Entities,
    objects: &mut Objects<T>,
) {
    if let Some(i) = entities.active.iter().position(|pointer| pointer == target) {
        entities.active.remove(i);
        objects.active.remove(i);
        entities.free.push(i);
    }
}
