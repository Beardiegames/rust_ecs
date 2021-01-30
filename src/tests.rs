
#[cfg(test)]
use std::time::SystemTime;
use super::*;

#[test]
fn update_speed() {

    let mut ecs = EcsBuilder::new()
            .define_component("call-1")
            .define_component("call-2")
            .define_component("call-3")
        .build_systems()

            .define_system(Box::new(Call1))
            .define_system(Box::new(Call2))
            .define_system(Box::new(Call3))
        .setup_factories()

            .define_factory("type-1", Box::new(factory_1))
            .define_factory("type-2", Box::new(factory_2))
            .define_factory("type-3", Box::new(factory_3))
        .finalize();


    ecs.spawn("entity-1", "type-1");
    ecs.spawn("entity-2", "type-2");
    ecs.spawn("entity-3", "type-3");
    ecs.start();

    let num_updates: u128 = 100_000_000;
    let num_calls: f64 = 3.0 * num_updates as f64;
    let now = SystemTime::now();

    for _i in 0..num_updates { ecs.update(); }

    let elapsed_res = now.elapsed();
    match elapsed_res {
        Ok(elapsed) => println!("updates {} M calls/s", (1_000.0 / elapsed.as_nanos() as f64) * num_calls),
        Err(e)      => println!("Error: {:?}", e),
    }

    if let Some(entity1) = ecs.objects.find("entity-1") {
        println!(" - result 1: {}, {}, {}", 
            ecs.objects.get_ref(entity1).call1, 
            ecs.objects.get_ref(entity1).call2, 
            ecs.objects.get_ref(entity1).call3
        );
    }
    if let Some(entity2) = ecs.objects.find("entity-2") {
        println!(" - result 2: {}, {}, {}", 
            ecs.objects.get_ref(entity2).call1, 
            ecs.objects.get_ref(entity2).call2, 
            ecs.objects.get_ref(entity2).call3
        );
    }if let Some(entity3) = ecs.objects.find("entity-3") {
        println!(" - result 3: {}, {}, {}", 
            ecs.objects.get_ref(entity3).call1, 
            ecs.objects.get_ref(entity3).call2, 
            ecs.objects.get_ref(entity3).call3
        );
    }if let Some(test) = ecs.objects.find("test") {
        println!(" - test: {}, {}, {}", 
            ecs.objects.get_ref(test).call1, 
            ecs.objects.get_ref(test).call2, 
            ecs.objects.get_ref(test).call3
        );
    }
    assert!(false);
}

#[test]
fn open_update_speed() {

    let mut ecs:Ecs<Cell> = EcsBuilder::new()
        .build_systems()
        .setup_factories()
            .define_factory("type-1", Box::new(factory_1))
        .finalize();

    ecs.spawn("entity-1", "type-1");
    ecs.start();
    
    let num_updates: u128 = 100_000_000;
    let now = SystemTime::now();

    for _i in 0..num_updates { 
        ecs.open_update(|obj| obj.call1 += 1 ); 
    }

    let elapsed_res = now.elapsed();
    match elapsed_res {
        Ok(elapsed) => println!("updates {} M calls/s", 1_000.0 / elapsed.as_nanos() as f64),
        Err(e)      => println!("Error: {:?}", e),
    }

    if let Some(entity1) = ecs.objects.find("entity-1") {
        println!(" - result 1: {}", 
            ecs.objects.get_ref(entity1).call1, 
        );
    }
    assert!(false);
}

#[derive(Default, Debug)]
struct Cell {
    pub spawned: bool,
    pub call1: u128,
    pub call2: u128,
    pub call3: u128,
}

fn factory_1<'a, T: Default>(tools: &mut BuildTools<T>) {
    tools.add_component("call-1")
}

fn factory_2<'a, T: Default>(tools: &mut BuildTools<T>) {
    tools.add_component("call-2")
}

fn factory_3<'a, T: Default>(tools: &mut BuildTools<T>) {
    tools.add_component("call-3")
}


#[derive(Default)]
struct Call1;

impl Behaviour<Cell> for Call1 {

    fn required_components(&self) -> Vec<NameTag> { 
        vec![ NameTag::from_str("call-1") ] 
    }

    #[allow(unused_variables)]
    fn on_start(&mut self, objects: &mut Objects<Cell>, system: &mut System) {
        system.spawn("test", "type-2");
    }

    #[allow(unused_variables)]
    fn on_update(&mut self, target: &ObjectIndex, objects: &mut Objects<Cell>, system: &mut System) {
        objects.get_mut(target).call1 += 1;
    }
}

#[derive(Default)]
struct Call2;

impl Behaviour<Cell> for Call2 {

    fn required_components(&self) -> Vec<NameTag> { 
        vec![ NameTag::from_str("call-2") ] 
    }

    #[allow(unused_variables)]
    fn on_update(&mut self, target: &ObjectIndex, objects: &mut Objects<Cell>, system: &mut System) {
        objects.get_mut(target).call2 += 1
    }
}

#[derive(Default)]
struct Call3;

impl Behaviour<Cell> for Call3 {

    fn required_components(&self) -> Vec<NameTag> { 
        vec![ NameTag::from_str("call-3") ] 
    }

    #[allow(unused_variables)]
    fn on_update(&mut self, target: &ObjectIndex, objects: &mut Objects<Cell>, system: &mut System) {
        objects.get_mut(target).call3 += 1;
    }
}
