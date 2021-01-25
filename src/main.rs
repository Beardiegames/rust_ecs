use std::fmt::Debug;
use std::convert::TryInto;
use std::time::{SystemTime};


const MAX_OBJECTS: usize = 10000;

type Entities = [BitFlags; MAX_OBJECTS];
type ObjectIndex = usize;
type ComponentIndex = usize;

struct Objects<T: Default> {
    pool: [T; MAX_OBJECTS],
    active: Vec<(ObjectIndex, NameTag)>,
}

impl<T: Default> Objects<T> {

    pub fn get_mut(&mut self, target: &ObjectIndex) -> &mut T {
        &mut self.pool[*target]
    }

    pub fn get_ref(&mut self, target: &ObjectIndex) -> &T {
        &self.pool[*target]
    }

    pub fn find(&mut self, name: &str) -> Option<&ObjectIndex> {
        let tag = NameTag::from_str(name);
        self.active.iter().find(|x| x.1 == tag).map(|a| &a.0)
    }
}

struct Ecs<T: Default> { 
    objects: Objects<T>, // object data pool, in other words entity component data
    entities: Entities, // object component implementation flags
    systems: Vec<System<T>>, // behaviour wrappers for executing custom behaviour scripts
    components: ComponentRefs, // component definitions, flag position & amount of components available
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
            objects: Objects { 
                pool: create_objects.try_into().unwrap(),
                active: Vec::new(),
            },
            entities: create_entities.try_into().unwrap(),
            systems: Vec::new(),
            components: ComponentRefs(Vec::new()),
            active: Vec::new(),
            free,
        }
    }

    fn update(&mut self) {
        // update routine
        for system in &mut self.systems {
            for pointer in &self.active {
                if self.entities[*pointer].0 == 
                    self.entities[*pointer].0 & system.component_flags().0 
                {
                    system.behaviour.update(&pointer, &mut self.objects);//, &mut self.actions);
                }
            }
        }

        // handle actions
    }

    pub fn define_system(&mut self, behaviour: Box<dyn Behaviour<T>>) {
        self.systems.push(System::new(behaviour, &self.components));
    }

    // setup new components by giving them a name and a flag position
    pub fn define_component(&mut self, name: &str) -> ComponentIndex {
        let comp_index = self.components.0.len();
        self.components.0.push(ComponentRef::new(comp_index.clone(), name));
        comp_index
    }

    pub fn get_component(&self, name: &str) -> Option<&ComponentRef> {
        self.components.0.iter().find(|x| *x.name() == NameTag::from_str(name))
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
            self.objects.active.push((pointer, NameTag::from_str(name)));
            return Some(pointer);
        }
        None
    }

    pub fn destroy(&mut self, target: &usize){
        if let Some(i) = self.active.iter().position(|pointer| pointer == target) {
            self.active.remove(i);
            self.objects.active.remove(i);
            self.free.push(i);
        }
    }
}

#[derive(Default, Clone, PartialEq)]
struct NameTag ([u8; 16]);

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


struct ComponentRefs(Vec<ComponentRef>);

impl ComponentRefs {

    pub fn get(&self, tag: &NameTag) -> Option<&ComponentRef> {
        self.0.iter().find(|c| c.name == *tag)
    }

    pub fn list(&self) -> &Vec<ComponentRef> {
        &self.0
    }
}


#[derive(Clone)]
struct ComponentRef {
    index: usize,
    name: NameTag,
}

impl ComponentRef {
    fn new(index: usize, name: &str) -> Self {
        ComponentRef {
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

struct System<T: Default> {
    behaviour: Box<dyn Behaviour<T>>,
    spawn_req: Vec<ObjectIndex>,
    destroy_req: Vec<ObjectIndex>,
    components: BitFlags,
}

impl<T: Default> System<T> {

    fn new(behaviour: Box<dyn Behaviour<T>>, component_refs: &ComponentRefs) -> Self {
        let mut components = BitFlags (0);

        for s in &mut behaviour.required_components().iter() {
            if let Some(c) = component_refs.get(s) {
                components.set_bit(*c.index(), true)
            }
        }

        System {
            behaviour,
            spawn_req: Vec::new(),
            destroy_req: Vec::new(),
            components,
        }
    }

    fn component_flags(&self) -> &BitFlags {
        &self.components
    }

    fn add_component(&mut self, component: ComponentRef) {
        self.components.set_bit(*component.index(), true)
    }
}

trait Behaviour<T: Default> {
    fn required_components(&self) -> Vec<NameTag>;
    fn update(&mut self, target: &ObjectIndex, objects: &mut Objects<T>);
}


/// USER DEFINED
/// 

fn main() {
    let mut ecs = Ecs::new();

    let comp1 = ecs.define_component("call-1").clone();
    let comp2 = ecs.define_component("call-2").clone();
    let comp3 = ecs.define_component("call-3").clone();

    let system1 = ecs.define_system(Box::new(Call1));
    let system2 = ecs.define_system(Box::new(Call2));
    let system3 = ecs.define_system(Box::new(Call3));

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

        // println!(" - result 1: {}, {}, {}", ecs.objects.get_ref(&entity1).call1.clone(), ecs.objects.get_ref(&entity1).call2.clone(), ecs.objects.get_ref(&entity1).call3.clone());
        // println!(" - result 2: {}, {}, {}", ecs.objects.get_ref(&entity2).call1.clone(), ecs.objects.get_ref(&entity2).call2.clone(), ecs.objects.get_ref(&entity2).call3.clone());
        // println!(" - result 3: {}, {}, {}", ecs.objects.get_ref(&entity3).call1.clone(), ecs.objects.get_ref(&entity3).call2.clone(), ecs.objects.get_ref(&entity3).call3.clone());
    }
}

#[derive(Default, Debug)]
struct Cell {
    pub call1: u128,
    pub call2: u128,
    pub call3: u128,
}


#[derive(Default)]
struct Call1;

impl Behaviour<Cell> for Call1 {

    fn required_components(&self) -> Vec<NameTag> { 
        vec![ NameTag::from_str("call-1") ] 
    }

    fn update(&mut self, target: &ObjectIndex, objects: &mut Objects<Cell>) {
        objects.get_mut(target).call1 += 1;
    }
}

#[derive(Default)]
struct Call2;

impl Behaviour<Cell> for Call2 {

    fn required_components(&self) -> Vec<NameTag> { 
        vec![ NameTag::from_str("call-2") ] 
    }

    fn update(&mut self, target: &ObjectIndex, objects: &mut Objects<Cell>) {
        objects.get_mut(target).call2 += 1
    }
}

#[derive(Default)]
struct Call3;

impl Behaviour<Cell> for Call3 {

    fn required_components(&self) -> Vec<NameTag> { 
        vec![ NameTag::from_str("call-3") ] 
    }

    fn update(&mut self, target: &ObjectIndex, objects: &mut Objects<Cell>) {
        objects.get_mut(target).call3 += 1;
    }
}
