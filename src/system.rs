
use crate::entity::Entity;
use crate::component::Component;

pub struct System<T: Component> {
    components: Vec<T>,
    active: Vec<Entity>,
    update_actor: Box<dyn FnMut(&Entity, T) -> T>,
}

#[allow(dead_code)]
impl<T: Component> System<T> {

    pub fn new<F: FnMut(&Entity, T) -> T + 'static> (size: usize, update_actor: F) -> Self {
        System {
            components: vec![T::blank(); size],
            active: Vec::with_capacity(size),
            update_actor: Box::new(update_actor),
        }
    }

    pub fn component (&self, e: &Entity) -> T {
        self.components[e.index()].clone()
    }

    pub fn activate (&mut self, e: &Entity) {
        if !self.active.contains(&e) {
            self.active.push(e.clone());
        }
    }

    pub fn deactivate (&mut self, e: &Entity) {
        if let Some(index) = self.active.iter().position(|x| x == e) {
            self.active.remove(index);
        }
    }

    pub fn update_entity (&mut self, e: &Entity) {
        let c = self.component(e);
        self.components[e.index()] = (self.update_actor)(&e, c);
    }

    pub fn update_active_entities (&mut self) {
        for e in self.active.clone() {
            self.update_entity(&e);
        }
    }
}