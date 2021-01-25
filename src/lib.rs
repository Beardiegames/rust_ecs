
mod pool;
mod systems;
mod types;
mod tests;

use std::fmt::Debug;
use pool::{ Objects, Entities };
use systems::{ System, Behaviour };
use types::{ NameTag, ComponentRefs, ComponentRef, BitFlags };


const MAX_OBJECTS: usize = 1000;

type ObjectIndex = usize;
type ComponentIndex = usize;
type SystemIndex = usize;

pub struct Ecs<T: Default> { 
    objects: Objects<T>, // object data pool, in other words entity component data
    entities: Entities, // object component implementation flags
    systems: Vec<System>, // behaviour wrappers for executing custom behaviour scripts
    behaviours: Vec<Box<dyn Behaviour<T>>>,
    component_refs: ComponentRefs, // component definitions, flag position & amount of components available
}

impl<T: Default + Debug> Ecs<T> {

    pub fn new() -> Self {
        let mut create_entities = Vec::<BitFlags>::with_capacity(MAX_OBJECTS);
        create_entities.resize_with(MAX_OBJECTS, Default::default);

        let mut free = Vec::with_capacity(MAX_OBJECTS);
        for i in 0..MAX_OBJECTS { free.push(i); }

        Ecs { 
            objects: Objects::new(),
            entities: Entities::new(),
            systems: Vec::new(),
            behaviours:Vec::new(),
            component_refs: ComponentRefs(Vec::new()),
        }
    }

    pub fn start(&mut self) {
        // update routine
        for system in &mut self.systems {
            self.behaviours[system.index].on_start(&mut self.objects, system);
        }
        // handle requests
        for system in &mut self.systems {
            if system.destroy_requests.len() > 0 || system.spawn_requests.len() > 0 {
                system.handle_requests(&mut self.objects, &mut self.entities, &self.component_refs);
            }
        }
    }

    pub fn update(&mut self) {
        // update routine
        for system in &mut self.systems {
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
                system.handle_requests(&mut self.objects, &mut self.entities, &self.component_refs);
            }
        }
    }

    pub fn define_system(&mut self, behaviour: Box<dyn Behaviour<T>>) {
        let mut components = BitFlags (0);

        for s in &mut behaviour.required_components().iter() {
            if let Some(c) = self.component_refs.get(s) {
                components.set_bit(*c.index(), true)
            }
        }
        self.behaviours.push(behaviour);
        self.systems.push(System::new(self.systems.len(), components));
    }

    // setup new components by giving them a name and a flag position
    pub fn define_component(&mut self, name: &str) {
        self.component_refs.0.push(ComponentRef::new(self.component_refs.0.len(), name));
    }

    pub fn components(&self) -> &ComponentRefs {
        &self.component_refs
    }

    // pub fn add_component(&mut self, target: &usize, component: &ComponentIndex) {
    //     self.entities[*target].set_bit(*component, true);
    // }

    // pub fn remove_component(&mut self, target: &usize, component: &ComponentIndex) {
    //     self.entities[*target].set_bit(*component, false);
    // }

    pub fn spawn(&mut self, name: &NameTag, components: Vec<NameTag>) {
        create_object(
            name,
            components,
            &mut self.objects.active,
            &mut self.entities,
            &mut self.component_refs,
        )
    }

    pub fn destroy(&mut self, target: &ObjectIndex) {
        destroy_object(
            target, 
            &mut self.objects.active,
            &mut self.entities
        )
    }
}

// DRY METHODES

fn create_object(
    name: &NameTag, 
    components: Vec<NameTag>, 
    objects_active: &mut Vec<(ObjectIndex, NameTag)>,
    entities: &mut Entities,
    component_refs: &ComponentRefs,
) {
    if let Some(pointer) = entities.free.pop() {

        entities.active.push(pointer);
        objects_active.push((pointer, name.clone()));
        entities.pool[pointer].reset();

        for comp in components {
            if let Some(c) = component_refs.get(&comp) {
                entities.pool[pointer].set_bit(*c.index(), true)
            }
        }
    }
}

fn destroy_object(
    target: &ObjectIndex, 
    objects_active: &mut Vec<(ObjectIndex, NameTag)>,
    entities: &mut Entities,
) {
    if let Some(i) = entities.active.iter().position(|pointer| pointer == target) {
        entities.active.remove(i);
        objects_active.remove(i);
        entities.free.push(i);
    }
}