
use super::{ NameTag, ComponentRefs, BitFlags };


pub trait Factory<'a, T: Default> {
    fn make_spawn(&mut self, tools: &mut BuildTools<T>);
}


pub struct BuildTools<'a, T: Default> {
    object: &'a mut T,
    component_refs: &'a ComponentRefs,
    entity: &'a mut BitFlags,
}

impl<'a, T: Default> BuildTools<'a, T> {

    pub fn new(
        object: &'a mut T,
        component_refs: &'a ComponentRefs,
        entity: &'a mut BitFlags,

    ) -> Self {
        BuildTools { object, component_refs, entity }
    }

    pub fn edit(&'a mut self) -> &'a mut T {
        self.object
    }

    pub fn add_component(&mut self, component_name: &str) {
        if let Some(component) = &self.component_refs.get(&NameTag::from_str(component_name)) {
            self.entity.set_bit(*component.index(), true);
        }
    }

    pub fn remove_component(&mut self, component_name: &str) {
        if let Some(component) = &self.component_refs.get(&NameTag::from_str(component_name)) {
            self.entity.set_bit(*component.index(), false);
        }
    }
}
