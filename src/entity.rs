
#[derive(Clone)]
pub struct Entity {
    name: String,
    index: usize,
}

impl Entity {
    pub fn blank () -> Self {
        Entity { 
            name: "".to_string(), 
            index: 0, 
        }
    }

    pub fn name(&self) -> &str {
        self.name.as_str()
    }

    pub fn index(&self) -> usize {
        self.index
    }
}

impl PartialEq for Entity {
    fn eq(&self, other: &Self) -> bool {
        self.index == other.index
    }
}

pub struct Entities {
    list: Vec<Entity>,
    using: Vec<usize>,
    freed: Vec<usize>,
}

#[allow(dead_code)]
impl Entities {
    pub fn new (size: usize) -> Self {
        let mut list: Vec<Entity> = vec![Entity::blank(); size];

        for i in 0..size { list[i].index = i; }

        Entities { 
            list, 
            using: Vec::with_capacity(size),
            freed: Vec::with_capacity(size),
        }
    }

    fn next_free (&mut self) -> Option<usize> {
        if self.using.len() < self.list.len() {
            Some(self.using.len())
        } else {
            self.freed.pop()
        }
    }

    pub fn spawn (&mut self, name: &str) -> Option<Entity> {
        match self.next_free() {
            Some(i) => {
                let e = &mut self.list[i];
                e.name = name.to_string();
                self.using.push(i);

                Some(e.clone())
            },
            None => None,
        }
    }

    pub fn destroy (&mut self, e: &Entity) {
        self.freed.push(e.index());
        self.using.remove(e.index());
    }

    pub fn find <P: FnMut(&usize, &str) -> bool> (&self, mut predicate: P) -> Option<Entity> {
        match self.using.iter().find(|x| predicate(*x, self.list[**x].name())) {
            Some (result) => Some(self.list[*result].clone()),
            None => None,
        }
    }
}