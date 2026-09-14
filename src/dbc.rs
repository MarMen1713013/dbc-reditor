use std::collections::HashMap;

use crate::{
    message::{Message, MessageId, MessageError},
    node::{Node, NodeId, NodeError},
};

pub struct Dbc {
    nodes: HashMap<NodeId, Node>,
    next_node_id: u64,
    messages: HashMap<MessageId, Message>,
    next_msg_id: u64,
}

impl Dbc {
    pub fn new() -> Self {
        Self {
            next_msg_id: 0,
            next_node_id: 0,
            nodes: HashMap::new(),
            messages: HashMap::new(),
        }
    }
    pub fn add_node(&mut self, node: Node) -> NodeId {
        let n_id = NodeId::new(self.next_node_id);
        self.next_node_id += 1;

        self.nodes.insert(n_id, node);
        n_id
    }
    pub fn get_node(&self, id: NodeId) -> Option<&Node> {
        self.nodes.get(&id)
    }
    pub fn add_message(&mut self, msg: Message) -> Result<MessageId, DbcError> {
        if let Some(n_id) = msg.sender() {
            if self.get_node(n_id).is_none() {
                return Err(DbcError::MessageSenderNotAvailable);
            }
        }
        let m_id = MessageId::new(self.next_msg_id);
        self.messages.insert(m_id, msg);
        self.next_msg_id += 1;

        Ok(m_id)
    }
    pub fn get_message(&self, m_id: MessageId) -> Option<&Message>{
        self.messages.get(&m_id)
    }
}

#[derive(Eq, PartialEq, Debug)]
pub enum DbcError {
    MessageSenderNotAvailable,
}
