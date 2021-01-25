use std::fmt::Debug;
use std::convert::TryInto;
use std::time::{SystemTime};

// enum Action { 
//     Spawn(NameTag), 
//     Destroy(ObjectIndex),
//     AddComponent(ObjectIndex, NameTag), 
//     RemoveComponent(ObjectIndex, NameTag),
// }

// struct Actions(Vec<Action>);

// impl Actions {
//     fn new() -> Self { Self(Vec::new()) }
    
//     pub fn spawn(&mut self, name: &str) {
//         self.0.push(Action::Spawn(NameTag::from_str(name)));
//     }

//     pub fn destroy(&mut self, target: usize) {
//         self.0.push(Action::Destroy(target));
//     }

//     pub fn add_component(&mut self, target: usize, component_name: &str) {
//         self.0.push(
//             Action::AddComponent(target, NameTag::from_str(component_name))
//         );
//     }

//     pub fn remove_component(&mut self, target: usize, component_name: &str) {
//         self.0.push(
//             Action::RemoveComponent(target, NameTag::from_str(component_name))
//         );
//     }

//     pub fn clear(&mut self) {
//         self.0.clear();
//     }

//     pub fn list(&self) -> &Vec<Action> {
//         &self.0
//     }
// }

const MAX_OBJECTS: usize = 10000;

type Objects<T> = [T; MAX_OBJECTS];
type Entities = [BitFlags; MAX_OBJECTS];
type ObjectIndex = usize;
type ComponentIndex = usize;

struct Ecs<T: Default> { 
    objects: Objects<T>, // object data pool, in other words entity component data
    entities: Entities, // object component implementation flags
    systems: Vec<Box<dyn System<T>>>, // behaviour wrappers for executing custom behaviour scripts
    components: Vec<Component>, // component definitions, flag position & amount of components available
    //actions: Actions, // perform actions that can only be handled after the update sequence
    active: Vec<ObjectIndex>,
    free: Vec<ObjectIndex>,
}

impl<T: Default + Debug> Ecs<T> {

    fn new() -> Self {
        let mut create_objects = Vec::<T>::with_capacity(MAX_OBJECTS);
        create_objects.resize_with(MAX_OBJECTS, Default::default);

        let mut create_entities = Vec::<BitFlags>::with_capacity(MAX_OBJECTS);
        create_entities.resize_with(MAX_OBJECTS, Default::default);

        let mut free = Vec::with_capacity(MAX_OBJECTS);
        for i in 0..MAX_OBJECTS { free.push(i); }

        Ecs { 
            objects: create_objects.try_into().unwrap(),
            entities: create_entities.try_into().unwrap(),
            systems: Vec::new(),
            components: Vec::new(),
           // actions: Actions::new(),
            active: Vec::new(),
            free,
        }
    }

    fn update(&mut self) {
        // update routine
        for system in &mut self.systems {
            for pointer in &self.active {
                if self.entities[*pointer].0 == 
                    self.entities[*pointer].0 & system.requires_components().0 
                {
                    system.update(&pointer, &mut self.objects, &self.active);//, &mut self.actions);
                }
            }
        }

        // handle actions
        // for action in self.actions.list() {
        //     match action {
        //         Action::Spawn(s) => {
        //             if self.objects.spawn().is_some() {
        //                 if let Some(pointer) = self.entities.spawn() {
        //                     self.entities.edit(&pointer).rename(s);
        //                 }
        //             }
        //         }, 
        //         Action::Destroy(p) => { 
        //             self.objects.destroy(p);
        //             self.entities.destroy(p);
        //         }, 
        //         Action::AddComponent(p, s) => {
        //             if let Some(component) = self.components.get(s) {
        //                 self.entities.edit(p).add_component(component.clone());
        //             }
        //         }, 
        //         Action::RemoveComponent(p, s) => {
        //             if let Some(component) = self.components.get(s) {
        //                 self.entities.edit(p).remove_component(component.clone());
        //             }
        //         }, 
        //     }
        // }
        // self.actions.clear();
    }

    pub fn define_system(&mut self, name: &str, system: Box<dyn System<T>>) {
        let index = self.systems.len();
        self.systems.push(system);
        self.systems[index].init(&self.components);
    }

    // setup new components by giving them a name and a flag position
    pub fn define_component(&mut self, name: &str) -> ComponentIndex {
        let comp_index = self.components.len();
        self.components.push(Component::new(comp_index.clone(), name));
        comp_index
    }

    pub fn get_component(&self, name: &str) -> Option<&Component> {
        self.components.iter().find(|x| *x.name() == NameTag::from_str(name))
    }

    pub fn add_component(&mut self, target: &usize, component: &ComponentIndex) {
        self.entities[*target].set_bit(*component, true);
    }

    pub fn remove_component(&mut self, target: &usize, component: &ComponentIndex) {
        self.entities[*target].set_bit(*component, false);
    }

    pub fn spawn(&mut self, name: &str) -> Option<usize> {
        if let Some(pointer) = self.free.pop() {
            self.active.push(pointer);
            return Some(pointer);
        }
        None
    }

    pub fn destroy(&mut self, target: &usize){
        if let Some(i) = self.active.iter().position(|pointer| pointer == target) {
            self.active.remove(i);
            self.free.push(i);
        }
    }
}

#[derive(Default, Clone, PartialEq)]
struct NameTag([u8; 16]);

impl NameTag {
    pub fn from_str(s: &str) -> Self {
        Self::from_string(s.to_string())
    }

    pub fn from_string(s: String) -> Self {
        let mut b = s.into_bytes();
        b.resize(16, 0);
        NameTag([
            b[0], b[1], b[2], b[3], 
            b[4], b[5], b[6], b[7],
            b[8], b[9], b[10], b[11], 
            b[12], b[13], b[14], b[15]
            ])
    }

    pub fn to_string(&self) -> String {
        let mut v = Vec::<u8>::new();
        v.extend_from_slice(&self.0);
        String::from_utf8(v).unwrap_or(String::new())
    }
}

#[derive(Clone)]
struct Component {
    index: usize,
    name: NameTag,
}

impl Component {
    fn new(index: usize, name: &str) -> Self {
        Component {
            index,
            name: NameTag::from_str(name),
        }
    }

    pub fn index(&self) -> &usize { &self.index }
    pub fn name(&self) -> &NameTag { &self.name }

    pub fn compare_str(&self, name: &str) -> bool {
        self.name == NameTag::from_str(name)
    }
}

type ComponentList = BitFlags;

impl ComponentList {
    fn new() -> Self { Self(0) }
    fn add(&mut self, component: Component) {
        self.set_bit(*component.index(), true)
    }
}
impl PartialEq for ComponentList {
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}

#[derive(Default, Debug)]
struct BitFlags(u32);
impl BitFlags {
    pub fn set_bit(&mut self, at_index: usize, to: bool) {
        match to {
            true => self.enable_bits(1 << at_index),
            false => self.disable_bits(1 << at_index),
        }
    }
    pub fn enable_bits(&mut self, bits: u32) { self.0 |= bits }
    pub fn disable_bits(&mut self, bits: u32) { self.0 &= !bits }
}

trait System<T: Default> {
    fn init(&mut self, components: &Vec<Component>);
    fn requires_components(&self) -> &ComponentList;
    fn update(&mut self, target: &ObjectIndex, objects: &mut Objects<T>, active: &Vec<usize>);//, actions: &mut Actions);
    //fn on_spawn_response(&mut self, spawn: &mut Entity, object: &mut T);
}


/// USER DEFINED
/// 

fn main() {
    let mut ecs = Ecs::new();

    let comp1 = ecs.define_component("call-1").clone();
    let comp2 = ecs.define_component("call-2").clone();
    let comp3 = ecs.define_component("call-3").clone();

    let system1 = ecs.define_system("system-call-1", Box::new(Call1::new()));
    let system2 = ecs.define_system("system-call-2", Box::new(Call2::new()));
    let system3 = ecs.define_system("system-call-3", Box::new(Call3::new()));

    let entity1 = ecs.spawn("entity-1").unwrap();
    ecs.add_component(&entity1, &comp1);

    let entity2 = ecs.spawn("entity-2").unwrap();
    ecs.add_component(&entity2, &comp2);

    let entity3 = ecs.spawn("entity-3").unwrap();
    ecs.add_component(&entity3, &comp3);

    let num_updates: u128 = 100_000_000;
    let num_calls: f64 = 3.0 * num_updates as f64;

    loop {
        let now = SystemTime::now();

        for _i in 0..num_updates { ecs.update(); }

        let elapsed_res = now.elapsed();
        match elapsed_res {
            Ok(elapsed) => println!("updates {} M calls/s", (1_000.0 / elapsed.as_nanos() as f64) * num_calls),
            Err(e)      => println!("Error: {:?}", e),
        }

        // println!(" - result 1: {}, {}, {}", ecs.scene.borrow(&entity1).call1.clone(), ecs.scene.borrow(&entity1).call2.clone(), ecs.scene.borrow(&entity1).call3.clone());
        // println!(" - result 2: {}, {}, {}", ecs.scene.borrow(&entity2).call1.clone(), ecs.scene.borrow(&entity2).call2.clone(), ecs.scene.borrow(&entity2).call3.clone());
        // println!(" - result 3: {}, {}, {}", ecs.scene.borrow(&entity3).call1.clone(), ecs.scene.borrow(&entity3).call2.clone(), ecs.scene.borrow(&entity3).call3.clone());
    }
}

#[derive(Default, Debug)]
struct Cell {
    pub call1: u128,
    pub call2: u128,
    pub call3: u128,
}


#[derive(Default)]
struct Call1 (ComponentList);

impl Call1 {
    pub fn new() -> Self { Self (ComponentList::new()) }
}

impl System<Cell> for Call1 {

    fn init(&mut self, components: &Vec<Component>) {
        if let Some(c) = components.iter().find(|c| c.compare_str("call-1") ) {
            self.0.add(c.clone());
        }
    }

    fn requires_components(&self) -> &ComponentList { &self.0 }

    fn update(&mut self, target: &ObjectIndex, objects: &mut Objects<Cell>, active: &Vec<usize>) {//, actions: &mut Actions) {
        objects[*target].call1 += 1;
        //actions.spawn("test");
    }

    // fn on_spawn_response(&mut self, spawn: &mut Entity, object: &mut Cell) {

    // }
}

#[derive(Default)]
struct Call2 (ComponentList);

impl Call2 {
    pub fn new() -> Self { Self (ComponentList::new()) }
}

impl System<Cell> for Call2 {

    fn init(&mut self, components: &Vec<Component>) {
        if let Some(c) = components.iter().find(|c| c.compare_str("call-2") ) {
            self.0.add(c.clone());
        }
    }

    fn requires_components(&self) -> &ComponentList { &self.0 }

    fn update(&mut self, target: &ObjectIndex, objects: &mut Objects<Cell>, active: &Vec<usize>){//, actions: &mut Actions) {
        objects[*target].call2 += 1
    }

    // fn on_spawn_response(&mut self, spawn: &mut Entity, object: &mut Cell) {

    // }
}

#[derive(Default)]
struct Call3 (ComponentList);

impl Call3 {
    pub fn new() -> Self { Self (ComponentList::new()) }
}

impl System<Cell> for Call3 {

    fn init(&mut self, components: &Vec<Component>) {
        if let Some(c) = components.iter().find(|c| c.compare_str("call-3") ) {
            self.0.add(c.clone());
        }
    }

    fn requires_components(&self) -> &ComponentList { &self.0 }

    fn update(&mut self, target: &ObjectIndex, objects: &mut Objects<Cell>, active: &Vec<usize>){//, actions: &mut Actions) {
        objects[*target].call3 += 1;
    }

    // fn on_spawn_response(&mut self, spawn: &mut Entity, object: &mut Cell) {

    // }
}
