
use super::*;
use super::pool::*;

pub struct System {
    pub(crate) index: SystemIndex,
    pub(crate) spawn_requests: Vec<(NameTag, String)>,
    pub(crate) destroy_requests: Vec<ObjectIndex>,
    pub(crate) components: BitFlags,
}

impl System {

    pub(crate) fn new(index: SystemIndex, components: BitFlags) -> Self {
        
        System {
            index,
            spawn_requests: Vec::new(),
            destroy_requests: Vec::new(),
            components,
        }
    }

    pub(crate) fn handle_requests<'a, T: Default> (
        &mut self, 
        objects: &mut Objects<T>,
        entities: &mut Entities,
        factories: &mut Vec<(String, Box<dyn Factory<'a, T>>)>,
        component_refs: &ComponentRefs,
    ) {
        // destroy requests
        while self.destroy_requests.len() > 0 {
            if let Some(target) = self.destroy_requests.pop() {
                super::destroy_object(
                    &target,           
                    entities,
                    objects,
                )
            }
        }
        // spawn requests
        while self.spawn_requests.len() > 0 {
            if let Some(spawn) = self.spawn_requests.pop() {
                super::create_object(
                    spawn.0,
                    &spawn.1,
                    entities,
                    objects,
                    factories,
                    &component_refs,
                );
            }
        }
        
    }

    // pub(crate) fn add_component(&mut self, component: ComponentRef) {
    //     self.components.set_bit(*component.index(), true)
    // }

    pub fn spawn(&mut self, new_name: &str, type_of: &str) {
        self.spawn_requests.push((NameTag::from_str(new_name), type_of.to_string()));
    }

    pub fn destroy(&mut self, target: &ObjectIndex) {
        self.destroy_requests.push(target.clone());
    }
}

pub trait Behaviour<T: Default> {
    fn required_components(&self) -> Vec<NameTag>;

    #[allow(unused_variables)]
    fn on_start(&mut self, objects: &mut Objects<T>, system: &mut System) {}

    #[allow(unused_variables)]
    fn on_update(&mut self, target: &ObjectIndex, objects: &mut Objects<T>, system: &mut System);
}
