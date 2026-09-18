#[derive(Copy, Clone, Hash, Eq, PartialEq, Debug)]
pub struct NodeId {
    id: u64,
}

impl NodeId {
    pub(crate) fn new(input: u64) -> Self {
        Self { id: input }
    }
}

pub struct Node {
    name: String,
}

impl Node {
    pub(crate) fn new(name: &str) -> Self {
        Self {
            name: String::from(name),
        }
    }
    pub fn name(&self) -> &str {
        &self.name
    }
    pub(crate) fn apply_command(&mut self, cmd: NodeCommand) -> Result<(), NodeError> {
        match cmd {
            NodeCommand::Rename(new_name) => {
                self.name = new_name;
            }
        }
        Ok(())
    }
}

#[derive(Eq, PartialEq, Debug)]
pub enum NodeError {}

#[derive(Debug, Clone, PartialEq)]
pub enum NodeCommand {
    Rename(String),
}
