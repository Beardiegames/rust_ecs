
mod entity;
use entity::{ Entity, Entities };

mod component;
use component::Component;

mod system;
use system::System;


pub struct ECS {
    entity_stack_size: usize,
    entities: Entities,
}

#[allow(dead_code)]
impl ECS {
    pub fn new (entity_stack_size: usize) -> Self {
        ECS { 
            entity_stack_size, 
            entities: Entities::new(entity_stack_size),
        }
    }

    pub fn system_build<C, F> (&self, update_actor: F) -> System<C>
        where
            C: Component,
            F: FnMut(&Entity, C) -> C + 'static
    {
        System::<C>::new(self.entity_stack_size, update_actor)
    }

    pub fn entity_spawn (&mut self, name: &str) -> Option<Entity> {
        self.entities.spawn(name)
    }

    pub fn entity_destroy (&mut self, e: &Entity) {
        self.entities.destroy(e)
    }

    pub fn entity_by_name (&self, name: &str) -> Option<Entity> {
        self.entities.find(|_i, n| n == name)
    }

    pub fn entity_by_index (&self, index: usize) -> Option<Entity> {
        self.entities.find(|i, _n| *i == index)
    }
}


#[cfg(test)]
mod tests {

    use super::*;

    #[derive(Clone)]
    struct Position (f32, f32);

    impl Component for Position {
        fn blank() -> Self { Position(0.0, 0.0) }
    }

    impl ToString for Position {
        fn to_string(&self) -> String {
            format!("Position({}, {})", self.0, self.1)
        }
    }

    fn move_updater (e: &Entity, mut c: Position) -> Position {
        c.0 += 1.0; 
        println!("updated '{}' to {}", e.name(), c.to_string());
        c 
    }

    #[test]
    fn usecase() {
        let mut ecs = ECS::new(100);
        let mut move_system = ecs.system_build(move_updater);

        let entity1 = ecs.entity_spawn("tester").unwrap();
        let entity2 = ecs.entity_spawn("test entity").unwrap();
        let entity3 = ecs.entity_spawn("entity").unwrap();
        move_system.activate(&entity1);
        move_system.activate(&entity2);
        move_system.activate(&entity3);

        let mut j = 0;
        for i in 1..10 {
            
            if i % 2 == 0 { move_system.activate(&entity1); j += 1; }
            else { move_system.deactivate(&entity1); }

            move_system.update_active_entities();

            let component1 = move_system.component(&entity1);
            let component2 = move_system.component(&entity2);
            let component3 = move_system.component(&entity3);
            assert_eq!(j as f32, component1.0);
            assert_eq!(i as f32, component2.0);
            assert_eq!(i as f32, component3.0);
        }
    }
}