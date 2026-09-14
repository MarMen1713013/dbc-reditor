#[derive(Copy, Clone, Hash, Eq, PartialEq, Debug)]
pub struct NodeId {
    id: u64,
}

impl NodeId {
    pub(crate) fn new(input: u64) -> Self{
        Self {id: input}
    }
}

pub struct Node {
    name: String,
}

impl Node {
    fn new(name: &str) -> Self{
        Self {
            name: String::from(name),
        }
    }
    fn name(&self) -> &str {
        &self.name
    }
}

#[derive(Eq, PartialEq, Debug)]
pub enum NodeError {
}
