
use std::time::{SystemTime};
use std::cell::{RefCell, RefMut, Ref};
use std::collections::VecDeque;

type Pointer = usize;

struct Pool<T> {
    pool: Vec<T>,
    active: Vec<Pointer>,
    free: Vec<Pointer>,
    iter: Pointer,
}

impl<T> Pool<T> {
    fn new<F: FnMut(usize) -> T>(size: usize, mut on_new: F) -> Self {
        let mut pool = Vec::<T>::with_capacity(size);
        let mut active = Vec::with_capacity(size);
        let mut free = Vec::with_capacity(size);
        
        for i in 0..size {
            pool.push(on_new(i));
            free.push(i)
        }
        Pool { pool, active, free, iter: 0 }
    }

    fn spawn(&mut self) -> Option<Pointer> {
        if let Some(pointer) = self.free.pop() {
            self.active.push(pointer);
            return Some(pointer);
        }
        None
    }

    fn destroy(&mut self, pointer: &Pointer) {
        if let Some(i) = self.active.iter().position(|p| p == pointer) {
            self.active.remove(i);
            self.free.push(i);
        }
    }

    pub fn borrow(&self, pointer: &Pointer) -> &T {
        &self.pool[*pointer]
    }

    pub fn edit(&mut self, pointer: &Pointer) -> &mut T {
        &mut self.pool[*pointer]
    }

    pub fn find<P>(&self, mut predicate: P) -> Option<Pointer>
    where P: FnMut(&T) -> bool {
        
        for active in &self.active {
            if predicate(&self.pool[*active]) {
                return Some(*active);
            }
        }
        None
    }

    pub fn list(&self) -> &Vec<Pointer> {
        &self.active
    }

    pub fn use_count(&self) -> usize {
        self.active.len()
    }

    pub fn most_recent(&self) -> &Pointer {
        &self.active[self.active.len()-1]
    }
}


struct Ecs<T: Default> {
    objects: Pool<T>, // object data pool, in other words entity component data
    entities: Pool<Entity>, // object pool reference & implement component flags
    components: Components, // component definitions, flag position & amount of components available
    systems: Vec<Box<dyn System<T>>>, // behaviour wrappers for executing custom behaviour scripts
}

impl<T: Default> Ecs<T> {
    pub fn new(size: usize) -> Self {
        let objects: Pool<T> = Pool::new(size, |i| { 
            Default::default() 
        });
        let entities: Pool<Entity> = Pool::new(size, |i| { 
            Entity::new(i, &format!("unused-{}", i)) 
        });

        Ecs { 
            objects, 
            entities, 
            components: Components::new(),
            systems: Vec::new(), 
        }
    }

    pub fn update(&mut self) {
        for system in &mut self.systems {
            for entity in self.entities.list() {
                let comp = &self.entities.borrow(entity).components;
                if comp.0 == comp.0 & system.requires_components().0
                {
                    system.update(self.entities.borrow(entity), &mut self.objects);
                }
            }
        }
    }

    pub fn define_system(&mut self, name: &str, system: Box<dyn System<T>>) {
        let index = self.systems.len();
        self.systems.push(system);
        self.systems[index].init(&self.components);
    }

    // setup new components by giving them a name and a flag position
    pub fn define_component(&mut self, name: &str) -> &Component {
        let num_components = self.components.0.len();
        self.components.0.push(Component::new(num_components, name));
        &self.components.0[num_components]
    }

    pub fn add_component(&mut self, entity: &Entity, component: &Component) {
        self.entities.edit(entity.pointer()).set_component(component.clone(), true);
    }

    pub fn remove_component(&mut self, entity: &Entity, component: &Component) {
        self.entities.edit(entity.pointer()).set_component(component.clone(), false);
    }

    pub fn spawn(&mut self, name: &str) -> Option<Entity> {
        if self.objects.spawn().is_some() {
            if let Some(pointer) = self.entities.spawn() {
                self.entities.edit(&pointer).rename(name);
                return Some(self.entities.edit(&pointer).clone())
            }
        }
        None
    }

    pub fn destroy(&mut self, entity: &Entity){
        self.objects.destroy(entity.pointer());
        self.entities.destroy(entity.pointer());
    }

    pub fn find(&mut self, name: &str) -> Option<Entity> {
        self.entities
            .find(|e| *e.name() == NameTag::from_str(name))
            .map(|p| self.entities.borrow(&p).clone())
    }

    pub fn borrow(&mut self, entity: &Entity) -> &T {
        self.objects.borrow(entity.pointer())
    }

    pub fn edit(&mut self, entity: &Entity) -> &mut T {
        self.objects.edit(entity.pointer())
    }
}

#[derive(Clone, Default, PartialEq)]
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
struct ObjectRef {
    pointer: usize,
    name: NameTag,
}

impl ObjectRef {
    fn new(pointer: usize, name: &str) -> Self {
        Self { pointer, name: NameTag::from_str(name) } 
    }

    pub fn pointer(&self) -> &usize { &self.pointer }
    pub fn name(&self) -> &NameTag { &self.name }

    pub fn rename(&mut self, name: &str) {
        self.name = NameTag::from_str(name)
    }
}


#[derive(Clone)]
struct Entity {
    obj_ref: ObjectRef,
    components: ComponentList,
}
impl Entity {
    fn new(pointer: usize, name: &str) -> Self {
        Self { 
            obj_ref: ObjectRef::new(pointer, name), 
            components: ComponentList::new() 
        } 
    }

    pub fn pointer(&self) -> &usize { &self.obj_ref.pointer }
    pub fn name(&self) -> &NameTag { &self.obj_ref.name }
    pub fn rename(&mut self, name: &str) { self.obj_ref.rename(name); }

    pub fn set_component(&mut self, component: Component, to: bool) { 
        self.components.set_bit(*component.pointer(), to);
    }
}
impl PartialEq for Entity {
    fn eq(&self, other: &Entity) -> bool {
        self.pointer() == other.pointer()
    }
}

type Component = ObjectRef;

struct Components(Vec<Component>);
impl Components {
    fn new() -> Self {
        Self(Vec::with_capacity(32))
    }
    fn get(&self, name: &str) -> Option<&Component> {
        self.0.iter().find(|x| *x.name() == NameTag::from_str(name))
    }
}

type ComponentList = BitFlags;

impl ComponentList {
    fn new() -> Self { Self(0) }
    fn add(&mut self, component: Component) {
        self.set_bit(*component.pointer(), true)
    }
    fn has(&self, other: &Self) -> bool {
        other.0 == self.0 & other.0
    }
}
impl PartialEq for ComponentList {
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}

#[derive(Clone, Default)]
struct BitFlags(u32);
impl BitFlags {
    pub fn set_bit(&mut self, position: usize, to: bool) {
        match to {
            true => self.enable_bits(1 << position),
            false => self.disable_bits(1 << position),
        }
    }
    pub fn enable_bits(&mut self, bits: u32) { self.0 |= bits }
    pub fn disable_bits(&mut self, bits: u32) { self.0 &= !bits }
}

trait System<T: Default> {
    fn init(&mut self, components: &Components);
    fn requires_components(&self) -> &ComponentList;
    fn update(&mut self, entity: &Entity, objects: &mut Pool<T>);
}


/// USER DEFINED
/// 

fn main() {
    let mut ecs = Ecs::new(100);

    let comp1 = ecs.define_component("call-1").clone();
    let comp2 = ecs.define_component("call-2").clone();
    let comp3 = ecs.define_component("call-3").clone();

    let system1 = ecs.define_system("system-call-1", Box::new(Call1::new()));
    let system2 = ecs.define_system("system-call-2", Box::new(Call2::new()));
    let system3 = ecs.define_system("system-call-3", Box::new(Call3::new()));

    let entity1 = ecs.spawn("entity-1").unwrap();
    ecs.add_component(&entity1, &comp1);

    let entity2 = ecs.spawn("entity-2").unwrap();
    ecs.add_component(&entity2, &comp1);

    let entity3 = ecs.spawn("entity-3").unwrap();
    ecs.add_component(&entity3, &comp3);

    let num_updates: u128 = 100_000_000;
    let num_calls: f64 = 3.0 * num_updates as f64;

    // loop {
    //     let now = SystemTime::now();
    //     while now.elapsed().unwrap().as_nanos() < 1_000_000_000 {
    //         ecs.update();
    //     }
    //     println!("updates {} calls/s", ecs.borrow(&entity1).call1.clone() * 3);
    //     ecs.edit(&entity1).call1 = 0;
    // }

    loop {
        let now = SystemTime::now();

        for _i in 0..num_updates { ecs.update(); }

        let elapsed_res = now.elapsed();
        match elapsed_res {
            Ok(elapsed) => println!("updates {} calls/s", (1_000_000_000.0 / elapsed.as_nanos() as f64) * num_calls),
            Err(e)      => println!("Error: {:?}", e),
        }

        // println!(" - result 1: {}, {}, {}", ecs.borrow(&entity1).call1.clone(), ecs.borrow(&entity1).call2.clone(), ecs.borrow(&entity1).call3.clone());
        // println!(" - result 2: {}, {}, {}", ecs.borrow(&entity2).call1.clone(), ecs.borrow(&entity2).call2.clone(), ecs.borrow(&entity2).call3.clone());
        // println!(" - result 3: {}, {}, {}", ecs.borrow(&entity3).call1.clone(), ecs.borrow(&entity3).call2.clone(), ecs.borrow(&entity3).call3.clone());
    }
}

#[derive(Default)]
struct Cell {
    pub call1: u128,
    pub call2: u128,
    pub call3: u128,
}


#[derive(Default)]
struct Call1 {
    requirements: ComponentList,
}

impl Call1 {
    pub fn new() -> Self {
        Self { requirements: ComponentList::new() }
    }
}

impl System<Cell> for Call1 {

    fn init(&mut self, components: &Components) {
        self.requirements.add(components.get("call-1").unwrap().clone());
    }

    fn requires_components(&self) -> &ComponentList {
        &self.requirements
    }

    fn update(&mut self, entity: &Entity, objects: &mut Pool<Cell>) {
        objects.edit(entity.pointer()).call1 += 1
    }
}

#[derive(Default)]
struct Call2{
    requirements: ComponentList,
}

impl Call2 {
    pub fn new() -> Self {
        Self { requirements: ComponentList::new() }
    }
}

impl System<Cell> for Call2 {

    fn init(&mut self, components: &Components) {
        self.requirements.add(components.get("call-2").unwrap().clone());
    }

    fn requires_components(&self) -> &ComponentList {
        &self.requirements
    }

    fn update(&mut self, entity: &Entity, objects: &mut Pool<Cell>) {
        objects.edit(entity.pointer()).call2 += 1
    }
}

#[derive(Default)]
struct Call3{
    requirements: ComponentList,
}

impl Call3 {
    pub fn new() -> Self {
        Self { requirements: ComponentList::new() }
    }
}

impl System<Cell> for Call3 {

    fn init(&mut self, components: &Components) {
        self.requirements.add(components.get("call-3").unwrap().clone());
    }

    fn requires_components(&self) -> &ComponentList {
        &self.requirements
    }

    fn update(&mut self, entity: &Entity, objects: &mut Pool<Cell>) {
        objects.edit(entity.pointer()).call3 += 1
    }
}
