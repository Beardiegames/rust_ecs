
use super::ComponentIndex;

#[derive(Default, Clone, PartialEq)]
pub struct NameTag (pub(crate) [u8; 16]);

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


pub struct ComponentRefs(pub(crate) Vec<ComponentRef>);

impl ComponentRefs {

    pub fn get(&self, tag: &NameTag) -> Option<&ComponentRef> {
        self.0.iter().find(|c| c.name == *tag)
    }

    pub fn list(&self) -> &Vec<ComponentRef> {
        &self.0
    }
}

#[derive(Clone)]
pub struct ComponentRef {
    index: ComponentIndex,
    name: NameTag,
}

impl ComponentRef {
    pub(crate) fn new(index: ComponentIndex, name: &str) -> Self {
        ComponentRef {
            index,
            name: NameTag::from_str(name),
        }
    }

    pub fn index(&self) -> &ComponentIndex { &self.index }
    pub fn name(&self) -> &NameTag { &self.name }
}


#[derive(Default, Debug)]
pub struct BitFlags(pub(crate) u32);

impl BitFlags {
    pub fn reset(&mut self) { self.0 = 0; }

    pub fn set_bit(&mut self, at_index: ComponentIndex, to: bool) {
        match to {
            true => self.enable_bits(1 << at_index),
            false => self.disable_bits(1 << at_index),
        }
    }

    pub fn enable_bits(&mut self, bits: u32) { self.0 |= bits }
    pub fn disable_bits(&mut self, bits: u32) { self.0 &= !bits }
}
